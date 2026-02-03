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

### Effects (15 Professional Effects!)
#### Dynamics & Distortion
- **Distortion**: Soft, Hard, Tube, and Fuzz algorithms
- **Compressor**: MSEC dynamics processing
- **Saturation**: Tape-style saturation with variable drive

#### Modulation
- **Chorus**: Stereo expansion and modulation
- **Phaser**: Stereo phaser with feedback
- **Flanger**: Stereo flanger with feedback
- **Tremolo**: Amplitude modulation with multiple waveforms
- **RingModulator**: Ring modulation with dual oscillators

#### Time & Space
- **Reverb**: Schroeder algorithmic room simulation
- **Delay**: Stereo delay with feedback control
- **Freeze**: Sample freeze and buffer manipulation
- **Warp**: Time stretching and pitch shifting (NEW!)

#### Filtering & Shaping
- **FilterBank**: Multi-band filter bank (8 bands)
- **SimpleEQ**: 3-band parametric equalizer
- **BitCrusher**: Sample & Hold with decimation

### AI-Powered Generation (Pro Feature)
- **Smart Melody Generator**: Generate original melodies in 14 scales and 6 styles
- **Intelligent Chord Progressions**: Auto-generate progressions in 8 styles
- **Rhythm Pattern Generator**: Create drum patterns in 12 genres

### Presets
50 professionally designed presets across categories:
- **Bass**: Sub, Reese, Acid, FM Bass
- **Pad**: Ambient, Electric, Synth Pad
- **Lead**: Pluck, Saw Lead, Square Lead
- **Keys**: Electric Piano, Clavi, Organ
- **Strings**: Orchestral, Synth Strings
- **Bell**: FM Bell, Glass Bell, Digital Bell
- **Effect**: Sci-Fi, Impact, Texture

## 🚀 Quick Start

### Prerequisites
- **Rust**: 1.70+ with Cargo
- **Godot**: 4.2+ (4.6 recommended)
- **Platforms**: macOS 12+, Windows 10+, or Linux (Ubuntu 22.04+)

### Installation

#### Option 1: From Source

```bash
# Clone the repository
git clone https://github.com/nana-nakajima/wavelet.git
cd wavelet

# Build the Rust audio engine
cargo build --release

# Copy the library to Godot project
cp target/release/libwavelet.* godot/
```

#### Option 2: Download Pre-built (Coming Soon)
Download from GitHub Releases or Steam (when available).

### Running in Godot

1. **Launch Godot 4.6+**
2. **Import Project**: Click "Import" and select `godot/project.godot`
3. **Run**: Press F5 or click "Run Project"

### First Sound (30 seconds!)

1. Click any **preset button** on the right panel
2. Press keys on your keyboard (Z=lower octaves, Q=upper octaves)
3. Adjust the **ADSR knobs** to shape your sound
4. Try the **AI Generate** button for instant inspiration!

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `Z-M` | Lower octave keys |
| `Q-]` | Upper octave keys |
| `1-8` | Quick preset select |
| `R` | Randomize parameters |

### MIDI Support

WAVELET supports MIDI keyboards:
1. Connect your MIDI device
2. Press any key to activate
3. Use pitch bend and modulation wheel

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

## 📁 File Structure

```
wavelet/
├── Cargo.toml                    # Rust project configuration
├── README.md                     # This file
├── src/                          # Rust audio engine
│   ├── lib.rs                    # Library entry point (12KB)
│   ├── oscillator.rs             # Oscillator module (12KB)
│   ├── filter.rs                 # Filter module (10KB)
│   ├── envelope.rs               # Envelope module (6KB)
│   ├── lfo.rs                    # LFO module (4KB)
│   ├── synth.rs                  # Main synth module (15KB)
│   ├── modulation/               # Modulation system
│   │   ├── mod_matrix.rs         # Modulation matrix (30KB)
│   │   └── lfo.rs                # LFO with modulation
│   ├── effects/                  # Effects modules
│   │   ├── mod.rs                # Effects module
│   │   ├── reverb.rs             # Schroeder reverb (12KB)
│   │   ├── delay.rs              # Stereo delay (8KB)
│   │   ├── distortion.rs         # Distortion (5KB)
│   │   ├── chorus.rs             # Chorus effect (19KB)
│   │   ├── compressor.rs         # MSEC compressor (6KB)
│   │   └── simple_eq.rs          # 3-band EQ (5KB)
│   ├── generators/               # AI generators
│   │   ├── melody_generator.rs   # Smart melody (30KB)
│   │   ├── chord_generator.rs    # Chord progressions (25KB)
│   │   └── rhythm_generator.rs   # Drum patterns (20KB)
│   ├── step_sequencer.rs         # Step sequencer (50KB)
│   ├── arpeggiator.rs            # Arpeggiator (20KB)
│   ├── presets.rs                # Preset system (12KB)
│   └── gdextension.rs            # Godot bindings
├── godot/                        # Godot UI project
│   ├── project.godot             # Project configuration
│   ├── scenes/
│   │   ├── main.tscn             # Main interface
│   │   ├── community_panel.tscn  # Community features
│   │   └── challenge_panel.tscn  # Challenge system
│   ├── scripts/
│   │   ├── main.gd               # Main controller
│   │   ├── http_client.gd        # API client
│   │   ├── community_panel.gd    # Community UI
│   │   └── challenge_panel.gd    # Challenge UI
│   └── presets/
│       └── wavelet_presets.json  # 50 presets
├── backend/                      # Community backend (Actix-web)
│   ├── Cargo.toml                # Backend configuration
│   ├── src/
│   │   ├── main.rs               # Server entry
│   │   ├── handlers/             # API handlers
│   │   ├── models/               # Data models
│   │   └── middleware/           # Auth middleware
│   └── migrations/               # Database migrations
├── PACKAGING.md                   # Cross-platform build guide
├── STEAM_PREPARE.md               # Steam publishing guide
└── MARKETING.md                   # Marketing materials
```

## 🧪 Testing

**338 unit tests passing!** ✅

```bash
# Run all tests
cargo test --lib

# Test specific module
cargo test modulation --lib
cargo test generators --lib
cargo test effects --lib
cargo test arpeggiator --lib
cargo test step_sequencer --lib

# Check code coverage
cargo tarpaulin --out Html
```

### Test Coverage by Module
| Module | Tests | Status |
|--------|-------|--------|
| Core (oscillator, filter, envelope, lfo) | 45 | ✅ |
| Synth | 12 | ✅ |
| Effects (15 effects) | 150+ | ✅ |
| Modulation Matrix | 21 | ✅ |
| Arpeggiator | 6 | ✅ |
| Step Sequencer | 11 | ✅ |
| Piano Roll | 10 | ✅ |
| AI Generators | 97 | ✅ |
| **Total** | **338** | **✅ All Passing** |

## 📦 Building for Release

```bash
# Build Rust library
cargo build --release

# Build for different targets
cargo build --release --target x86_64-apple-darwin    # macOS
cargo build --release --target x86_64-pc-windows-msvc # Windows
cargo build --release --target x86_64-unknown-linux-gnu # Linux

# Package with PyInstaller (see PACKAGING.md)
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## License

MIT License - see LICENSE file for details.

## 👥 Credits

- **Developer**: [Nana Nakajima](https://github.com/nana-nakajima)
- **Audio Engine**: Rust + custom DSP
- **UI Framework**: Godot 4
- **Inspiration**: Classic hardware synthesizers, VCV Rack

## 📞 Support

- **GitHub Issues**: Report bugs or request features
- **Discord**: Join our community server
- **Steam**: Subscribe for updates

---

<div align="center">

**Made with 💕 for the music technology community**

*WAVELET - A small wave, like a musical note.*

🌊 🎹 🎵

</div>

---

<div align="center">

*Made with ❤️ for the music technology community*

**WAVELET** - A small wave, like a musical note.

</div>
