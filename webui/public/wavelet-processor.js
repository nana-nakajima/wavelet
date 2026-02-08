// ==========================================================================
// wavelet-processor.js — AudioWorkletProcessor
//
// Runs on the audio rendering thread. Loads the Rust WASM module, reads
// parameters from a SharedArrayBuffer written by the UI thread, and writes
// audio output + engine state back.
//
// Communication:
//   UI → Worklet:  SharedArrayBuffer (params)  +  postMessage (commands)
//   Worklet → UI:  SharedArrayBuffer (state)   +  postMessage (events)
// ==========================================================================

// SharedArrayBuffer layout offsets (must match Rust SAB_* constants)
const SAB_TRANSPORT_FLAGS = 0;
const SAB_TEMPO = 1;
const SAB_CURRENT_STEP = 2;
const SAB_MASTER_VOLUME = 3;
const SAB_TRACK_VOLUMES = 4;
const SAB_TRACK_PANS = 20;
const SAB_TRACK_MUTES = 36;
const SAB_TRACK_SOLOS = 52;
const SAB_TRACK_PARAMS = 68;
const SAB_WAVEFORM = 196;
const SAB_ACTIVE_TRACK = 260;
const SAB_PEAK_L = 261;
const SAB_PEAK_R = 262;
const SAB_CPU_LOAD = 263;
const SAB_TOTAL_FLOATS = 264;

class WaveletProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();

    /** @type {Float32Array|null} backed by SharedArrayBuffer */
    this._paramBlock = null;

    /** @type {object|null} WASM exports */
    this._wasm = null;

    /** @type {object|null} WasmAudioHost instance (opaque ptr) */
    this._host = null;

    /** @type {boolean} */
    this._ready = false;

    /** @type {Array} queued messages received before WASM is ready */
    this._pendingMessages = [];

    this.port.onmessage = (e) => this._onMessage(e.data);
  }

  // --------------------------------------------------------------------------
  // Message handling
  // --------------------------------------------------------------------------

  _onMessage(msg) {
    switch (msg.type) {
      case "init":
        this._initWasm(msg.wasmBytes, msg.sampleRate);
        break;

      case "attach-sab":
        // Receive the SharedArrayBuffer from the main thread
        this._paramBlock = new Float32Array(msg.sab);
        break;

      case "note-on":
      case "note-off":
      case "load-sample":
      case "clear-sample":
        // These require JSON serialization through the WASM host
        if (this._ready) {
          this._forwardToHost(msg);
        } else {
          this._pendingMessages.push(msg);
        }
        break;
    }
  }

  async _initWasm(wasmBytes, sr) {
    try {
      const { instance } = await WebAssembly.instantiate(wasmBytes, {
        env: {
          // Minimal imports the WASM module may need
          abort: () => { throw new Error("WASM abort"); },
        },
      });

      this._wasm = instance.exports;

      // Call the Rust init_audio function to create the host
      const result = this._wasm.init_audio(sr || sampleRate);
      this._host = result;
      this._ready = true;

      // Flush pending messages
      for (const pending of this._pendingMessages) {
        this._forwardToHost(pending);
      }
      this._pendingMessages = [];

      this.port.postMessage({ type: "ready" });
    } catch (err) {
      this.port.postMessage({ type: "error", message: String(err) });
    }
  }

  _forwardToHost(msg) {
    if (!this._wasm || !this._host) return;

    // Serialize to JSON and pass through handle_message
    try {
      let audioMsg;
      switch (msg.type) {
        case "note-on":
          audioMsg = {
            type: "NoteOn",
            note: msg.note,
            velocity: msg.velocity,
            track: msg.track,
          };
          break;
        case "note-off":
          audioMsg = {
            type: "NoteOff",
            note: msg.note,
            track: msg.track,
          };
          break;
        case "load-sample":
          audioMsg = {
            type: "LoadSample",
            track: msg.track,
            sample_id: msg.sampleId || "sample",
            data: Array.from(msg.data),
          };
          break;
        case "clear-sample":
          audioMsg = {
            type: "ClearSample",
            track: msg.track,
          };
          break;
        default:
          return;
      }

      if (this._wasm.handle_message) {
        this._wasm.handle_message(JSON.stringify(audioMsg));
      }
    } catch (err) {
      this.port.postMessage({
        type: "error",
        message: `Message forward failed: ${err}`,
      });
    }
  }

  // --------------------------------------------------------------------------
  // Audio processing — called 375× per second at 48kHz / 128 frames
  // --------------------------------------------------------------------------

  process(_inputs, outputs, _parameters) {
    if (!this._ready || !this._wasm) {
      return true; // Keep processor alive while loading
    }

    const output = outputs[0];
    if (!output || output.length === 0) return true;

    const channelL = output[0];
    const channelR = output.length > 1 ? output[1] : output[0];
    const frames = channelL.length; // typically 128

    const t0 = currentTime;

    // ── 1. Read UI parameters from SharedArrayBuffer ──
    if (this._paramBlock) {
      // The Rust side reads from this block via read_shared_params.
      // For now, we read the SAB directly and apply via handle_message
      // until the WASM module exposes direct memory access.
      this._syncParamsFromSAB();
    }

    // ── 2. Process audio through the WASM engine ──
    // Create a temporary interleaved buffer
    const mono = new Float32Array(frames);

    if (this._wasm.process) {
      // Direct WASM process call with output buffer
      this._wasm.process(mono);
    }

    // De-interleave to stereo (for now, duplicate mono to both channels)
    for (let i = 0; i < frames; i++) {
      channelL[i] = mono[i];
      channelR[i] = mono[i];
    }

    // ── 3. Compute peak levels ──
    let peakL = 0;
    let peakR = 0;
    for (let i = 0; i < frames; i++) {
      const absL = Math.abs(channelL[i]);
      const absR = Math.abs(channelR[i]);
      if (absL > peakL) peakL = absL;
      if (absR > peakR) peakR = absR;
    }

    // ── 4. Write engine state back to SharedArrayBuffer ──
    if (this._paramBlock) {
      this._paramBlock[SAB_PEAK_L] = peakL;
      this._paramBlock[SAB_PEAK_R] = peakR;

      const elapsed = currentTime - t0;
      const budget = frames / sampleRate;
      this._paramBlock[SAB_CPU_LOAD] = Math.min(1.0, elapsed / budget);
    }

    return true; // Keep processor alive
  }

  // --------------------------------------------------------------------------
  // Sync parameters from SharedArrayBuffer to the WASM engine
  // --------------------------------------------------------------------------

  _syncParamsFromSAB() {
    if (!this._paramBlock || !this._wasm) return;

    const sab = this._paramBlock;

    // Read transport flags and apply via message
    const flags = sab[SAB_TRANSPORT_FLAGS];
    const playing = (flags & 1) !== 0;
    const tempo = sab[SAB_TEMPO];
    const masterVol = sab[SAB_MASTER_VOLUME];

    // Apply transport state changes via handle_message
    // (In a production build, read_shared_params would be called directly
    //  on the Rust side with a pointer to the SAB memory.)
    if (this._wasm.handle_message) {
      try {
        if (playing !== this._lastPlaying) {
          this._wasm.handle_message(
            JSON.stringify({ type: playing ? "Play" : "Stop" })
          );
          this._lastPlaying = playing;
        }

        if (Math.abs(tempo - (this._lastTempo || 120)) > 0.01) {
          this._wasm.handle_message(
            JSON.stringify({ type: "SetTempo", tempo })
          );
          this._lastTempo = tempo;
        }

        if (Math.abs(masterVol - (this._lastMasterVol || 0.8)) > 0.001) {
          this._wasm.handle_message(
            JSON.stringify({ type: "SetMasterVolume", volume: masterVol })
          );
          this._lastMasterVol = masterVol;
        }
      } catch (_) {
        // Swallow serialization errors in the audio thread
      }
    }
  }
}

registerProcessor("wavelet-processor", WaveletProcessor);
