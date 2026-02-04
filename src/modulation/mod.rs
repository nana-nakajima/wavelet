//! Modulation Module
//!
//! This module provides modulation routing capabilities for the synthesizer.
//! It includes the modulation matrix, sources, targets, and connections.
//!
//! # Architecture
//!
//! - `ModulationMatrix` - Main component managing all connections
//! - `ModulationSource` - Source of modulation (LFO, Envelope, etc.)
//! - `ModulationTarget` - Target parameter to modulate
//! - `ModulationConnection` - Connection from source to target
//!
//! # Usage
//!
//! ```rust
//! use wavelet::modulation::{ModulationMatrix, ModulationConnection};
//!
//! let mut matrix = ModulationMatrix::new(0);
//!
//! // Add a connection from LFO to Filter Cutoff
//! let conn = ModulationConnection::default_lfo_filter();
//! matrix.add_connection(conn).unwrap();
//! ```

pub mod mod_matrix;

pub use mod_matrix::{
    ModulationMatrix,
    ModulationConnection,
    ModulationSource,
    ModulationTarget,
    ModulationSourceType,
    ModulationTargetType,
    ModulationConnectionConfig,
    ModulationMatrixError,
    MAX_CONNECTIONS,
    MAX_SOURCES_PER_TRACK,
};

#[cfg(feature = "midi_cc")]
pub mod midi_cc;

#[cfg(feature = "midi_cc")]
pub use midi_cc::{
    MidiCCManager,
    MidiCCError,
    AssignableCC,
    CCParameterTarget,
    StandardCC,
    cc_to_cutoff,
    cc_to_resonance,
    cc_to_time,
    cc_to_pitch,
    MAX_CC_COUNT,
};
