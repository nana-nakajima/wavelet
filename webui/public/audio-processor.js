// Wavelet Audio Processor - AudioWorklet
//
// This processor runs in the real-time audio thread.
// It handles sample playback, effects processing, and mixing.

class WaveletProcessor extends AudioWorkletProcessor {
  private wasmInstance: any = null;
  private messageQueue: any[] = [];
  private outputBuffer: Float32Array[] = [];
  private bufferSize: number = 128;
  private sampleRate: number = 48000;

  constructor() {
    super();
    this.outputBuffer = [new Float32Array(this.bufferSize), new Float32Array(this.bufferSize)];

    this.port.onmessage = (event) => {
      if (event.data.type === 'wasmReady') {
        this.wasmInstance = event.data.instance;
      } else {
        this.messageQueue.push(event.data);
      }
    };
  }

  process(inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
    const output = outputs[0];

    if (!output || output.length === 0) {
      return true;
    }

    // Handle any pending messages
    while (this.messageQueue.length > 0) {
      const msg = this.messageQueue.shift();
      if (this.wasmInstance) {
        this.wasmInstance.handle_message(JSON.stringify(msg));
      }
    }

    // Get input if available (passthrough mode)
    const input = inputs[0];
    if (input && input.length > 0) {
      for (let channel = 0; channel < Math.min(output.length, input.length); channel++) {
        output[channel].set(input[channel]);
      }
    }

    // If WASM is loaded, process through it
    if (this.wasmInstance) {
      try {
        const left = new Float32Array(this.bufferSize);
        const right = new Float32Array(this.bufferSize);

        // Process through WASM DSP
        this.wasmInstance.process(left, right);

        // Copy to output
        if (output[0]) output[0].set(left);
        if (output[1]) output[1].set(right);
      } catch (e) {
        // Fall back to silence on error
        for (let channel = 0; channel < output.length; channel++) {
          output[channel].fill(0);
        }
      }
    }

    return true;
  }
}

registerProcessor('wavelet-processor', WaveletProcessor);
