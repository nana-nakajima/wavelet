# WAVELET — Sampling Workstation & Synthesizer

A professional-grade sampling workstation and synthesizer that runs in the browser. Built with a Rust DSP engine compiled to WebAssembly, an Actix-web backend for presets and community features, and a React-based hardware-emulation UI.

## Features

### Core Synthesis Engine (Rust → WASM)
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

### Backend API (Rust / Actix-web)
- **User authentication** (JWT)
- **Preset management** (CRUD, community sharing)
- **Project sharing** and social features (follows, challenges)
- **WebSocket** audio state sync
- **Local file storage** for preset data

### Web UI
- **Hardware-emulation interface** inspired by Elektron workflow
- **16-track architecture**: 8 Audio, 4 Bus, 3 Send, 1 Mix
- **OLED-style display** with dot-matrix pixel font (Canvas 2D)
- **8 rotary encoders** with page-based parameter mapping
- **16-step sequencer** with P-Lock parameter locks
- **2-octave piano keyboard** with QWERTY input
- **XY planar pad**, FX slot panel, transport controls
- **Real-time engine sync** via SharedArrayBuffer (zero-copy)

## Prerequisites

- **Rust** 1.70+
- **Node.js** 18+
- **PostgreSQL** 15+ (for the backend API)

## Quick Start

### 1. Start the Backend API

The backend is an Actix-web server that handles user accounts, presets, and community features. It requires a PostgreSQL database.

```bash
# Set up the database (default: postgresql://wavelet:wavelet@localhost:5432/wavelet)
export DATABASE_URL="postgresql://wavelet:wavelet@localhost:5432/wavelet"

# Build and run the backend
cd backend
cargo run              # Starts on http://127.0.0.1:8080
```

Verify it's running:

```bash
curl http://127.0.0.1:8080/health
# → {"status":"healthy","service":"wavelet-backend","version":"0.2.0"}
```

### 2. Start the Web UI

The Web UI is a Remix SPA (client-only) that runs the WASM audio engine directly in the browser via AudioWorklet + SharedArrayBuffer.

```bash
cd webui
npm install
npm run dev            # Starts on http://localhost:5173
```

Open `http://localhost:5173` in Chrome (SharedArrayBuffer requires COOP/COEP headers, configured automatically by Vite). Click the screen to initialize the audio engine.

**UI-only mode**: Add `?dev` to the URL to bypass engine initialization and test the interface without a WASM binary: `http://localhost:5173/?dev`

### 3. Build the WASM Engine

To compile the Rust DSP engine to WebAssembly:

```bash
# From the project root
cargo build --release                    # Native build (for tests)
wasm-pack build --target web ./          # WASM build (requires wasm-pack)
```

The WASM binary (`wavelet_bg.wasm`) is loaded by the Web UI's AudioWorklet processor at runtime.

### 4. Run Tests

```bash
cargo test --lib                         # Rust DSP tests (476 passing)
cargo clippy                             # Lint
cd webui && npx tsc --noEmit             # TypeScript type check
cd webui && npm run build                # Production build
```

## Making a Drum Beat

Here's how to program a basic 4-on-the-floor drum pattern using the step sequencer.

### Select a kick drum track

Click **T1** in the track selector on the left sidebar. This is your first audio track — use it for the kick drum. The track name highlights and the OLED screen shows the track's current parameters.

### Program the kick pattern

The **step grid** is the row of 16 buttons at the bottom of the center panel. Each button represents one 16th-note step. Click steps **1, 5, 9, 13** to place kick triggers on every quarter note. Active steps light up teal.

```
Step:  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16
Kick:  ●  ·  ·  ·  ●  ·  ·  ·  ●  ·  ·  ·  ●  ·  ·  ·
```

### Shape the kick sound

Click the **SRC** page button to open the source parameters. The 8 encoders now control the oscillator. Drag the **TUNE** encoder down for a low pitch, and adjust **DECAY** for a short punchy hit. Switch to the **AMP** page and set a fast **ATTACK** and short **DECAY** for a tight envelope.

### Add a snare on track 2

Click **T2** in the track selector. Program steps **5** and **13** for a backbeat snare:

```
Step:  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16
Snare: ·  ·  ·  ·  ●  ·  ·  ·  ·  ·  ·  ·  ●  ·  ·  ·
```

Use the **SRC** page to dial in a higher pitch with some **NOISE** mixed in for the snare character.

### Add hi-hats on track 3

Click **T3**. Program closed hats on every 8th note (steps **1, 3, 5, 7, 9, 11, 13, 15**) and open hats on the off-beats by using P-Locks:

```
Step:  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16
Hats:  ●  ·  ●  ·  ●  ·  ●  ·  ●  ·  ●  ·  ●  ·  ●  ·
```

Set a high **TUNE**, zero **DECAY** on the SRC page, and short **DECAY** on the AMP page for a tight closed hat sound.

### Use P-Locks for variation

Long-press a step button to enter **P-Lock edit mode** (the step glows orange). While holding, turn any encoder to set a per-step parameter value. For example, long-press step 3 on the hi-hat track and increase the **DECAY** encoder to make that hat ring longer as an open hat. Release the step button to exit P-Lock mode.

### Set the tempo

The **tempo encoder** is in the transport bar at the top. Drag it to set your BPM (default is 120). Hold **FUNC** (or the Shift key) while dragging for fine adjustment.

### Hit play

Press the **▶ PLAY** button in the transport bar (or press `Space` on your keyboard). The step grid playhead sweeps across the 16 steps and you'll hear your pattern loop. Press **■ STOP** (or `Space` again) to stop.

### Add effects

Select a drum track and click the **FX** page button. The FX slot panel shows two insert effect slots. Use the encoders to adjust effect parameters — try adding some distortion to the kick or reverb to the snare via the **REVERB_SEND** encoder on the AMP page.

### Tips

- **FUNC + encoder drag** = fine mode (4× resolution) for precise tuning
- **FUNC + step click** = add a lock trig (parameter-only, no note trigger)
- Use tracks **T1–T8** for individual drum sounds, one per track
- Route drums to a **BUS** track (BUS 1–4) for group processing
- Use **SEND** tracks for shared reverb/delay effects

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
├── backend/                # Actix-web API server
│   └── src/
│       ├── main.rs         # Server entry (port 8080)
│       ├── handlers/       # HTTP route handlers
│       ├── services/       # Business logic (users, presets)
│       ├── db/             # PostgreSQL repositories
│       ├── audio_engine.rs # Backend audio state
│       └── websocket.rs    # WebSocket audio sync
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

## License

MIT License
