# WAVELET - Abstract Sound Synthesizer

<div align="center">

**A small wave, like a musical note.** 🌊

*WAVELET is a modular synthesizer built with Rust and Godot 4.*

</div>

## About WAVELET

WAVELET is an abstract sound synthesizer that combines the power of Rust for audio processing with the flexibility of Godot 4 for the user interface. The name "WAVELET" evokes the idea of a small, precise waveform - like a musical note itself.

## Features

### Core Synthesis Engine
- **Multiple Oscillators**: Sine, Square, Sawtooth, and Triangle waveforms
- ** polyphony**: Up to 16 voices of polyphony
- **ADSR Envelopes**: Full ADSR envelope generators for amplitude and filter
- **Dual LFOs**: Low-frequency oscillators for modulation
- **Biquad Filters**: Low-pass, High-pass, Band-pass, Notch, and All-pass filters

### Effects
- **Reverb**: Room simulation for depth and space
- **Delay**: Echo effects for rhythmic interest
- **Distortion**: Waveshaping for grit and character

### Presets
20 professionally designed presets across categories:
- Basic, Bass, Pad, Lead
- Keys, Strings, Bell, Effect

## Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- Godot 4.2+
- macOS, Linux, or Windows

### Building

```bash
# Clone the repository
git clone https://github.com/n3kjm/wavelet.git
cd wavelet

# Build the Rust library
cargo build --release

# Copy the library to Godot project
cp target/release/libwavelet.* godot/
```

### Running in Godot

1. Open Godot 4.2+
2. Import the `godot/` folder as a project
3. Run the project (F5)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    WAVELET Synth                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────┐    ┌─────────┐    ┌─────────┐             │
│  │ Osc 1   │    │ Osc 2   │    │ Osc 3   │             │
│  └────┬────┘    └────┬────┘    └────┬────┘             │
│       │              │              │                   │
│       └──────────────┼──────────────┘                   │
│                      ▼                                  │
│              ┌─────────────┐                            │
│              │   Mixer     │                            │
│              └──────┬──────┘                            │
│                     ▼                                   │
│              ┌─────────────┐                            │
│              │   Filter    │                            │
│              └──────┬──────┘                            │
│                     ▼                                   │
│              ┌─────────────┐                            │
│              │  Amplifier  │                            │
│              │ (Envelope)  │                            │
│              └──────┬──────┘                            │
│                     ▼                                   │
│              ┌─────────────┐                            │
│              │  Effects    │                            │
│              └──────┬──────┘                            │
│                     ▼                                   │
│              ┌─────────────┐                            │
│              │   Output    │                            │
│              └─────────────┘                            │
└─────────────────────────────────────────────────────────┘
```

## API Usage

### Rust

```rust
use wavelet::Synth;

let mut synth = Synth::new(48000.0);
synth.note_on(60, 100);  // Play C4
synth.note_off(60);      // Release C4
```

### Godot (GDScript)

```gdscript
extends WaveletSynth

func _ready():
    note_on(60, 127)  # Play C4
    await get_tree().create_timer(0.5).timeout
    note_off(60)      # Release C4
```

## File Structure

```
wavelet/
├── Cargo.toml              # Rust project configuration
├── README.md               # This file
├── src/
│   ├── lib.rs              # Library entry point
│   ├── oscillator.rs       # Oscillator module
│   ├── filter.rs           # Filter module
│   ├── envelope.rs         # Envelope module
│   ├── lfo.rs              # LFO module
│   ├── effects/
│   │   └── mod.rs          # Effects module
│   ├── synth.rs            # Main synth module
│   └── gdextension.rs      # Godot bindings
└── godot/
    ├── project.godot       # Godot 4 project file
    ├── scenes/
    │   └── main.tscn       # Main scene
    ├── scripts/
    │   └── main.gd         # Main script
    └── presets/
        └── 20_presets.json # Preset bank
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## License

MIT License - see LICENSE file for details.

## Credits

- **Developer**: n3kjm
- **Audio Engine**: Rust + custom DSP
- **UI Framework**: Godot 4
- **Inspiration**: Classic hardware synthesizers

---

<div align="center">

*Made with ❤️ for the music technology community*

**WAVELET** - A small wave, like a musical note.

</div>
