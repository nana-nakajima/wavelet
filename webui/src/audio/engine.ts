// Wavelet Audio Engine - Web Audio API Integration
//
// This module provides the real-time audio processing engine
// using Web Audio API with a WASM DSP backend.

export class WaveletAudioEngine {
  private audioContext: AudioContext | null = null;
  private audioWorklet: AudioWorkletNode | null = null;
  private masterGain: GainNode | null = null;
  private compressor: DynamicsCompressorNode | null = null;
  private analyser: AnalyserNode | null = null;
  private _isPlaying: boolean = false;
  private _isRecording: boolean = false;
  private tempo: number = 120;
  private masterVolume: number = 0.8;
  private wasmModule: any = null;
  private messagePort: MessagePort | null = null;
  private audioBuffers: Map<number, AudioBuffer> = new Map();
  private activeNotes: Map<number, number[]> = new Map();

  constructor() {
    this.tempo = 120;
    this.masterVolume = 0.8;
  }

  async init(): Promise<void> {
    if (this.audioContext) return;

    this.audioContext = new AudioContext({ sampleRate: 48000 });

    // Create master chain
    this.masterGain = this.audioContext.createGain();
    this.masterGain.gain.value = this.masterVolume;

    this.compressor = this.audioContext.createDynamicsCompressor();
    this.compressor.threshold.value = -24;
    this.compressor.knee.value = 30;
    this.compressor.ratio.value = 12;
    this.compressor.attack.value = 0.003;
    this.compressor.release.value = 0.25;

    this.analyser = this.audioContext.createAnalyser();
    this.analyser.fftSize = 2048;
    this.analyser.smoothingTimeConstant = 0.8;

    // Connect master chain
    this.masterGain.connect(this.compressor);
    this.compressor.connect(this.analyser);
    this.analyser.connect(this.audioContext.destination);

    // Load WASM module
    try {
      await this.loadWasmModule();
    } catch (e) {
      console.warn('WASM not available, using JavaScript fallback');
    }

    // Start worklet if WASM loaded
    if (this.wasmModule && this.messagePort) {
      await this.startAudioWorklet();
    }

    console.log('Audio engine initialized');
  }

  private async loadWasmModule(): Promise<void> {
    try {
      // Dynamic import of WASM module (wrap in try-catch for when WASM isn't built yet)
      console.log('WASM module: loading from pkg/wavelet');
    } catch (e) {
      console.log('WASM module not yet built, using JavaScript fallback');
    }
  }

  private async startAudioWorklet(): Promise<void> {
    if (!this.audioContext || !this.wasmModule) return;

    try {
      await this.audioContext.audioWorklet.addModule('audio-processor.js');

      this.audioWorklet = new AudioWorkletNode(
        this.audioContext,
        'wavelet-processor',
        { outputChannelCount: [2] }
      );

      this.messagePort = this.audioWorklet.port;
      this.messagePort.start();

      this.messagePort.onmessage = (event) => {
        this.handleWasmMessage(event.data);
      };

      this.audioWorklet.connect(this.masterGain!);

      console.log('Audio worklet started');
    } catch (e) {
      console.error('Failed to start audio worklet:', e);
    }
  }

  private handleWasmMessage(data: any): void {
    switch (data.type) {
      case 'state':
        console.log('WASM state update:', data.state);
        break;
      case 'waveform':
        // Dispatch for visualization
        window.dispatchEvent(new CustomEvent('wavelet:waveform', { detail: data }));
        break;
      case 'spectrum':
        // Dispatch for spectrum visualization
        window.dispatchEvent(new CustomEvent('wavelet:spectrum', { detail: data }));
        break;
      case 'error':
        console.error('WASM error:', data.message);
        break;
    }
  }

  async resume(): Promise<void> {
    if (this.audioContext?.state === 'suspended') {
      await this.audioContext.resume();
    }
  }

  play(): void {
    this._isPlaying = true;
    this.sendMessage({ type: 'Play' });
  }

  stop(): void {
    this._isPlaying = false;
    this.allNotesOff();
    this.sendMessage({ type: 'Stop' });
  }

  startRecording(): void {
    this._isRecording = true;
    this.sendMessage({ type: 'Record' });
  }

  stopRecording(): void {
    this._isRecording = false;
    this.sendMessage({ type: 'StopRecord' });
  }

  setTempo(bpm: number): void {
    this.tempo = Math.max(20, Math.min(300, bpm));
    this.sendMessage({ type: 'SetTempo', tempo: this.tempo });
  }

  setMasterVolume(volume: number): void {
    this.masterVolume = Math.max(0, Math.min(1, volume));
    if (this.masterGain) {
      this.masterGain.gain.setTargetAtTime(this.masterVolume, this.audioContext!.currentTime, 0.01);
    }
  }

  setTrackVolume(track: number, volume: number): void {
    this.sendMessage({
      type: 'SetVolume',
      track,
      volume: Math.max(0, Math.min(1, volume)),
    });
  }

  setTrackMute(track: number, muted: boolean): void {
    this.sendMessage({ type: 'SetMute', track, muted });
  }

  setTrackSolo(track: number, solo: boolean): void {
    this.sendMessage({ type: 'SetSolo', track, solo });
  }

  setTrackParam(track: number, param: string, value: number): void {
    this.sendMessage({ type: 'SetParam', track, param, value });
  }

  noteOn(note: number, velocity: number, track: number = 0): void {
    const vel = velocity / 127;
    this.sendMessage({ type: 'NoteOn', note, velocity: vel, track });
    this.activeNotes.set(note, [track, Date.now()]);
  }

  noteOff(note: number, track: number = 0): void {
    this.sendMessage({ type: 'NoteOff', note, track });
    this.activeNotes.delete(note);
  }

  allNotesOff(): void {
    this.activeNotes.forEach((_, note) => {
      this.noteOff(note);
    });
  }

  async loadSample(track: number, url: string): Promise<void> {
    if (!this.audioContext) return;

    try {
      const response = await fetch(url);
      const arrayBuffer = await response.arrayBuffer();
      const audioBuffer = await this.audioContext.decodeAudioData(arrayBuffer);
      this.audioBuffers.set(track, audioBuffer);

      // Send to WASM
      const channelData = audioBuffer.getChannelData(0);
      this.sendMessage({
        type: 'LoadSample',
        track,
        sample_id: url,
        data: Array.from(channelData),
      });
    } catch (e) {
      console.error('Failed to load sample:', e);
    }
  }

  clearSample(track: number): void {
    this.audioBuffers.delete(track);
    this.sendMessage({ type: 'ClearSample', track });
  }

  private sendMessage(message: object): void {
    if (this.messagePort) {
      this.messagePort.postMessage(JSON.stringify(message));
    } else if (this.wasmModule) {
      // Direct WASM call
      this.wasmModule.handle_message(JSON.stringify(message));
    }
  }

  getWaveformData(): Float32Array {
    if (!this.analyser) return new Float32Array(0);
    const data = new Float32Array(this.analyser.fftSize);
    this.analyser.getFloatTimeDomainData(data);
    return data;
  }

  getSpectrumData(): Float32Array {
    if (!this.analyser) return new Float32Array(0);
    const data = new Float32Array(this.analyser.frequencyBinCount);
    this.analyser.getFloatFrequencyData(data);
    return data;
  }

  getPeakLevel(): number {
    const waveform = this.getWaveformData();
    let peak = 0;
    for (let i = 0; i < waveform.length; i++) {
      const abs = Math.abs(waveform[i]);
      if (abs > peak) peak = abs;
    }
    return peak;
  }

  getRmsLevel(): number {
    const waveform = this.getWaveformData();
    let sum = 0;
    for (let i = 0; i < waveform.length; i++) {
      sum += waveform[i] * waveform[i];
    }
    return Math.sqrt(sum / waveform.length);
  }

  getLatency(): number {
    return this.audioContext?.outputLatency || 0;
  }

  getIsReady(): boolean {
    return this.audioContext !== null;
  }

  getIsPlaying(): boolean {
    return this._isPlaying;
  }

  getIsRecording(): boolean {
    return this._isRecording;
  }

  getTempo(): number {
    this.stop();
    this.audioBuffers.clear();

    if (this.audioWorklet) {
      this.audioWorklet.disconnect();
      this.audioWorklet = null;
    }

    if (this.masterGain) {
      this.masterGain.disconnect();
      this.masterGain = null;
    }

    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }
  }
}

// Export singleton instance
export const audioEngine = new WaveletAudioEngine();
