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

pub mod oscillator;
pub mod filter;
pub mod envelope;
pub mod lfo;
pub mod effects;
pub mod synth;
pub mod gdextension;

pub use oscillator::{Oscillator, OscillatorType, Waveform, OversampleFactor};
pub use filter::{Filter, FilterType, ZdfFilter, ZdfFilterMode};
pub use envelope::{Envelope, EnvelopeStage};
pub use lfo::{Lfo, LfoRate};
pub use effects::{Effect, EffectType, Saturation};
pub use synth::Synth;

// Re-export commonly used types for convenience
pub use crate::oscillator::Oscillator;
pub use crate::filter::BiquadFilter;
pub use crate::envelope::AdsrEnvelope;

// Virtual Analog parameter IDs (for automation and UI)
pub use synth::{
    PARAM_ZDF_ENABLED,
    PARAM_ZDF_CUTOFF,
    PARAM_ZDF_RES,
    PARAM_ZDF_DRIVE,
    PARAM_SATURATION_DRIVE,
    PARAM_SATURATION_MIX,
    PARAM_OVERSAMPLE,
};
