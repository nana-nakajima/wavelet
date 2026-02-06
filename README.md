# WAVELET - Abstract Sound Synthesizer

A modular synthesizer built with Rust and Godot 4.

## Features

### Core Synthesis Engine
- **Multiple Oscillators**: Sine, Square, Sawtooth, Triangle waveforms with oversampling
- **16-Voice Polyphony**
- **ADSR Envelopes**: Amplitude and filter envelope generators
- **Dual LFOs**: Low-frequency oscillators for modulation
- **Filters**: Biquad (LP, HP, BP, Notch, Allpass) and ZDF Moog-style ladder

### Effects (15 Professional Effects)
- **Dynamics**: Distortion, Compressor, Saturation
- **Modulation**: Chorus, Phaser, Flanger, Tremolo, Ring Modulator
- **Time/Space**: Reverb, Delay, Freeze, Warp (time-stretch)
- **Filtering**: Filter Bank, 3-band EQ, Bit Crusher

### AI-Powered Generation
- **Melody Generator**: 14 scales, 6 styles
- **Chord Progressions**: 8 styles
- **Rhythm Patterns**: 12 genres

### Presets
50 professionally designed presets: Bass, Pad, Lead, Keys, Strings, Bell, Effect

## Quick Start

### Prerequisites
- Rust 1.70+
- Godot 4.6+
- Windows 10+, macOS 12+, or Linux (Ubuntu 22.04+)

### Build

```bash
# Build the Rust audio engine
cargo build --release

# Copy library to Godot project
cp target/release/libwavelet.* godot/
```

### Run in Godot
1. Open `godot/project.godot` in Godot 4.6+
2. Press F5 to run

### Keyboard
| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `Z-M` | Lower octave |
| `Q-]` | Upper octave |
| `1-8` | Quick preset select |

## Architecture

```
Oscillators (3x) -> Mixer -> Filter -> Amplifier (ADSR) -> Effects -> Output
```

## API Usage

### Rust
```rust
use wavelet::Synth;

let mut synth = Synth::new(48000.0);
synth.note_on(60, 100);  // Play C4
synth.note_off(60);
```

### GDScript
```gdscript
extends WaveletSynth

func _ready():
    note_on(60, 127)
    await get_tree().create_timer(0.5).timeout
    note_off(60)
```

## Testing

```bash
cargo test --lib                    # Run all tests
cargo test <module> --lib           # Test specific module
```

## Project Structure

```
wavelet/
├── src/                  # Rust audio engine
│   ├── synth.rs          # Main synthesizer
│   ├── oscillator.rs     # Waveform generators
│   ├── filter.rs         # Biquad & ZDF filters
│   ├── effects/          # 15 effects
│   └── modulation/       # Mod matrix, MIDI CC
├── godot/                # Godot 4 UI
├── webui/                # React web interface
└── backend/              # Actix-web REST API
```

## License

MIT License
