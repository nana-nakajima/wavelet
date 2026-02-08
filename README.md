# WAVELET — Sampling Workstation & Synthesizer

A professional-grade sampling workstation and synthesizer that runs in the browser. Built with a Rust DSP engine compiled to WebAssembly and a React-based hardware-emulation UI.

## Features

### Core Synthesis Engine (Rust)
- **Multiple Oscillators**: Sine, Square, Sawtooth, Triangle, Noise, PM with oversampling
- **16-Voice Polyphony** with voice stealing
- **ADSR Envelopes**: Amplitude and filter envelope generators
- **Dual LFOs**: Low-frequency oscillators with sync modes
- **Filters**: Biquad (LP, HP, BP, Notch, Allpass) and ZDF Moog-style ladder
- **Sampler**: Sample playback, multi-sampling, recording, auto-sampling

### Effects (22 Types)
- **Dynamics**: Distortion, Compressor, Saturation
- **Modulation**: Chorus, Phaser, Flanger, Tremolo, Ring Modulator
- **Time/Space**: Reverb, Delay, Freeze, Warp (time-stretch)
- **Filtering**: Filter Bank, 3-band EQ, Bit Crusher

### Web UI
- **Hardware-emulation interface** inspired by Elektron workflow
- **16-track architecture**: 8 Audio, 4 Bus, 3 Send, 1 Mix
- **OLED-style display** with dot-matrix pixel font (Canvas 2D)
- **8 rotary encoders** with page-based parameter mapping
- **16-step sequencer** with P-Lock parameter locks
- **2-octave piano keyboard** with QWERTY input
- **XY planar pad**, FX slot panel, transport controls
- **Real-time engine sync** via SharedArrayBuffer (zero-copy)

## Quick Start

### Prerequisites
- Rust 1.70+
- Node.js 18+

### Web UI (Development)

```bash
cd webui
npm install
npm run dev       # http://localhost:5173
```

Add `?dev` to bypass engine init for UI-only testing: `http://localhost:5173/?dev`

### Rust Engine

```bash
cargo build --release    # Native build
cargo test --lib         # Run tests (476 passing)
cargo clippy             # Lint
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  WebUI (Remix + React + Zustand)                            │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Components: Workstation, OledScreen, EncoderSection,   │ │
│  │   StepGrid, PianoKeyboard, TrackStrips, TransportBar   │ │
│  │ State: Zustand store ↔ SharedArrayBuffer (zero-copy)   │ │
│  └────────────────────────────────────────────────────────┘ │
│                          │                                   │
│              ┌───────────┴───────────┐                      │
│              ▼                       ▼                      │
│  ┌──────────────────┐   ┌─────────────────────┐            │
│  │ WASM Audio Engine │   │ IndexedDB Storage   │            │
│  │ (Rust DSP)        │   │ Projects, Samples   │            │
│  └──────────────────┘   └─────────────────────┘            │
│              │                                               │
│              ▼                                               │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Web Audio API (AudioWorkletProcessor, <10ms latency)    ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
wavelet/
├── src/                    # Rust DSP core
│   ├── synth.rs            # Main synthesizer engine
│   ├── oscillator.rs       # Waveform generators
│   ├── filter.rs           # Biquad & ZDF filters
│   ├── envelope.rs         # ADSR envelopes
│   ├── lfo.rs              # Low-frequency oscillators
│   ├── sampler.rs          # Sample playback engine
│   ├── effects/            # 22 effect types
│   ├── tracks/             # Audio/Bus/Send/Mix tracks
│   └── wasm/               # WASM bridge + SAB layout
├── webui/                  # React web interface (Remix SPA)
│   ├── app/components/     # 17 UI components
│   ├── app/engine/         # WASM + AudioWorklet bridge
│   ├── app/store/          # Zustand state management
│   ├── app/hooks/          # Encoder drag, long-press, keyboard
│   └── app/lib/            # OLED renderer, formatting, colors
├── docs/tonverk/           # Reference documentation
└── CLAUDE.md               # AI assistant instructions
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play/Stop |
| `Shift` (hold) | FUNC modifier (fine encoder, lock trigs) |
| `A-P` | Piano keys (C3–D#4, QWERTY layout) |

## Testing

```bash
cargo test --lib         # All Rust tests
cd webui && npx tsc --noEmit   # TypeScript check
cd webui && npm run build      # Production build
```

## License

MIT License
