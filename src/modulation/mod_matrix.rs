//! Modulation Matrix Module
//!
//! This module provides a flexible modulation routing system for the synthesizer.
//! It allows users to connect modulation sources to modulation targets with
//! configurable depth and polarity.
//!
//! # Architecture
//!
//! - `ModulationSource`: Source of modulation (LFO, Envelope, etc.)
//! - `ModulationTarget`: Target parameter to modulate
//! - `ModulationConnection`: A connection from source to target
//! - `ModulationMatrix`: Collection of all modulation connections

use std::collections::HashMap;
use std::fmt;

/// Maximum number of modulation connections
pub const MAX_CONNECTIONS: usize = 32;

/// Maximum number of modulation sources per track
pub const MAX_SOURCES_PER_TRACK: usize = 8;

/// Enumeration of modulation source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModulationSourceType {
    /// LFO (Low Frequency Oscillator)
    LFO,

    /// Envelope generator
    Envelope,

    /// Velocity
    Velocity,

    /// Aftertouch
    Aftertouch,

    /// Pitch bend wheel
    PitchBend,

    /// Modulation wheel
    ModWheel,

    /// Step sequencer output
    StepSequencer,

    /// Audio rate oscillator (for FM)
    AudioOscillator,

    /// Per-track effects output
    TrackFX,

    /// Master clock
    MasterClock,

    /// Random/Noise source
    Random,

    /// MIDI Control Change
    MIDI,
}

/// Display implementation for modulation source types
impl fmt::Display for ModulationSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModulationSourceType::LFO => write!(f, "LFO"),
            ModulationSourceType::Envelope => write!(f, "Envelope"),
            ModulationSourceType::Velocity => write!(f, "Velocity"),
            ModulationSourceType::Aftertouch => write!(f, "Aftertouch"),
            ModulationSourceType::PitchBend => write!(f, "Pitch Bend"),
            ModulationSourceType::ModWheel => write!(f, "Mod Wheel"),
            ModulationSourceType::StepSequencer => write!(f, "Step Sequencer"),
            ModulationSourceType::AudioOscillator => write!(f, "Audio Osc"),
            ModulationSourceType::TrackFX => write!(f, "Track FX"),
            ModulationSourceType::MasterClock => write!(f, "Master Clock"),
            ModulationSourceType::Random => write!(f, "Random"),
            ModulationSourceType::MIDI => write!(f, "MIDI"),
        }
    }
}

/// Enumeration of modulation target types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModulationTargetType {
    /// Oscillator frequency
    OscillatorFrequency,

    /// Oscillator pulse width
    OscillatorPulseWidth,

    /// Oscillator amplitude
    OscillatorAmplitude,

    /// Filter cutoff frequency
    FilterCutoff,

    /// Filter resonance
    FilterResonance,

    /// Filter drive
    FilterDrive,

    /// LFO rate
    LFORate,

    /// LFO depth
    LFODepth,

    /// Effect mix
    EffectMix,

    /// Effect parameter
    EffectParameter,

    /// Pan/Width
    Pan,

    /// Volume
    Volume,

    /// Pitch (coarse)
    PitchCoarse,

    /// Pitch (fine)
    PitchFine,

    /// Custom parameter
    Custom(u8),
}

/// Display implementation for modulation target types
impl fmt::Display for ModulationTargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModulationTargetType::OscillatorFrequency => write!(f, "OSC Frequency"),
            ModulationTargetType::OscillatorPulseWidth => write!(f, "OSC Pulse Width"),
            ModulationTargetType::OscillatorAmplitude => write!(f, "OSC Amplitude"),
            ModulationTargetType::FilterCutoff => write!(f, "Filter Cutoff"),
            ModulationTargetType::FilterResonance => write!(f, "Filter Resonance"),
            ModulationTargetType::FilterDrive => write!(f, "Filter Drive"),
            ModulationTargetType::LFORate => write!(f, "LFO Rate"),
            ModulationTargetType::LFODepth => write!(f, "LFO Depth"),
            ModulationTargetType::EffectMix => write!(f, "Effect Mix"),
            ModulationTargetType::EffectParameter => write!(f, "Effect Parameter"),
            ModulationTargetType::Pan => write!(f, "Pan"),
            ModulationTargetType::Volume => write!(f, "Volume"),
            ModulationTargetType::PitchCoarse => write!(f, "Pitch Coarse"),
            ModulationTargetType::PitchFine => write!(f, "Pitch Fine"),
            ModulationTargetType::Custom(n) => write!(f, "Custom {}", n),
        }
    }
}

/// Modulation source with current value
#[derive(Debug, Clone)]
pub struct ModulationSource {
    /// Source type
    source_type: ModulationSourceType,

    /// Source ID (for multiple LFOs, Envelopes, etc.)
    source_id: u8,

    /// Current modulation value (-1.0 to 1.0 typically)
    current_value: f32,

    /// Whether the source is active
    active: bool,
}

impl ModulationSource {
    /// Creates a new modulation source
    pub fn new(source_type: ModulationSourceType, source_id: u8) -> Self {
        Self {
            source_type,
            source_id,
            current_value: 0.0,
            active: true,
        }
    }

    /// Creates an LFO source
    pub fn lfo(id: u8) -> Self {
        Self::new(ModulationSourceType::LFO, id)
    }

    /// Creates an envelope source
    pub fn envelope(id: u8) -> Self {
        Self::new(ModulationSourceType::Envelope, id)
    }

    /// Gets the source type
    pub fn source_type(&self) -> ModulationSourceType {
        self.source_type
    }

    /// Gets the source ID
    pub fn source_id(&self) -> u8 {
        self.source_id
    }

    /// Gets the current value
    pub fn current_value(&self) -> f32 {
        self.current_value
    }

    /// Sets the current value
    pub fn set_current_value(&mut self, value: f32) {
        self.current_value = value.clamp(-10.0, 10.0); // Allow some headroom
    }

    /// Checks if the source is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Enables or disables the source
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

/// Modulation target specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModulationTarget {
    /// Target type
    target_type: ModulationTargetType,

    /// Target ID (e.g., oscillator index, filter index)
    target_id: u8,

    /// Parameter index for complex targets
    param_index: u8,
}

impl ModulationTarget {
    /// Creates a new modulation target
    pub fn new(target_type: ModulationTargetType, target_id: u8, param_index: u8) -> Self {
        Self {
            target_type,
            target_id,
            param_index,
        }
    }

    /// Creates an oscillator frequency target
    pub fn oscillator_frequency(oscillator_id: u8) -> Self {
        Self::new(ModulationTargetType::OscillatorFrequency, oscillator_id, 0)
    }

    /// Creates a filter cutoff target
    pub fn filter_cutoff(filter_id: u8) -> Self {
        Self::new(ModulationTargetType::FilterCutoff, filter_id, 0)
    }

    /// Gets the target type
    pub fn target_type(&self) -> ModulationTargetType {
        self.target_type
    }

    /// Gets the target ID
    pub fn target_id(&self) -> u8 {
        self.target_id
    }

    /// Gets the parameter index
    pub fn param_index(&self) -> u8 {
        self.param_index
    }
}

/// Configuration for a modulation connection
#[derive(Debug, Clone, PartialEq)]
pub struct ModulationConnectionConfig {
    /// Source specification
    pub source: ModulationSourceType,
    pub source_id: u8,

    /// Target specification
    pub target: ModulationTargetType,
    pub target_id: u8,
    pub target_param: u8,

    /// Modulation depth (-1.0 to 1.0, or more for some targets)
    pub depth: f32,

    /// Whether this connection is enabled
    pub enabled: bool,

    /// Bipolar flag (true = -depth to +depth, false = 0 to +depth)
    pub bipolar: bool,
}

impl Default for ModulationConnectionConfig {
    fn default() -> Self {
        Self {
            source: ModulationSourceType::LFO,
            source_id: 0,
            target: ModulationTargetType::FilterCutoff,
            target_id: 0,
            target_param: 0,
            depth: 0.5,
            enabled: true,
            bipolar: true,
        }
    }
}

/// A single modulation connection from source to target
#[derive(Debug, Clone)]
pub struct ModulationConnection {
    /// Connection configuration
    config: ModulationConnectionConfig,

    /// Current modulation value from this connection
    current_value: f32,
}

impl ModulationConnection {
    /// Creates a new modulation connection from config
    pub fn from_config(config: ModulationConnectionConfig) -> Self {
        Self {
            config,
            current_value: 0.0,
        }
    }

    /// Creates a default LFO -> Filter Cutoff connection
    pub fn default_lfo_filter() -> Self {
        Self::from_config(ModulationConnectionConfig {
            source: ModulationSourceType::LFO,
            source_id: 0,
            target: ModulationTargetType::FilterCutoff,
            target_id: 0,
            target_param: 0,
            depth: 0.5,
            enabled: true,
            bipolar: true,
        })
    }

    /// Gets the configuration
    pub fn config(&self) -> &ModulationConnectionConfig {
        &self.config
    }

    /// Gets the source type
    pub fn source_type(&self) -> ModulationSourceType {
        self.config.source
    }

    /// Gets the source ID
    pub fn source_id(&self) -> u8 {
        self.config.source_id
    }

    /// Gets the target type
    pub fn target_type(&self) -> ModulationTargetType {
        self.config.target
    }

    /// Gets the target ID
    pub fn target_id(&self) -> u8 {
        self.config.target_id
    }

    /// Gets the modulation depth
    pub fn depth(&self) -> f32 {
        self.config.depth
    }

    /// Sets the modulation depth
    pub fn set_depth(&mut self, depth: f32) {
        self.config.depth = depth.clamp(-2.0, 2.0);
    }

    /// Checks if the connection is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enables or disables the connection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Checks if bipolar
    pub fn is_bipolar(&self) -> bool {
        self.config.bipolar
    }

    /// Sets bipolar mode
    pub fn set_bipolar(&mut self, bipolar: bool) {
        self.config.bipolar = bipolar;
    }

    /// Gets the current modulation value
    pub fn current_value(&self) -> f32 {
        self.current_value
    }

    /// Updates the current modulation value from a source
    pub fn update_from_source(&mut self, source_value: f32) {
        if !self.config.enabled {
            self.current_value = 0.0;
            return;
        }

        if self.config.bipolar {
            // Bipolar: output ranges from -depth to +depth
            self.current_value = source_value * self.config.depth;
        } else {
            // Unipolar: output ranges from 0 to +depth
            self.current_value = (source_value + 1.0) * 0.5 * self.config.depth;
        }
    }

    /// Converts to config for serialization
    pub fn to_config(&self) -> ModulationConnectionConfig {
        self.config.clone()
    }
}

/// The modulation matrix - manages all modulation connections
#[derive(Debug, Clone)]
pub struct ModulationMatrix {
    /// All modulation connections
    connections: Vec<ModulationConnection>,

    /// Map of source (type, id) to connections
    source_map: HashMap<(ModulationSourceType, u8), Vec<usize>>,

    /// Map of target (type, id) to connections
    target_map: HashMap<(ModulationTargetType, u8), Vec<usize>>,

    /// Track ID this matrix belongs to
    track_id: u8,

    /// Maximum connections
    max_connections: usize,

    /// Whether this matrix is enabled
    enabled: bool,
}

impl ModulationMatrix {
    /// Creates a new modulation matrix for a track
    pub fn new(track_id: u8) -> Self {
        Self {
            connections: Vec::with_capacity(MAX_CONNECTIONS),
            source_map: HashMap::new(),
            target_map: HashMap::new(),
            track_id,
            max_connections: MAX_CONNECTIONS,
            enabled: true,
        }
    }

    /// Creates from configurations
    pub fn from_configs(track_id: u8, configs: &[ModulationConnectionConfig]) -> Self {
        let mut matrix = Self::new(track_id);

        for config in configs {
            let _ = matrix.add_connection_from_config(config.clone());
        }

        matrix
    }

    /// Gets the track ID
    pub fn track_id(&self) -> u8 {
        self.track_id
    }

    /// Checks if the matrix is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables or disables the matrix
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Adds a modulation connection
    pub fn add_connection(
        &mut self,
        connection: ModulationConnection,
    ) -> Result<usize, ModulationMatrixError> {
        if self.connections.len() >= self.max_connections {
            return Err(ModulationMatrixError::MaxConnectionsReached);
        }

        let index = self.connections.len();
        self.connections.push(connection);

        // Update maps
        let conn = &self.connections[index];
        let source_key = (conn.source_type(), conn.source_id());
        let target_key = (conn.target_type(), conn.target_id());

        self.source_map.entry(source_key).or_default().push(index);
        self.target_map.entry(target_key).or_default().push(index);

        Ok(index)
    }

    /// Adds a connection from config
    pub fn add_connection_from_config(
        &mut self,
        config: ModulationConnectionConfig,
    ) -> Result<usize, ModulationMatrixError> {
        let connection = ModulationConnection::from_config(config);
        self.add_connection(connection)
    }

    /// Removes a connection by index
    pub fn remove_connection(&mut self, index: usize) -> Result<(), ModulationMatrixError> {
        if index >= self.connections.len() {
            return Err(ModulationMatrixError::InvalidIndex);
        }

        let conn = &self.connections[index];
        let source_key = (conn.source_type(), conn.source_id());
        let target_key = (conn.target_type(), conn.target_id());

        // Remove from maps
        if let Some(vec) = self.source_map.get_mut(&source_key) {
            vec.retain(|&i| i != index);
        }
        if let Some(vec) = self.target_map.get_mut(&target_key) {
            vec.retain(|&i| i != index);
        }

        // Remove from connections
        self.connections.remove(index);

        Ok(())
    }

    /// Gets a connection by index
    pub fn connection(&self, index: usize) -> Option<&ModulationConnection> {
        self.connections.get(index)
    }

    /// Gets a mutable connection by index
    pub fn connection_mut(&mut self, index: usize) -> Option<&mut ModulationConnection> {
        self.connections.get_mut(index)
    }

    /// Gets the total number of connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Gets all connections
    pub fn connections(&self) -> &[ModulationConnection] {
        &self.connections
    }

    /// Gets connections for a specific source
    pub fn connections_for_source(
        &self,
        source_type: ModulationSourceType,
        source_id: u8,
    ) -> Vec<&ModulationConnection> {
        let key = (source_type, source_id);
        self.source_map
            .get(&key)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.connections.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets connections for a specific target
    pub fn connections_for_target(
        &self,
        target_type: ModulationTargetType,
        target_id: u8,
    ) -> Vec<&ModulationConnection> {
        let key = (target_type, target_id);
        self.target_map
            .get(&key)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.connections.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Updates all connections from a source value
    pub fn update_from_source(
        &mut self,
        source_type: ModulationSourceType,
        source_id: u8,
        value: f32,
    ) {
        if !self.enabled {
            return;
        }

        let key = (source_type, source_id);
        if let Some(indices) = self.source_map.get(&key) {
            for &index in indices {
                if let Some(conn) = self.connections.get_mut(index) {
                    conn.update_from_source(value);
                }
            }
        }
    }

    /// Gets the total modulation for a specific target
    pub fn total_modulation_for_target(
        &self,
        target_type: ModulationTargetType,
        target_id: u8,
    ) -> f32 {
        let key = (target_type, target_id);
        self.target_map
            .get(&key)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.connections.get(i))
                    .filter(|c| c.is_enabled())
                    .map(|c| c.current_value())
                    .sum()
            })
            .unwrap_or(0.0)
    }

    /// Processes modulation for a single sample
    pub fn process(
        &mut self,
        source_values: &HashMap<(ModulationSourceType, u8), f32>,
    ) -> HashMap<(ModulationTargetType, u8), f32> {
        if !self.enabled {
            return HashMap::new();
        }

        let mut output = HashMap::new();

        for conn in &mut self.connections {
            if !conn.is_enabled() {
                continue;
            }

            let source_key = (conn.source_type(), conn.source_id());
            if let Some(&source_value) = source_values.get(&source_key) {
                conn.update_from_source(source_value);

                let target_key = (conn.target_type(), conn.target_id());
                let current = output.get(&target_key).copied().unwrap_or(0.0);
                output.insert(target_key, current + conn.current_value());
            }
        }

        output
    }

    /// Resets all connections
    pub fn reset(&mut self) {
        for conn in &mut self.connections {
            conn.current_value = 0.0;
        }
    }

    /// Converts to configs for serialization
    pub fn to_configs(&self) -> Vec<ModulationConnectionConfig> {
        self.connections.iter().map(|c| c.to_config()).collect()
    }
}

/// Errors for modulation matrix operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModulationMatrixError {
    /// Maximum connections reached
    MaxConnectionsReached,

    /// Invalid connection index
    InvalidIndex,

    /// Source not found
    SourceNotFound,

    /// Target not found
    TargetNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulation_source_creation() {
        let source = ModulationSource::lfo(0);
        assert_eq!(source.source_type(), ModulationSourceType::LFO);
        assert_eq!(source.source_id(), 0);
        assert!(source.is_active());
        assert_eq!(source.current_value(), 0.0);
    }

    #[test]
    fn test_modulation_source_value() {
        let mut source = ModulationSource::envelope(1);

        source.set_current_value(0.75);
        assert_eq!(source.current_value(), 0.75);

        // Value should be clamped
        source.set_current_value(15.0);
        assert_eq!(source.current_value(), 10.0);
    }

    #[test]
    fn test_modulation_target() {
        let target = ModulationTarget::filter_cutoff(0);
        assert_eq!(target.target_type(), ModulationTargetType::FilterCutoff);
        assert_eq!(target.target_id(), 0);
    }

    #[test]
    fn test_modulation_connection_creation() {
        let conn = ModulationConnection::default_lfo_filter();

        assert_eq!(conn.source_type(), ModulationSourceType::LFO);
        assert_eq!(conn.target_type(), ModulationTargetType::FilterCutoff);
        assert_eq!(conn.depth(), 0.5);
        assert!(conn.is_enabled());
        assert!(conn.is_bipolar());
    }

    #[test]
    fn test_modulation_connection_update_bipolar() {
        let mut conn = ModulationConnection::default_lfo_filter();
        conn.set_bipolar(true);

        // Positive input
        conn.update_from_source(1.0);
        assert_eq!(conn.current_value(), 0.5); // 1.0 * 0.5

        // Negative input
        conn.update_from_source(-0.5);
        assert_eq!(conn.current_value(), -0.25); // -0.5 * 0.5
    }

    #[test]
    fn test_modulation_connection_update_unipolar() {
        let mut conn = ModulationConnection::default_lfo_filter();
        conn.set_bipolar(false);

        // Positive input
        conn.update_from_source(1.0);
        assert_eq!(conn.current_value(), 0.5); // (1.0 + 1.0) * 0.5 * 0.5 = 0.5

        // Negative input (should become positive)
        conn.update_from_source(-1.0);
        assert_eq!(conn.current_value(), 0.0); // (-1.0 + 1.0) * 0.5 * 0.5 = 0.0
    }

    #[test]
    fn test_modulation_connection_disabled() {
        let mut conn = ModulationConnection::default_lfo_filter();
        conn.set_enabled(false);

        conn.update_from_source(1.0);
        assert_eq!(conn.current_value(), 0.0);
    }

    #[test]
    fn test_modulation_connection_depth() {
        let mut conn = ModulationConnection::default_lfo_filter();

        conn.set_depth(1.0);
        assert_eq!(conn.depth(), 1.0);

        // Depth should be clamped
        conn.set_depth(3.0);
        assert_eq!(conn.depth(), 2.0);
    }

    #[test]
    fn test_modulation_matrix_creation() {
        let matrix = ModulationMatrix::new(0);

        assert_eq!(matrix.track_id(), 0);
        assert!(matrix.is_enabled());
        assert_eq!(matrix.connection_count(), 0);
    }

    #[test]
    fn test_modulation_matrix_add_connection() {
        let mut matrix = ModulationMatrix::new(0);

        let conn = ModulationConnection::default_lfo_filter();
        let result = matrix.add_connection(conn);

        assert!(result.is_ok());
        assert_eq!(matrix.connection_count(), 1);
    }

    #[test]
    fn test_modulation_matrix_max_connections() {
        let mut matrix = ModulationMatrix::new(0);

        // Add maximum connections
        for _ in 0..MAX_CONNECTIONS {
            let conn = ModulationConnection::default_lfo_filter();
            assert!(matrix.add_connection(conn).is_ok());
        }

        // One more should fail
        let conn = ModulationConnection::default_lfo_filter();
        assert_eq!(
            matrix.add_connection(conn),
            Err(ModulationMatrixError::MaxConnectionsReached)
        );
    }

    #[test]
    fn test_modulation_matrix_remove_connection() {
        let mut matrix = ModulationMatrix::new(0);

        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();
        assert_eq!(matrix.connection_count(), 1);

        let result = matrix.remove_connection(0);
        assert!(result.is_ok());
        assert_eq!(matrix.connection_count(), 0);

        // Remove non-existent
        assert_eq!(
            matrix.remove_connection(0),
            Err(ModulationMatrixError::InvalidIndex)
        );
    }

    #[test]
    fn test_modulation_matrix_connections_for_source() {
        let mut matrix = ModulationMatrix::new(0);

        // Add LFO -> Filter
        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();

        // Add LFO -> Volume
        let mut config = ModulationConnectionConfig::default();
        config.target = ModulationTargetType::Volume;
        config.target_id = 0;
        matrix.add_connection_from_config(config).unwrap();

        let connections = matrix.connections_for_source(ModulationSourceType::LFO, 0);
        assert_eq!(connections.len(), 2);
    }

    #[test]
    fn test_modulation_matrix_connections_for_target() {
        let mut matrix = ModulationMatrix::new(0);

        // Add LFO -> Filter
        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();

        // Add Envelope -> Filter
        let mut config = ModulationConnectionConfig::default();
        config.source = ModulationSourceType::Envelope;
        config.source_id = 0;
        matrix.add_connection_from_config(config).unwrap();

        let connections = matrix.connections_for_target(ModulationTargetType::FilterCutoff, 0);
        assert_eq!(connections.len(), 2);
    }

    #[test]
    fn test_modulation_matrix_update_from_source() {
        let mut matrix = ModulationMatrix::new(0);

        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();

        // Update LFO value
        matrix.update_from_source(ModulationSourceType::LFO, 0, 0.5);

        // Check connection value
        let conn = matrix.connection(0).unwrap();
        assert_eq!(conn.current_value(), 0.25); // 0.5 * 0.5
    }

    #[test]
    fn test_modulation_matrix_total_modulation() {
        let mut matrix = ModulationMatrix::new(0);

        // Add LFO -> Filter
        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();

        // Add Envelope -> Filter
        let mut config = ModulationConnectionConfig::default();
        config.source = ModulationSourceType::Envelope;
        config.source_id = 0;
        matrix.add_connection_from_config(config).unwrap();

        // Update both sources
        matrix.update_from_source(ModulationSourceType::LFO, 0, 0.5);
        matrix.update_from_source(ModulationSourceType::Envelope, 0, 0.3);

        let total = matrix.total_modulation_for_target(ModulationTargetType::FilterCutoff, 0);
        assert!((total - 0.4).abs() < 0.001); // 0.25 + 0.15
    }

    #[test]
    fn test_modulation_matrix_process() {
        let mut matrix = ModulationMatrix::new(0);

        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();

        let mut source_values = HashMap::new();
        source_values.insert((ModulationSourceType::LFO, 0), 0.8);

        let output = matrix.process(&source_values);

        let expected_value = 0.8 * 0.5; // value * depth
        assert_eq!(
            output.get(&(ModulationTargetType::FilterCutoff, 0)),
            Some(&expected_value)
        );
    }

    #[test]
    fn test_modulation_matrix_disabled() {
        let mut matrix = ModulationMatrix::new(0);
        matrix.set_enabled(false);

        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();

        let mut source_values = HashMap::new();
        source_values.insert((ModulationSourceType::LFO, 0), 0.8);

        let output = matrix.process(&source_values);

        assert!(output.is_empty());
    }

    #[test]
    fn test_modulation_matrix_reset() {
        let mut matrix = ModulationMatrix::new(0);

        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();
        matrix.update_from_source(ModulationSourceType::LFO, 0, 0.8);

        matrix.reset();

        let conn = matrix.connection(0).unwrap();
        assert_eq!(conn.current_value(), 0.0);
    }

    #[test]
    fn test_modulation_matrix_to_configs() {
        let mut matrix = ModulationMatrix::new(0);

        matrix
            .add_connection(ModulationConnection::default_lfo_filter())
            .unwrap();

        let configs = matrix.to_configs();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].target, ModulationTargetType::FilterCutoff);
    }

    #[test]
    fn test_modulation_matrix_from_configs() {
        let mut configs = Vec::new();

        let mut config = ModulationConnectionConfig::default();
        config.target = ModulationTargetType::FilterCutoff;
        configs.push(config);

        let mut config2 = ModulationConnectionConfig::default();
        config2.target = ModulationTargetType::Volume;
        configs.push(config2);

        let matrix = ModulationMatrix::from_configs(0, &configs);

        assert_eq!(matrix.connection_count(), 2);
    }
}
