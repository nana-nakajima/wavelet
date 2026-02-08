// ==========================================================================
// wavelet-engine.ts — WASM Initialization & AudioWorklet Loader
//
// This module handles:
//   1. Creating the AudioContext (with user-gesture gate)
//   2. Loading the WASM binary
//   3. Registering the AudioWorkletProcessor
//   4. Allocating the SharedArrayBuffer for zero-copy param sync
//   5. Providing a typed API for the UI to interact with the engine
// ==========================================================================

// SharedArrayBuffer layout — mirrors Rust SAB_* constants and processor.js
export const SAB = {
  TRANSPORT_FLAGS: 0,
  TEMPO: 1,
  CURRENT_STEP: 2,
  MASTER_VOLUME: 3,
  TRACK_VOLUMES: 4,    // +16
  TRACK_PANS: 20,      // +16
  TRACK_MUTES: 36,     // +16
  TRACK_SOLOS: 52,     // +16
  TRACK_PARAMS: 68,    // +128 (16 tracks × 8 params)
  WAVEFORM: 196,       // +64
  ACTIVE_TRACK: 260,
  PEAK_L: 261,
  PEAK_R: 262,
  CPU_LOAD: 263,
  TOTAL_FLOATS: 264,
} as const;

export const PARAMS_PER_TRACK = 8;

export type EngineState = "uninitialized" | "loading" | "suspended" | "running" | "error";

export interface EngineEvents {
  stateChange: (state: EngineState) => void;
  ready: () => void;
  error: (message: string) => void;
}

export class WaveletEngine {
  private _ctx: AudioContext | null = null;
  private _worklet: AudioWorkletNode | null = null;
  private _sab: SharedArrayBuffer | null = null;
  private _paramBlock: Float32Array | null = null;
  private _state: EngineState = "uninitialized";
  private _listeners: Partial<Record<keyof EngineEvents, Function[]>> = {};

  // ── Public getters ──

  get state(): EngineState {
    return this._state;
  }

  get audioContext(): AudioContext | null {
    return this._ctx;
  }

  get paramBlock(): Float32Array | null {
    return this._paramBlock;
  }

  get sampleRate(): number {
    return this._ctx?.sampleRate ?? 48000;
  }

  // ── Lifecycle ──

  /**
   * Initialize the audio engine. Must be called from a user gesture handler.
   * @param wasmUrl URL to the compiled .wasm binary (e.g. "/wavelet_bg.wasm")
   */
  async init(wasmUrl: string): Promise<void> {
    if (this._state === "running" || this._state === "loading") return;

    this._setState("loading");

    try {
      // 1. Create AudioContext
      this._ctx = new AudioContext({ sampleRate: 48000, latencyHint: "interactive" });

      // 2. Allocate SharedArrayBuffer
      if (typeof SharedArrayBuffer === "undefined") {
        throw new Error(
          "SharedArrayBuffer is not available. Ensure COOP/COEP headers are set."
        );
      }
      this._sab = new SharedArrayBuffer(SAB.TOTAL_FLOATS * 4);
      this._paramBlock = new Float32Array(this._sab);

      // Set defaults
      this._paramBlock[SAB.TEMPO] = 120.0;
      this._paramBlock[SAB.MASTER_VOLUME] = 0.8;
      for (let i = 0; i < 16; i++) {
        this._paramBlock[SAB.TRACK_VOLUMES + i] = 0.8;
        this._paramBlock[SAB.TRACK_PANS + i] = 0.0;
        this._paramBlock[SAB.TRACK_MUTES + i] = 0.0;
        this._paramBlock[SAB.TRACK_SOLOS + i] = 0.0;
      }

      // 3. Fetch WASM binary
      const wasmResponse = await fetch(wasmUrl);
      if (!wasmResponse.ok) {
        throw new Error(`Failed to fetch WASM binary: ${wasmResponse.status} ${wasmResponse.statusText}`);
      }
      const wasmBytes = await wasmResponse.arrayBuffer();

      // 4. Register AudioWorklet processor
      await this._ctx.audioWorklet.addModule("/wavelet-processor.js");

      // 5. Create AudioWorkletNode
      this._worklet = new AudioWorkletNode(this._ctx, "wavelet-processor", {
        numberOfInputs: 0,
        numberOfOutputs: 1,
        outputChannelCount: [2],
      });

      // Connect to destination
      this._worklet.connect(this._ctx.destination);

      // Listen for messages from the worklet
      this._worklet.port.onmessage = (e) => this._onWorkletMessage(e.data);

      // 6. Send WASM binary and SAB to the worklet
      this._worklet.port.postMessage(
        {
          type: "init",
          wasmBytes,
          sampleRate: this._ctx.sampleRate,
        },
        [wasmBytes] // Transfer ownership of the ArrayBuffer
      );

      this._worklet.port.postMessage({
        type: "attach-sab",
        sab: this._sab,
      });

      // AudioContext starts suspended — wait for resume
      this._setState("suspended");
    } catch (err) {
      this._setState("error");
      this._emit("error", String(err));
      throw err;
    }
  }

  /**
   * Resume audio playback. Call this from a click/keydown handler.
   */
  async resume(): Promise<void> {
    if (!this._ctx) return;
    await this._ctx.resume();
    this._setState("running");
  }

  /**
   * Suspend audio (pause).
   */
  async suspend(): Promise<void> {
    if (!this._ctx) return;
    await this._ctx.suspend();
    this._setState("suspended");
  }

  /**
   * Tear down the engine completely.
   */
  async destroy(): Promise<void> {
    if (this._worklet) {
      this._worklet.disconnect();
      this._worklet = null;
    }
    if (this._ctx) {
      await this._ctx.close();
      this._ctx = null;
    }
    this._sab = null;
    this._paramBlock = null;
    this._setState("uninitialized");
  }

  // ── Parameter access (zero-copy via SharedArrayBuffer) ──

  setTransportPlaying(playing: boolean): void {
    if (!this._paramBlock) return;
    const current = this._paramBlock[SAB.TRANSPORT_FLAGS];
    if (playing) {
      this._paramBlock[SAB.TRANSPORT_FLAGS] = current | 1;
    } else {
      this._paramBlock[SAB.TRANSPORT_FLAGS] = current & ~1;
    }
  }

  setTransportRecording(recording: boolean): void {
    if (!this._paramBlock) return;
    const current = this._paramBlock[SAB.TRANSPORT_FLAGS];
    if (recording) {
      this._paramBlock[SAB.TRANSPORT_FLAGS] = current | 2;
    } else {
      this._paramBlock[SAB.TRANSPORT_FLAGS] = current & ~2;
    }
  }

  setTempo(bpm: number): void {
    if (!this._paramBlock) return;
    this._paramBlock[SAB.TEMPO] = Math.max(20, Math.min(300, bpm));
  }

  setMasterVolume(vol: number): void {
    if (!this._paramBlock) return;
    this._paramBlock[SAB.MASTER_VOLUME] = Math.max(0, Math.min(1, vol));
  }

  setTrackVolume(track: number, vol: number): void {
    if (!this._paramBlock || track < 0 || track > 15) return;
    this._paramBlock[SAB.TRACK_VOLUMES + track] = Math.max(0, Math.min(1, vol));
  }

  setTrackPan(track: number, pan: number): void {
    if (!this._paramBlock || track < 0 || track > 15) return;
    this._paramBlock[SAB.TRACK_PANS + track] = Math.max(-1, Math.min(1, pan));
  }

  setTrackMute(track: number, muted: boolean): void {
    if (!this._paramBlock || track < 0 || track > 15) return;
    this._paramBlock[SAB.TRACK_MUTES + track] = muted ? 1.0 : 0.0;
  }

  setTrackSolo(track: number, solo: boolean): void {
    if (!this._paramBlock || track < 0 || track > 15) return;
    this._paramBlock[SAB.TRACK_SOLOS + track] = solo ? 1.0 : 0.0;
  }

  /**
   * Set a per-track DSP parameter (encoder A–H maps to param index 0–7).
   */
  setTrackParam(track: number, paramIndex: number, value: number): void {
    if (!this._paramBlock || track < 0 || track > 15) return;
    if (paramIndex < 0 || paramIndex >= PARAMS_PER_TRACK) return;
    const offset = SAB.TRACK_PARAMS + track * PARAMS_PER_TRACK + paramIndex;
    this._paramBlock[offset] = value;
  }

  setActiveTrack(track: number): void {
    if (!this._paramBlock || track < 0 || track > 15) return;
    this._paramBlock[SAB.ACTIVE_TRACK] = track;
  }

  // ── Read-back from engine (written by AudioWorklet) ──

  getCurrentStep(): number {
    return this._paramBlock?.[SAB.CURRENT_STEP] ?? 0;
  }

  getPeakLevels(): [number, number] {
    if (!this._paramBlock) return [0, 0];
    return [this._paramBlock[SAB.PEAK_L], this._paramBlock[SAB.PEAK_R]];
  }

  getCpuLoad(): number {
    return this._paramBlock?.[SAB.CPU_LOAD] ?? 0;
  }

  getWaveformData(): Float32Array {
    if (!this._paramBlock) return new Float32Array(64);
    return this._paramBlock.subarray(SAB.WAVEFORM, SAB.WAVEFORM + 64);
  }

  // ── Commands that go through postMessage (non-realtime) ──

  noteOn(note: number, velocity: number, track: number = 0): void {
    this._worklet?.port.postMessage({
      type: "note-on",
      note,
      velocity,
      track,
    });
  }

  noteOff(note: number, track: number = 0): void {
    this._worklet?.port.postMessage({
      type: "note-off",
      note,
      track,
    });
  }

  loadSample(track: number, data: Float32Array): void {
    this._worklet?.port.postMessage(
      {
        type: "load-sample",
        track,
        data,
      },
      [data.buffer] // Transfer
    );
  }

  clearSample(track: number): void {
    this._worklet?.port.postMessage({
      type: "clear-sample",
      track,
    });
  }

  // ── Events ──

  on<K extends keyof EngineEvents>(event: K, fn: EngineEvents[K]): void {
    if (!this._listeners[event]) this._listeners[event] = [];
    this._listeners[event]!.push(fn as Function);
  }

  off<K extends keyof EngineEvents>(event: K, fn: EngineEvents[K]): void {
    const arr = this._listeners[event];
    if (!arr) return;
    const idx = arr.indexOf(fn as Function);
    if (idx >= 0) arr.splice(idx, 1);
  }

  // ── Private ──

  private _setState(s: EngineState): void {
    this._state = s;
    this._emit("stateChange", s);
  }

  private _emit(event: string, ...args: unknown[]): void {
    const arr = this._listeners[event as keyof EngineEvents];
    if (!arr) return;
    for (const fn of arr) fn(...args);
  }

  private _onWorkletMessage(msg: { type: string; message?: string }): void {
    switch (msg.type) {
      case "ready":
        this._emit("ready");
        break;
      case "error":
        this._emit("error", msg.message ?? "Unknown worklet error");
        break;
    }
  }
}

// Singleton — one engine per app
export const engine = new WaveletEngine();
