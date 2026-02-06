# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WAVELET is a modular synthesizer combining a Rust audio DSP engine with a Godot 4 UI. It includes a React-based web interface and an Actix-web backend for community features.

## Build Commands

### Rust Audio Engine (root)
```bash
cargo build --release              # Build release library
cargo build --release --features gdext  # Build with Godot bindings
cargo test --lib                   # Run all tests (338 tests)
cargo test <module> --lib          # Test specific module (e.g., modulation, effects, arpeggiator)
cargo clippy                       # Lint
cargo fmt                          # Format
cargo bench                        # Run benchmarks (oscillator_bench, filter_bench, synth_bench)
```

### Backend API (backend/)
```bash
cd backend
cargo build --release
cargo run                          # Runs on port 8080
cargo test
```

### Web UI (webui/)
```bash
cd webui
npm install
npm run dev                        # Development server
npm run build                      # Production build (tsc && vite build)
```

### Godot UI
Open `godot/project.godot` in Godot 4.6+. Copy built library (`target/release/libwavelet.*`) to `godot/` before running.

## Architecture

### Signal Flow
Oscillators (3x) -> Mixer -> Filter (Biquad/ZDF) -> Amplifier (ADSR) -> Effects Chain -> Output

### Core Components (src/)
- **synth.rs**: Main synthesizer with 16-voice polyphony, orchestrates all components
- **oscillator.rs**: Waveform generators (sine, square, sawtooth, triangle)
- **filter.rs**: Biquad and ZDF filters including Moog-style ladder
- **envelope.rs**: ADSR envelope generators
- **lfo.rs**: Low-frequency oscillators for modulation
- **modulation/mod_matrix.rs**: Modulation routing matrix connecting sources to destinations
- **modulation/midi_cc.rs**: MIDI CC mapping (requires `midi_cc` feature)

### Effects System (src/effects/)
15 effects organized by category: reverb, delay, distortion, chorus, compressor, saturation, phaser, flanger, ring_modulator, tremolo, warp (time-stretch), freeze, filter_bank, eq, bit_crusher. Per-track chains managed in track_effects.rs.

### Generators (src/)
- **melody_generator.rs**: AI melody generation (14 scales, 6 styles)
- **chord_generator.rs**: Chord progressions (8 styles)
- **rhythm_generator.rs**: Drum patterns (12 genres)

### Sequencing (src/)
- **step_sequencer.rs**: 16-track step sequencer
- **piano_roll.rs**: Piano roll editor
- **arpeggiator.rs**: Arpeggiator patterns

### Godot Integration
- **gdextension.rs**: GDExtension bindings exposing Rust engine to Godot
- **godot/scripts/main.gd**: Main controller bridging UI to Rust engine
- **godot/presets/wavelet_presets.json**: 50 preset definitions

### Web UI (webui/src/)
React + TypeScript + Zustand state management. VCV Rack-inspired modular interface with Rack.tsx (container), ModulePanel.tsx (modules), Knob.tsx/Port.tsx (controls).

### Backend (backend/src/)
Actix-web REST API with JWT auth. Handlers in handlers/ for presets, follows, projects, challenges. PostgreSQL via sqlx with migrations in migrations/.

## Feature Flags
- `gdext`: Enable Godot GDExtension bindings
- `midi_cc`: Enable MIDI CC mapping
