//! WAVELET - Abstract Sound Synthesizer Library
//!
//! This module provides the core audio synthesis engine for WAVELET,
//! a modular synthesizer built with Rust and integrated with Godot 4.
//!
//! # Architecture
//!
//! The synthesizer consists of several interconnected modules:
//! - **Oscillator**: Generates basic waveforms (sine, square, sawtooth, triangle)
//! - **Filter**: Shapes the frequency content of the audio signal
//! - **Envelope**: Controls amplitude and filter modulation over time
//! - **LFO**: Low-frequency oscillator for modulating other parameters
//! - **Effects**: Additional audio processing (reverb, delay, etc.)
//!
//! # Virtual Analog Features (v2.1.0)
//!
//! This version introduces Virtual Analog features:
//! - **ZDF Filter**: Zero-Delay Feedback ladder filter (Moog-style)
//! - **Saturation**: Analog-style soft clipping and harmonic enhancement
//! - **Oversampling**: Anti-aliasing for oscillators
//!
//! # Example
//!
//! ```rust
//! use wavelet::Synth;
//!
//! let mut synth = Synth::new();
//! synth.set_oscillator_type(0, OscillatorType::Sawtooth);
//! synth.note_on(60, 127); // MIDI note 60 (C4) with velocity 127
//! ```

pub mod effects;
pub mod envelope;
pub mod filter;
pub mod gdextension;
pub mod lfo;
pub mod melody_generator;
pub mod oscillator;
pub mod synth;
pub mod chord_generator;

pub use effects::{Effect, EffectType, Saturation};
pub use envelope::{Envelope, EnvelopeStage};
pub use filter::{Filter, FilterType, ZdfFilter, ZdfFilterMode};
pub use lfo::{Lfo, LfoRate};
pub use melody_generator::{Melody, MelodyGenerator, MelodyNote, MelodyStyle};
pub use oscillator::{Oscillator, OscillatorType, OversampleFactor, Waveform};
pub use synth::Synth;
pub use chord_generator::{ChordGenerator, ChordStyle, Key, Scale, Chord, ChordType, ProgressionPattern};

// Re-export commonly used types for convenience
pub use crate::envelope::AdsrEnvelope;
pub use crate::filter::BiquadFilter;

// Virtual Analog parameter IDs (for automation and UI)
pub use synth::{
    PARAM_OVERSAMPLE, PARAM_SATURATION_DRIVE, PARAM_SATURATION_MIX, PARAM_ZDF_CUTOFF,
    PARAM_ZDF_DRIVE, PARAM_ZDF_ENABLED, PARAM_ZDF_RES,
};
