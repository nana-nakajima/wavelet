# Wavelet Web UI

Hardware-emulation workstation interface built with Remix (SPA mode), React, Zustand, and Canvas 2D.

## Setup

```bash
npm install
npm run dev          # Development server at http://localhost:5173
npm run build        # Production build
npx tsc --noEmit     # Type check
```

## Dev Mode

Add `?dev` to the URL to bypass engine initialization and test the UI without a WASM binary:

```
http://localhost:5173/?dev
```

## Architecture

- **Remix SPA** — Vite-based, no SSR, client-only rendering
- **Zustand store** — 16-track state with SharedArrayBuffer sync
- **AudioWorklet** — Real-time DSP via WASM in a dedicated thread
- **SharedArrayBuffer** — Zero-copy parameter exchange (264 floats)
- **Canvas 2D** — OLED screen with 3x5 pixel font at 4x scale
- **COOP/COEP headers** — Required for SharedArrayBuffer support

## Components

| Component | Description |
|-----------|-------------|
| `Workstation` | Top-level shell, engine init, layout grid |
| `TransportBar` | Play/Stop/Rec, tempo encoder, step position |
| `OledScreen` | Canvas 2D dot-matrix display, page-specific content |
| `EncoderSection` | 8 rotary knobs mapped to active page params |
| `Encoder` | SVG knob with vertical drag interaction |
| `PageButtons` | TRIG/SRC/FLTR/AMP/FX/MOD page selector |
| `FuncKey` | Modifier button (fine mode, lock trigs) |
| `StepGrid` | 16 LED buttons with click/long-press P-Lock |
| `PianoKeyboard` | 2-octave chromatic keyboard (C3-B4) |
| `TrackSelector` | Left sidebar with AUDIO/BUS/SEND/MIX groups |
| `TrackStrips` | Right panel with 16 track strips |
| `TrackStrip` | Single strip: badge, VU, volume, mute/solo |
| `FxSlotPanel` | 2 insert FX slots with param bars |
| `PlanarPad` | XY pad for macro control |
| `MasterSection` | Master volume encoder + stereo peak meter |
| `StatusBar` | Engine state, CPU load, L/R peak meters |

## Hooks

| Hook | Description |
|------|-------------|
| `use-encoder-drag` | Vertical pointer drag to 0-1 value, fine mode |
| `use-long-press` | 300ms hold detection for P-Lock editing |
| `use-keyboard-shortcuts` | QWERTY piano, Space play/stop, Shift FUNC |

## Key Interactions

### P-Lock Workflow
1. Long-press a step button to enter P-Lock edit mode
2. Turn encoders to set per-step parameter values
3. Release step button to exit P-Lock mode

### FUNC Modifier
- Hold FUNC + drag encoder = fine mode (4x resolution)
- Hold FUNC + click step = add lock trig instead of note trig
