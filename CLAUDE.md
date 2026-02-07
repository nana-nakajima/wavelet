# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working on the WAVELET project.

## Project Overview

**WAVELET** is a professional-grade sampling workstation and synthesizer designed for musicians who want classic electronic sound without DAW complexity.

### Target Audience
- Professional musicians who need great sounds quickly
- Workflow-oriented, not menu-diving
- Professional audio quality, easy user experience

### Design Philosophy
1. **Web-first**: Runs in browser, works offline (PWA)
2. **Local-first**: No account required, projects stay on your machine
3. **Professional sound**: Signature Wavelet character - punchy, musical, characterful
4. **Instant workflow**: Preset-first, intuitive controls, minimal setup
5. **AI-ready**: Architecture supports ML integration later

### Reference
Consult `docs/tonverk/Tonverk-User-Manual.md` for signal flow, parameter behavior, and UX patterns. Wavelet takes inspiration but develops its own identity.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      WAVELET ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  WebUI (React)                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Components: EncoderSection, OledDisplay, StepGrid,       │  │
│  │            FxSlotPanel, PianoKeyboard, TransportBar       │  │
│  │  State: Zustand (tonverkStore.ts)                       │  │
│  │  Audio: WebSocketContext → WASM bridge                   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│              ┌───────────────┴───────────────┐                  │
│              ▼                               ▼                  │
│  ┌─────────────────────┐     ┌─────────────────────────────┐   │
│  │ WASM Audio Engine   │     │ IndexedDB (Local Storage)   │   │
│  │ (Rust DSP compiled) │     │ - Projects                  │   │
│  │ - Oscillator        │     │ - Samples                   │   │
│  │ - Filters (ZDF)     │     │ - Presets                   │   │
│  │ - Envelopes (ADSR)  │     │ - User settings             │   │
│  │ - Effects (22)      │     └─────────────────────────────┘   │
│  │ - Sampler           │                                      │
│  │ - Modulation        │                                      │
│  └─────────────────────┘                                      │
│              │                                               │
│              ▼                                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Web Audio API (AudioWorkletProcessor)                   │   │
│  │ - Low latency (< 10ms)                                 │   │
│  │ - Real-time DSP processing                             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Build Commands

### Web UI (webui/)
```bash
cd webui
npm install
npm run dev           # Development server (port 5173)
npm run build         # Production build
```

### Rust Audio Engine (root)
```bash
cargo build --release                    # Native build
wasm-pack build --target web ./wasm     # WASM for web (when wasm/ exists)
cargo test --lib                        # Run all tests (must pass)
cargo clippy                            # Lint
cargo fmt                               # Format
```

---

## MVP Scope (Ship in 4-6 weeks)

### Must-Haves
- [ ] Low-latency Web Audio (under 10ms)
- [ ] 50+ professionally designed presets
- [ ] Solid, intuitive UI (from current Tonverk layout)
- [ ] MIDI input support
- [ ] Project save/load (IndexedDB)
- [ ] WAV audio export

### Nice-to-Haves (Post-MVP)
- AI melody generation
- Cloud preset sharing
- Mobile support
- Collaboration

---

## Track Structure

| Tracks | Type | Purpose |
|--------|------|---------|
| 1-8 | Audio | Sample playback, 8-voice polyphony each |
| 9-12 | Bus | Insert FX + MIDI control |
| 13-15 | Send | Send FX processing |
| 16 | Mix | Master bus |

### Signal Flow (Audio Track)
```
SRC (Sample) → Overdrive → Base-Width Filter → Multimode Filter → AMP (ADSR) → Insert FX (2) → Output
```

---

## Development Priorities (ASAP)

### Phase 1: Audio Foundation (Week 1-2)
1. **WASM compilation** - Rust DSP to WebAssembly
2. **Web Audio bridge** - AudioWorkletProcessor for real-time
3. **Parameter smoothing** - Prevent clicks/pops
4. **Polyphony** - 16 voices with stealing
5. **Sample loading** - IndexedDB + decode

### Phase 2: UI Polish (Week 2-3)
1. **FX parameter editing** - Connect real effect params
2. **MIDI input** - Device selector, automatic reconnection
3. **MIDI Learn** - Map MIDI controls to parameters
4. **Project persistence** - Save/load from IndexedDB
5. **Audio export** - WAV render offline

### Phase 3: Presets & Samples (Week 3-4)
1. **50 professional presets** - Leads, pads, bass, keys, drums, FX, arps
2. **Sample pack** - Starter drums, one-shots
3. **Preset browser UI** - Categories, favorites, search

### Phase 4: Polish & Ship (Week 4-6)
1. **Performance** - Latency, FPS, memory optimization
2. **PWA features** - Service Worker, offline
3. **Documentation** - Quick start guide
4. **Marketing assets** - Screenshots, demo video

---

## Preset Categories (50 Total)

| Category | Count | Description |
|----------|-------|-------------|
| Leads | 10 | Aggressive, synth-y, character |
| Pads | 8 | Atmospheric, evolving, lush |
| Basses | 8 | Fat, gritty, sub-heavy |
| Keys | 6 | Electric, piano, organ |
| Drums | 8 | Kicks, snares, hats (sample-based) |
| FX | 6 | Transitions, textural |
| Arps | 4 | Ready-to-use patterns |

### Wavelet Signature Sound
- **Filters**: Custom ZDF with saturation character
- **Envelopes**: Punchy, fast attack options
- **FX**: Space/reverb with character
- **Overall**: "Modern classic" - reminiscent of Elektron, uniquely Wavelet

---

## Core DSP Modules (Already Implemented)

Located in `src/`:
- `synth.rs` - Main synthesizer engine
- `oscillator.rs` - Waveforms: sine, square, sawtooth, triangle, noise, PM
- `filter.rs` - Biquad + ZDF (Zero-Delay Feedback) Moog-style ladder
- `envelope.rs` - ADSR envelope generators
- `lfo.rs` - Low-frequency oscillators with sync modes
- `sampler.rs` - Sample playback, multi-sampling, recording, auto-sampling
- `effects/` - 22 effect types (reverb, delay, distortion, etc.)
- `tracks/` - AudioTrack, BusTrack, SendTrack, MixTrack

---

## Web UI Components

Located in `webui/src/`:
- `components/EncoderSection.tsx` - Shared 8 rotary encoders
- `components/OledDisplay.tsx` - Central OLED-style display
- `components/StepGrid.tsx` - Global 16-step LED grid
- `components/FxSlotPanel.tsx` - FX slot visualization
- `components/PianoKeyboard.tsx` - 2-octave playable keyboard
- `components/TransportBar.tsx` - Transport controls
- `components/TrackColumn.tsx` - Track strip view
- `components/TrackSelector.tsx` - Track selection sidebar
- `context/WebSocketContext.tsx` - Audio engine connection
- `tonverkStore.ts` - Zustand state management

---

## Current Status

### Working
- Tonverk-aligned UI layout (shared encoders, OLED, step grid)
- WebSocket connection to backend
- 476 passing Rust tests
- DSP modules (oscillator, filter, envelope, LFO, effects)
- Sample playback engine (single + multi)

### In Progress
- WASM compilation target
- Web Audio API integration
- FX parameter binding
- Project persistence

### Missing (Priority)
- Real-time audio output
- MIDI device input
- WAV export
- Preset library
- Sample pack

---

## Testing Requirements

After any Rust DSP changes:
```bash
cargo test --lib        # All tests must pass
cargo clippy            # No warnings
cargo bench             # Verify performance
```

---

## Design Guidelines

### Visual Style
- Dark, professional aesthetic (#0a0a0c background)
- Track-based layout (16 strips on right)
- Shared encoder section (bottom)
- OLED display (center)
- Minimal colors: white text, track-type accents (green/audio, blue/bus, purple/send, yellow/mix)

### UX Principles
1. **Preset-first**: Browse sounds immediately
2. **Fast editing**: Encoders change sound in real-time
3. **No menu diving**: All parameters accessible via pages
4. **Visual feedback**: LEDs, waveforms, meters
5. **Professional polish**: Latency meter, CPU usage, VU

---

## File Structure

```
wavelet/
├── src/                    # Rust DSP core
│   ├── synth.rs
│   ├── oscillator.rs
│   ├── filter.rs
│   ├── envelope.rs
│   ├── lfo.rs
│   ├── sampler.rs
│   ├── effects/
│   ├── tracks/
│   └── wasm/              # WASM bridge (create this)
├── webui/                  # React web interface
│   ├── src/
│   │   ├── components/
│   │   ├── context/
│   │   ├── hooks/
│   │   └── tonverkStore.ts
│   └── package.json
├── backend/                # Actix-web (optional for MVP)
├── docs/tonverk/           # Reference documentation
├── presets/                # Preset definitions (create this)
└── samples/                # Sample pack (create this)
```

---

## Key Technical Decisions

### 1. Audio Transport: Web Audio API
- Native browser support, low latency, cross-platform
- Requires user gesture (acceptable)

### 2. State: Rust ↔ UI
```
WASM Memory → serialize → Float32Array → postMessage → UI
UI → postMessage → deserialize → WASM Memory
```

### 3. Sample Storage: IndexedDB
- lz4 compression for sample blobs
- Streaming decode (not all at once)
- LRU cache for frequently used

### 4. MIDI: Web MIDI API
- Device selector + auto-reconnection
- Per-track channel assignment
- MIDI Learn for custom control

---

## Feature Flags
- `wasm`: Enable WebAssembly compilation (future)
- `midi_cc`: Enable MIDI CC mapping

---

## Resources

- Tonverk Manual: `docs/tonverk/Tonverk-User-Manual.md`
- Web Audio API: https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API
- wasm-bindgen: https://rustwasm.github.io/wasm-bindgen/
