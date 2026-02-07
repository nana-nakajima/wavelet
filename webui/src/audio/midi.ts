// MIDI Input Handler for Wavelet
//
// Handles MIDI device input, note-on/off, CC messages,
// and MIDI Learn functionality.

export type MidiCallback = (data: MidiMessage) => void;

export interface MidiMessage {
  type: 'noteOn' | 'noteOff' | 'cc' | 'pitchbend' | 'aftertouch' | 'program';
  channel: number;
  note?: number;
  velocity?: number;
  controller?: number;
  value?: number;
}

export interface MidiDevice {
  id: string;
  name: string;
  manufacturer: string;
  state: 'connected' | 'disconnected';
}

export type MidiLearnCallback = (controller: number, channel: number) => void;

export class MidiHandler {
  private midiAccess: MIDIAccess | null = null;
  private inputs: Map<string, MIDIInput> = new Map();
  private outputs: Map<string, MIDIOutput> = new Map();
  private callbacks: Set<MidiCallback> = new Set();
  private midiLearnCallback: MidiLearnCallback | null = null;
  private midiLearnActive: boolean = false;
  private midiLearnController: number | null = null;

  constructor() {}

  async init(): Promise<boolean> {
    if (!navigator.requestMIDIAccess) {
      console.warn('Web MIDI API not supported');
      return false;
    }

    try {
      this.midiAccess = await navigator.requestMIDIAccess({ sysex: false });

      // Enumerate devices
      this.midiAccess.inputs.forEach((input, id) => {
        this.inputs.set(id, input);
        input.onmidimessage = (e) => this.handleMidiMessage(e);
      });

      this.midiAccess.outputs.forEach((output, id) => {
        this.outputs.set(id, output);
      });

      // Listen for connection changes
      this.midiAccess.onstatechange = (e) => this.handleStateChange(e);

      console.log(`MIDI initialized: ${this.inputs.size} inputs, ${this.outputs.size} outputs`);
      return true;
    } catch (e) {
      console.error('Failed to initialize MIDI:', e);
      return false;
    }
  }

  private handleMidiMessage(event: MIDIMessageEvent): void {
    const [status, data1, data2] = event.data!;
    const channel = (status & 0x0f) + 1;
    const messageType = status & 0xf0;

    let message: MidiMessage | null = null;

    switch (messageType) {
      case 0x90: // Note On
        if (data2 > 0) {
          message = {
            type: 'noteOn',
            channel,
            note: data1,
            velocity: data2,
          };
        } else {
          // Note On with velocity 0 = Note Off
          message = {
            type: 'noteOff',
            channel,
            note: data1,
            velocity: 0,
          };
        }
        break;

      case 0x80: // Note Off
        message = {
          type: 'noteOff',
          channel,
          note: data1,
          velocity: data2,
        };
        break;

      case 0xb0: // Control Change
        message = {
          type: 'cc',
          channel,
          controller: data1,
          value: data2,
        };

        // Handle MIDI Learn
        if (this.midiLearnActive && data1 !== 123) {
          this.midiLearnCallback?.(data1, channel);
          this.midiLearnActive = false;
        }
        break;

      case 0xe0: // Pitch Bend
        const pitchBend = ((data2 << 7) | data1) - 8192;
        message = {
          type: 'pitchbend',
          channel,
          value: pitchBend,
        };
        break;

      case 0xd0: // Channel Aftertouch
        message = {
          type: 'aftertouch',
          channel,
          value: data1,
        };
        break;

      case 0xc0: // Program Change
        message = {
          type: 'program',
          channel,
          value: data1 + 1,
        };
        break;
    }

    if (message) {
      this.callbacks.forEach((cb) => cb(message!));
    }
  }

  private handleStateChange(event: MIDIConnectionEvent): void {
    const port = event.port;

    if (port.state === 'connected') {
      if (port.type === 'input') {
        this.inputs.set(port.id, port as MIDIInput);
        (port as MIDIInput).onmidimessage = (e) => this.handleMidiMessage(e);
      } else {
        this.outputs.set(port.id, port as MIDIOutput);
      }
    } else if (port.state === 'disconnected') {
      this.inputs.delete(port.id);
      this.outputs.delete(port.id);
    }

    // Dispatch event for UI
    window.dispatchEvent(new CustomEvent('wavelet:midiStateChange'));
  }

  subscribe(callback: MidiCallback): () => void {
    this.callbacks.add(callback);
    return () => this.callbacks.delete(callback);
  }

  startMidiLearn(callback: MidiLearnCallback): void {
    this.midiLearnActive = true;
    this.midiLearnCallback = callback;
  }

  stopMidiLearn(): void {
    this.midiLearnActive = false;
    this.midiLearnCallback = null;
  }

  isMidiLearnActive(): boolean {
    return this.midiLearnActive;
  }

  getDevices(): MidiDevice[] {
    const devices: MidiDevice[] = [];

    this.inputs.forEach((input, id) => {
      devices.push({
        id,
        name: input.name || 'Unknown',
        manufacturer: input.manufacturer || 'Unknown',
        state: input.state as 'connected' | 'disconnected',
      });
    });

    this.outputs.forEach((output, id) => {
      if (!this.inputs.has(id)) {
        devices.push({
          id,
          name: output.name || 'Unknown',
          manufacturer: output.manufacturer || 'Unknown',
          state: output.state as 'connected' | 'disconnected',
        });
      }
    });

    return devices;
  }

  isInputConnected(): boolean {
    return this.inputs.size > 0;
  }

  dispose(): void {
    this.callbacks.clear();
    this.midiLearnCallback = null;
    this.midiLearnActive = false;

    this.inputs.forEach((input) => {
      input.onmidimessage = null;
    });
    this.inputs.clear();
    this.outputs.clear();
  }
}

// Standard MIDI CC mappings for Wavelet
export const STANDARD_CC: Record<number, string> = {
  1: 'MODULATION',
  2: 'BREATH',
  7: 'VOLUME',
  10: 'PAN',
  11: 'EXPRESSION',
  64: 'SUSTAIN',
  65: 'PORTAMENTO',
  71: 'RESONANCE',
  74: 'CUTOFF',
  91: 'REVERB',
  93: 'CHORUS',
};

// Export singleton
export const midiHandler = new MidiHandler();
