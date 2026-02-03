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

pub mod arpeggiator;
pub mod chord_generator;
pub mod effects;
pub mod envelope;
pub mod filter;
pub mod gdextension;
pub mod lfo;
pub use lfo::{Lfo, LfoRate};
pub mod melody_generator;
pub mod modulation;
pub mod oscillator;
pub mod piano_roll;
pub mod presets;
pub mod rhythm_generator;
pub mod sampler;
pub mod step_sequencer;
pub mod synth;

pub use arpeggiator::{ArpConfig, ArpNoteValue, ArpPattern, Arpeggiator};
pub use chord_generator::{
    Chord, ChordGenerator, ChordStyle, ChordType, Key, ProgressionPattern, Scale,
};
pub use effects::{Chorus, Effect, EffectType, Saturation, SimpleEq};
pub use envelope::{Envelope, EnvelopeStage};
pub use filter::{Filter, FilterType, ZdfFilter, ZdfFilterMode};
pub use melody_generator::{Melody, MelodyGenerator, MelodyNote, MelodyStyle};
pub use modulation::{
    ModulationConnection, ModulationConnectionConfig, ModulationMatrix, ModulationMatrixError,
    ModulationSource, ModulationSourceType, ModulationTarget, ModulationTargetType,
    MAX_CONNECTIONS, MAX_SOURCES_PER_TRACK,
};
pub use oscillator::{Oscillator, OscillatorType, OversampleFactor, Waveform};
pub use piano_roll::{EditMode, NoteEvent, PianoRoll, PianoRollConfig, Resolution};
pub use presets::{
    Preset, PresetCategory, PresetCollection, PresetManager, PresetParameters,
};
pub use rhythm_generator::{
    Complexity, DrumNote, DrumPattern, DrumSound, RhythmGenerator, RhythmStyle,
};
pub use sampler::{
    AutoSlicer, LoopInfo, LoopMode, Sample, SampleFormat, SampleInfo, SampleLibrary,
    Sampler, SlicingMode, SlicePoint,
};
pub use step_sequencer::{
    DrumStyle, NUM_STEPS, NUM_TRACKS, ParamLocks, Scale as SeqScale, Step, StepSequencer, Track,
    TrigCondition,
};
pub use synth::Synth;

// Re-export commonly used types for convenience
pub use crate::envelope::AdsrEnvelope;
pub use crate::filter::BiquadFilter;

// Virtual Analog parameter IDs (for automation and UI)
pub use synth::{
    PARAM_OVERSAMPLE, PARAM_SATURATION_DRIVE, PARAM_SATURATION_MIX, PARAM_ZDF_CUTOFF,
    PARAM_ZDF_DRIVE, PARAM_ZDF_ENABLED, PARAM_ZDF_RES,
};
