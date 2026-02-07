# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WAVELET is a sampling workstation inspired by the Elektron Tonverk, combining a Rust audio DSP engine with a React web UI and an Actix-web backend. The goal is to replicate the Tonverk's full feature set as a software instrument.

**Reference document:** Always consult `docs/tonverk/Tonverk-User-Manual.md` (with images in `docs/tonverk/images/`) before implementing or modifying any feature. The Tonverk manual is the source of truth for behavior, parameter names, signal flow, and UI layout.

## Development Priorities

1. **Web UI first.** All new features should have their UI built in `webui/` before or alongside Rust core work.
2. **Audio performance testing required.** After any Rust DSP implementation or modification, run `cargo test --lib` and `cargo bench` to verify correctness and performance. Never merge DSP changes without passing tests and benchmarks.
3. **Tonverk parity.** Every feature should match the Tonverk's behavior as documented in the manual. Use the same parameter names, ranges, and signal flow where possible.

## Build Commands

### Rust Audio Engine (root)
```bash
cargo build --release
cargo build --release --features gdext  # With Godot bindings
cargo test --lib                        # Run all tests (must pass before any DSP merge)
cargo test <module> --lib               # Test specific module
cargo clippy                            # Lint
cargo fmt                               # Format
cargo bench                             # Run benchmarks (must run after any DSP change)
```

### Backend API (backend/)
```bash
cd backend
cargo build --release
cargo run                               # Runs on port 8080
cargo test
```

### Web UI (webui/)
```bash
cd webui
npm install
npm run dev                             # Development server
npm run build                           # Production build (tsc && vite build)
```

### Godot UI
Open `godot/project.godot` in Godot 4.6+. Copy built library (`target/release/libwavelet.*`) to `godot/` before running.

## Target Architecture (Tonverk-aligned)

### Track Structure (16 tracks)
- **Tracks 1-8:** Audio tracks, 8-voice polyphony each
- **Tracks 9-12:** Bus tracks (insert FX processing, also usable as MIDI tracks)
- **Tracks 13-15:** Send FX tracks
- **Track 16:** Mix track (master bus)

### Signal Flow (per audio track)
```
Machine (SRC) → Overdrive → Base-width Filter → Multimode Filter → Amp (ADSR) → Insert FX (2 slots) → Routing
```
Filters and overdrive can be reordered. Each track routes to: MIX AB, OUT CD, OUT EF, or BUS 1-4.

### Machine Types (SRC)
- **Single Player:** Mono/stereo sample playback, 8-voice poly. Params: TUNE (±5 oct), PLAY MODE (Fwd/Rev/FwdLoop/RevLoop), LOOP CROSSFADE, SAMPLE SLOT, STRT, END, LSTR, LEND.
- **Multi Player:** Polyphonic multi-sampled instruments. Params: TUNE, VIBR, SPD, FADE.
- **Subtracks:** 8 samples in separate subtracks with independent SRC/FLTR/AMP/MOD per subtrack, plus a shared "supertrack" for FX.
- **MIDI Machine:** External MIDI control on tracks 1-12. Params: CHAN, BANK, SBNK, PROG, PB, AT, MW, BC, 16 assignable CCs.

### Filter System
- **Multimode Filter (FLTR Page 1):** Morphing LP→BP→HP. Params: ATK, DEC, SUS, REL, FREQ, RESO, TYPE, ENV (bipolar depth).
- **Base-width Filter (FLTR Page 2):** HP+LP in series. Params: BASE, WIDTH, DEL, SPRD (stereo), KEY.T (keytrack), RSET.

### Effects

**Insert FX (Tracks 1-8, 2 slots each):**
- Chrono Pitch (granular pitch shifter): TUNE, WIN, FDBK, DEP, HPF, LPF, SPD, MIX
- Comb ± Filter: SPD, DEP, SPH, DTUN, FREQ, FDBK, LPF, MIX
- Compressor: THR, ATK, REL, MUP, RAT (1.50-20.00), SCS, SCF, MIX
- Degrader (lo-fi): BR (16-1 bit), OVER, SRR, DROP, RATE, DEP, FREZ, F.TIM
- Dirtshaper (distortion): DRV, RECT, HPF, LPF, NOIS, N.FRQ, N.RES, MIX
- Filterbank (8-band fixed): Gain A-H (90Hz LP to 4kHz HP)
- Infinite Flanger (barber-pole): SPD, DEP, TUNE, FDBK, LPF
- Low-pass Filter (4-pole 24dB/oct): SPD, DEP, SPH, LAG, FREQ, RESO, SPRD
- Panoramic Chorus: DEP, SPD, HPF, WDTH, MIX
- Phase 98 (4/6-stage phaser): SPD, DEP, SHP, LAG, FREQ, FDBK, STG, MIX
- Saturator Delay: TIME (1/128 to 1 bar), X (ping-pong), WID, FDBK, HPF, LPF, MIX

**Bus FX (Tracks 9-12, 2 slots each):**
All Insert FX plus: Frequency Warper, Supervoid Reverb, Warble.

**Send FX (Tracks 13-15, 1 slot each):**
- Compressor, Daisy Delay, Panoramic Chorus, Rumsklang Reverb, Saturator Delay, Supervoid Reverb.

**Mix FX (Track 16, 1 slot):**
All Bus FX plus: Multimode Filter.

**Key effect details:**
- Daisy Delay: DRV, TIME (division), FDBK, WIDH, MOD, SKEW, FILT
- Rumsklang Reverb: PRE, EARLY, DAMP, SIZE, LOWC, HIGHC
- Supervoid Reverb: PRE, DEC, FREQ, GAIN, HPF, LPF, MIX
- Frequency Warper: SPD, DEP, SPH, LAG, SHFT, SPRD, SBND, MIX
- Warble (tape): SPEED, DEPTH, BASE, WIDTH, N.LEV, N.HPF, STEREO, MIX

### Modulation System
- **Voice LFO 1 & 2:** Per-voice, modulate SRC/FLTR/AMP params.
- **FX LFO 1 & 2:** Modulate insert FX params.
- **Mod Envelope:** ADSR for modulation routing.
- **LFO Params:** SPD, MULT (1-2K), FADE (bipolar), DEST, WAVE (Tri/Sine/Sq/Saw/Random/Exp/Ramp), SPH, MODE (FREE/TRIG/HOLD/ONE/HALF), DEP (bipolar).
- **Mod Destinations:** All SRC params, FLTR (Type/Freq/Reso/Spread/EnvDepth/ATK/DEC/SUS/REL/Base/Width), AMP (ATK/Hold/DEC/SUS/REL/Overdrive/Pan/Volume), FX (16 knobs across 2 slots), Routing (Output/Send 1-3 amounts).

### Sequencer
- **256 steps max** (16 pages × 16 steps per track)
- **Grid recording:** Place trigs on step grid. Trig types: Note (red), Lock (yellow), Combined (blinking red).
- **Live recording:** Real-time with keyboard/pads, quantized or free.
- **Parameter locks:** Lock any parameter to a per-step value.
- **Trig conditions:** FILL/FILL̄, PRE/PRĒ, NEI/NEĪ, 1ST/1ST̄, LST/LST̄, A:B/A:B̄.
- **Micro timing:** Per-step timing offset ahead/behind beat.
- **Retrigs:** RTRG on/off, RTIM (1/128 to 1/1 incl. triplets), RVEL (velocity curve).
- **Page setup:** LENGTH (1-16), SCALE (1/8 to 2×), CHANGE, RESET.

### Arpeggiator
- MODE, SPEED, RANGE (octaves), N.LEN (note length), OFFSET, ARP LENGTH.

### Song Mode
- Up to 16 songs per project, 99 rows per song.
- Per-row: LABEL (Verse/Chorus/Bridge/etc.), PTN, PLAY COUNT, LENGTH (2-1024 steps), TEMPO, END (LOOP/STOP).

### Perform Mode
- Temporary parameter tweaks not saved to pattern. Revert on exit.

### Sampling
- Record up to 6:06:06. Params: REC, ARM, RLEN (1/16 to MAX), THR, SRC (inputs/main/tracks/buses), MON.
- Auto Sampler for multi-sampled instruments: START/END note range, SAMPLE EVERY, VELOCITY LAYERS, NOTE DURATION, LTNCY, RELEASE TIME.
- Formats: WAV/AIFF, 16/24/32-bit, 48kHz native. Up to 4GB in project RAM, 1023 sample slots.

### MIDI
- Sync: Clock/transport receive/send, program change receive/send.
- Port config: Receive notes/CC, input from MIDI/USB/MIDI+USB.
- Channels: AUTO CHANNEL, per-track MIDI channels (1-16).
- MIDI tracks: CHAN, BANK, SBNK, PROG, PB, AT, MW, BC, 16 assignable CCs.

## Current Architecture

### Core Components (src/)
- **synth.rs**: Main synthesizer with 16-voice polyphony
- **oscillator.rs**: Waveform generators (sine, square, sawtooth, triangle)
- **filter.rs**: Biquad and ZDF filters including Moog-style ladder
- **envelope.rs**: ADSR envelope generators
- **lfo.rs**: Low-frequency oscillators for modulation
- **modulation/mod_matrix.rs**: Modulation routing matrix
- **modulation/midi_cc.rs**: MIDI CC mapping (requires `midi_cc` feature)

### Effects System (src/effects/)
15 effects: reverb, delay, distortion, chorus, compressor, saturation, phaser, flanger, ring_modulator, tremolo, warp, freeze, filter_bank, eq, bit_crusher. Per-track chains in track_effects.rs.

### Generators (src/)
- **melody_generator.rs**: AI melody generation (14 scales, 6 styles)
- **chord_generator.rs**: Chord progressions (8 styles)
- **rhythm_generator.rs**: Drum patterns (12 genres)

### Sequencing (src/)
- **step_sequencer.rs**: 16-track step sequencer
- **piano_roll.rs**: Piano roll editor
- **arpeggiator.rs**: Arpeggiator patterns

### Godot Integration
- **gdextension.rs**: GDExtension bindings
- **godot/scripts/main.gd**: Main controller
- **godot/presets/wavelet_presets.json**: 50 preset definitions

### Web UI (webui/src/)
React + TypeScript + Zustand. Elektron Tonverk-inspired interface. Components: Rack.tsx, ModulePanel.tsx, ModuleBrowser.tsx, Knob.tsx, Port.tsx, Header.tsx, Icons.tsx.

### Backend (backend/src/)
Actix-web REST API with JWT auth. PostgreSQL via sqlx.

## Feature Flags
- `gdext`: Enable Godot GDExtension bindings
- `midi_cc`: Enable MIDI CC mapping

## Web UI Design Direction

The web UI should follow the Elektron Tonverk's interface style:
- **Dark, minimal aesthetic.** Black/dark gray backgrounds (#0a0a0c, #0f1014), high-contrast text.
- **Track-based layout.** 16 tracks displayed as selectable columns/buttons, not a modular rack.
- **Parameter pages.** Each track has tabbed pages: TRIG, SRC, FLTR, AMP, FX, MOD — matching the Tonverk's page structure.
- **Rotary encoders as knobs.** 8 knobs per page mapping to 8 parameters, with labels below.
- **Step sequencer grid.** 16 step buttons with LED-style indicators (red=note, yellow=lock, blinking=combined).
- **OLED-style display area.** Central screen showing current page, parameter values, waveform/envelope visualizations.
- **Transport bar.** Play/Stop/Record, BPM, pattern select, song position.
- **Mute/Solo buttons** per track.
- **Accent colors:** Minimal — white/gray text, subtle colored indicators for track state.
