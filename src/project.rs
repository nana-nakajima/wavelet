//!
//! WAVELET - Project Save/Load Module
//!
//! This module provides project serialization and deserialization
//! for complete project save/restore functionality.
//!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Maximum project name length
pub const MAX_PROJECT_NAME_LENGTH: usize = 256;

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,
    /// Project description
    pub description: String,
    /// BPM (beats per minute)
    pub bpm: u32,
    /// Time signature numerator
    pub time_signature_numerator: u8,
    /// Time signature denominator
    pub time_signature_denominator: u8,
    /// Created timestamp
    pub created_at: u64,
    /// Modified timestamp
    pub modified_at: u64,
    /// Project version
    pub version: String,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        let now = std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_secs();
        Self {
            name: String::from("Untitled Project"),
            description: String::new(),
            bpm: 120,
            time_signature_numerator: 4,
            time_signature_denominator: 4,
            created_at: now,
            modified_at: now,
            version: String::from("1.0.0"),
        }
    }
}

/// Global project settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalSettings {
    /// Master volume (0.0 to 1.0)
    pub master_volume: f64,
    /// Master pan (0.0 = center)
    pub master_pan: f64,
    /// Swing amount (0.0 to 1.0)
    pub swing: f64,
    /// Quantize value
    pub quantize: u8,
    /// MIDI channel
    pub midi_channel: u8,
    /// Output sample rate
    pub output_sample_rate: u32,
    /// Output buffer size
    pub output_buffer_size: u32,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            master_pan: 0.0,
            swing: 0.0,
            quantize: 4,
            midi_channel: 0,
            output_sample_rate: 44100,
            output_buffer_size: 256,
        }
    }
}

/// Oscillator state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OscillatorState {
    /// Oscillator type/index
    pub osc_index: u8,
    /// Waveform type
    pub waveform: String,
    /// Frequency (Hz or MIDI note)
    pub frequency: f64,
    /// Detune (cents)
    pub detune: f64,
    /// Pulse width (for PWM)
    pub pulse_width: f64,
    /// Oscillator volume
    pub volume: f64,
    /// Oscillator pan
    pub pan: f64,
    /// Oscillator mute state
    pub muted: bool,
    /// Oscillator sync state
    pub sync: bool,
    /// Ring modulation input
    pub ring_mod_input: Option<u8>,
}

impl Default for OscillatorState {
    fn default() -> Self {
        Self {
            osc_index: 0,
            waveform: String::from("sine"),
            frequency: 440.0,
            detune: 0.0,
            pulse_width: 0.5,
            volume: 0.7,
            pan: 0.0,
            muted: false,
            sync: false,
            ring_mod_input: None,
        }
    }
}

/// Filter state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterState {
    /// Filter index
    pub filter_index: u8,
    /// Filter type
    pub filter_type: String,
    /// Cutoff frequency (Hz)
    pub cutoff: f64,
    /// Resonance/Q value
    pub resonance: f64,
    /// Filter drive
    pub drive: f64,
    /// Filter mix (for parallel filters)
    pub mix: f64,
    /// Filter enabled
    pub enabled: bool,
    /// Filter chain mode
    pub chain_mode: String,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            filter_index: 0,
            filter_type: String::from("lowpass"),
            cutoff: 2000.0,
            resonance: 1.0,
            drive: 0.0,
            mix: 1.0,
            enabled: true,
            chain_mode: String::from("series"),
        }
    }
}

/// Envelope state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvelopeState {
    /// Envelope name
    pub name: String,
    /// Attack time (seconds)
    pub attack: f64,
    /// Decay time (seconds)
    pub decay: f64,
    /// Sustain level (0.0 to 1.0)
    pub sustain: f64,
    /// Release time (seconds)
    pub release: f64,
    /// Attack curve
    pub attack_curve: String,
    /// Decay curve
    pub decay_curve: String,
    /// Release curve
    pub release_curve: String,
    /// Velocity influence
    pub velocity_influence: f64,
}

impl Default for EnvelopeState {
    fn default() -> Self {
        Self {
            name: String::from("Default"),
            attack: 0.01,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
            attack_curve: String::from("linear"),
            decay_curve: String::from("linear"),
            release_curve: String::from("linear"),
            velocity_influence: 1.0,
        }
    }
}

/// LFO state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LfoState {
    /// LFO index
    pub lfo_index: u8,
    /// Waveform type
    pub waveform: String,
    /// Frequency (Hz)
    pub frequency: f64,
    /// Amplitude
    pub amplitude: f64,
    /// Phase offset
    pub phase: f64,
    /// Sync to tempo
    pub sync: bool,
    /// Sync rate
    pub sync_rate: String,
    /// Delay time
    pub delay: f64,
    /// Fade in
    pub fade_in: f64,
}

impl Default for LfoState {
    fn default() -> Self {
        Self {
            lfo_index: 0,
            waveform: String::from("sine"),
            frequency: 4.0,
            amplitude: 0.5,
            phase: 0.0,
            sync: false,
            sync_rate: String::from("1/4"),
            delay: 0.0,
            fade_in: 0.0,
        }
    }
}

/// Effect slot state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectSlotState {
    /// Effect type
    pub effect_type: String,
    /// Effect enabled
    pub enabled: bool,
    /// Effect parameters
    pub parameters: HashMap<String, f64>,
    /// Mix level
    pub mix: f64,
    /// Preset name
    pub preset: Option<String>,
}

impl Default for EffectSlotState {
    fn default() -> Self {
        Self {
            effect_type: String::new(),
            enabled: false,
            parameters: HashMap::new(),
            mix: 0.5,
            preset: None,
        }
    }
}

/// Insert effect chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InsertEffectChain {
    /// Effect slots
    pub slots: Vec<EffectSlotState>,
    /// Bypass state
    pub bypass: bool,
}

impl Default for InsertEffectChain {
    fn default() -> Self {
        Self {
            slots: Vec::with_capacity(8),
            bypass: false,
        }
    }
}

/// Track state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackState {
    /// Track index
    pub index: u8,
    /// Track name
    pub name: String,
    /// Track color
    pub color: String,
    /// Track type
    pub track_type: String,
    /// Oscillators
    pub oscillators: Vec<OscillatorState>,
    /// Filters
    pub filters: Vec<FilterState>,
    /// Envelopes
    pub envelopes: Vec<EnvelopeState>,
    /// LFOs
    pub lfos: Vec<LfoState>,
    /// Insert effect chain
    pub insert_effects: InsertEffectChain,
    /// Track volume
    pub volume: f64,
    /// Track pan
    pub pan: f64,
    /// Track mute
    pub mute: bool,
    /// Track solo
    pub solo: bool,
    /// Output routing
    pub output_routing: String,
    /// Send A level
    pub send_a_level: f64,
    /// Send B level
    pub send_b_level: f64,
    /// Send C level
    pub send_c_level: f64,
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::from("Track 1"),
            color: String::from("#FFFFFF"),
            track_type: String::from("synth"),
            oscillators: Vec::with_capacity(4),
            filters: Vec::with_capacity(2),
            envelopes: Vec::with_capacity(4),
            lfos: Vec::with_capacity(4),
            insert_effects: InsertEffectChain::default(),
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            output_routing: String::from("main"),
            send_a_level: 0.0,
            send_b_level: 0.0,
            send_c_level: 0.0,
        }
    }
}

/// Send effect track state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendTrackState {
    /// Send track index
    pub index: u8,
    /// Send track name
    pub name: String,
    /// Effect slots
    pub effect_slots: Vec<EffectSlotState>,
    /// Pre/Post fader
    pub pre_post: String,
    /// Effect return level
    pub return_level: f64,
    /// Effect enabled
    pub enabled: bool,
}

impl Default for SendTrackState {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::from("Reverb"),
            effect_slots: Vec::with_capacity(2),
            pre_post: String::from("post"),
            return_level: 0.5,
            enabled: true,
        }
    }
}

/// Modulation routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModulationRouting {
    /// Source name
    pub source: String,
    /// Source details
    pub source_details: String,
    /// Target name
    pub target: String,
    /// Target details
    pub target_details: String,
    /// Modulation amount
    pub amount: f64,
    /// Modulation enabled
    pub enabled: bool,
}

impl Default for ModulationRouting {
    fn default() -> Self {
        Self {
            source: String::new(),
            source_details: String::new(),
            target: String::new(),
            target_details: String::new(),
            amount: 0.0,
            enabled: true,
        }
    }
}

/// Piano roll note
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PianoRollNote {
    /// Note pitch (MIDI note number)
    pub pitch: u8,
    /// Note velocity (0-127)
    pub velocity: u8,
    /// Start time (in steps)
    pub start_step: u16,
    /// Duration (in steps)
    pub duration: u16,
    /// Note probability
    pub probability: f64,
    /// Note length
    pub note_length: f64,
}

impl Default for PianoRollNote {
    fn default() -> Self {
        Self {
            pitch: 60,
            velocity: 100,
            start_step: 0,
            duration: 4,
            probability: 1.0,
            note_length: 1.0,
        }
    }
}

/// Pattern state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternState {
    /// Pattern index
    pub index: u8,
    /// Pattern name
    pub name: String,
    /// Pattern length (in steps)
    pub length: u16,
    /// Pattern color
    pub color: String,
    /// Piano roll notes
    pub notes: Vec<PianoRollNote>,
    /// Pattern swing
    pub swing: f64,
    /// Pattern swing style
    pub swing_style: String,
    /// Automation data
    pub automation: HashMap<String, Vec<(f64, f64)>>,
}

impl Default for PatternState {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::from("Pattern 1"),
            length: 16,
            color: String::from("#FF6B6B"),
            notes: Vec::new(),
            swing: 0.0,
            swing_style: String::from("eighths"),
            automation: HashMap::new(),
        }
    }
}

/// Drum track pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrumPatternState {
    /// Drum track index
    pub track_index: u8,
    /// Drum sound name
    pub sound_name: String,
    /// Pattern data (steps)
    pub steps: Vec<bool>,
    /// Velocity data
    pub velocities: Vec<u8>,
    /// Probability data
    pub probabilities: Vec<f64>,
    /// Parameter locks
    pub param_locks: HashMap<u16, HashMap<String, f64>>,
}

impl Default for DrumPatternState {
    fn default() -> Self {
        Self {
            track_index: 0,
            sound_name: String::from("Kick"),
            steps: vec![false; 16],
            velocities: vec![100; 16],
            probabilities: vec![1.0; 16],
            param_locks: HashMap::new(),
        }
    }
}

/// Drum track state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrumTrackState {
    /// Track index
    pub index: u8,
    /// Track name
    pub name: String,
    /// Drum patterns
    pub patterns: Vec<DrumPatternState>,
    /// Track volume
    pub volume: f64,
    /// Track pan
    pub pan: f64,
    /// Track mute
    pub mute: bool,
    /// Track solo
    pub solo: bool,
}

impl Default for DrumTrackState {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::from("Drums"),
            patterns: Vec::with_capacity(8),
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
        }
    }
}

/// Main project structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// Project metadata
    pub metadata: ProjectMetadata,
    /// Global settings
    pub global_settings: GlobalSettings,
    /// Track states
    pub tracks: Vec<TrackState>,
    /// Drum tracks
    pub drum_tracks: Vec<DrumTrackState>,
    /// Patterns
    pub patterns: Vec<PatternState>,
    /// Send tracks
    pub send_tracks: Vec<SendTrackState>,
    /// Modulation routings
    pub modulation_routings: Vec<ModulationRouting>,
    /// Current pattern index
    pub current_pattern: u8,
    /// Current song index
    pub current_song: u8,
    /// Pattern bank
    pub pattern_bank: u8,
    /// Is project modified
    pub is_modified: bool,
    /// Project tags
    pub tags: Vec<String>,
    /// Project author
    pub author: String,
    /// Preset associations
    pub preset_associations: HashMap<String, String>,
    /// Custom data (for extensibility)
    pub custom_data: HashMap<String, String>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            metadata: ProjectMetadata::default(),
            global_settings: GlobalSettings::default(),
            tracks: Vec::with_capacity(8),
            drum_tracks: Vec::with_capacity(4),
            patterns: Vec::with_capacity(16),
            send_tracks: Vec::with_capacity(3),
            modulation_routings: Vec::new(),
            current_pattern: 0,
            current_song: 0,
            pattern_bank: 0,
            is_modified: false,
            tags: Vec::new(),
            author: String::new(),
            preset_associations: HashMap::new(),
            custom_data: HashMap::new(),
        }
    }
}

/// Project errors
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectError {
    /// Invalid project format
    InvalidFormat,
    /// Project version mismatch
    VersionMismatch,
    /// Missing required field
    MissingField(String),
    /// Serialization error
    SerializationError(String),
    /// Deserialization error
    DeserializationError(String),
    /// File I/O error
    FileIoError(String),
    /// Project too large
    TooLarge,
    /// Invalid data
    InvalidData(String),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::InvalidFormat => write!(f, "Invalid project format"),
            ProjectError::VersionMismatch => write!(f, "Project version mismatch"),
            ProjectError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ProjectError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ProjectError::DeserializationError(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
            ProjectError::FileIoError(msg) => write!(f, "File I/O error: {}", msg),
            ProjectError::TooLarge => write!(f, "Project file too large"),
            ProjectError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for ProjectError {}

/// Project save/load manager
#[derive(Debug, Clone)]
pub struct ProjectManager {
    /// Current project
    pub current_project: Project,
    /// Last saved path
    pub last_saved_path: Option<String>,
    /// Auto-save enabled
    pub auto_save_enabled: bool,
    /// Auto-save interval (seconds)
    pub auto_save_interval: u32,
    /// Last auto-save time
    pub last_auto_save_time: u64,
    /// Maximum backup count
    pub max_backup_count: u32,
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self {
            current_project: Project::default(),
            last_saved_path: None,
            auto_save_enabled: true,
            auto_save_interval: 300, // 5 minutes
            last_auto_save_time: 0,
            max_backup_count: 5,
        }
    }
}

impl ProjectManager {
    /// Create new project manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new project
    pub fn new_project(&mut self) {
        self.current_project = Project::default();
        self.last_saved_path = None;
    }

    /// Load project from JSON string
    pub fn from_json(json: &str) -> Result<Project, ProjectError> {
        serde_json::from_str(json)
            .map_err(|e| ProjectError::DeserializationError(format!("Failed to parse JSON: {}", e)))
    }

    /// Convert project to JSON string
    pub fn to_json(project: &Project) -> Result<String, ProjectError> {
        serde_json::to_string_pretty(project).map_err(|e| {
            ProjectError::SerializationError(format!("Failed to serialize project: {}", e))
        })
    }

    /// Load project from file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), ProjectError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ProjectError::FileIoError(format!("Failed to read file: {}", e)))?;
        self.current_project = Self::from_json(&content)?;
        self.last_saved_path = Some(path.to_string());
        self.current_project.is_modified = false;
        Ok(())
    }

    /// Save project to file
    pub fn save_to_file(&mut self, path: &str) -> Result<(), ProjectError> {
        // Update modified timestamp
        self.current_project.metadata.modified_at = std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        let json = Self::to_json(&self.current_project)?;
        std::fs::write(path, json)
            .map_err(|e| ProjectError::FileIoError(format!("Failed to write file: {}", e)))?;

        self.last_saved_path = Some(path.to_string());
        self.current_project.is_modified = false;
        Ok(())
    }

    /// Save project as new version
    pub fn save_as_version(&mut self, base_path: &str) -> Result<String, ProjectError> {
        let timestamp = std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_secs();
        let versioned_path = format!("{}_{}.wavelet", base_path, timestamp);

        self.save_to_file(&versioned_path)?;
        Ok(versioned_path)
    }

    /// Check if auto-save is needed
    pub fn check_auto_save(&mut self) -> Option<Result<String, ProjectError>> {
        if !self.auto_save_enabled || !self.current_project.is_modified {
            return None;
        }

        let now = std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        if now - self.last_auto_save_time >= self.auto_save_interval as u64 {
            self.last_auto_save_time = now;

            // Clone path to avoid mutable borrow while holding immutable reference
            if let Some(path) = self.last_saved_path.clone() {
                let auto_save_path = format!("{}_autosave", path);
                if let Err(e) = self.save_to_file(&auto_save_path) {
                    return Some(Err(e));
                }
                return Some(Ok(auto_save_path));
            }
        }

        None
    }

    /// Create backup of current project
    pub fn create_backup(&mut self) -> Result<String, ProjectError> {
        let timestamp = std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        let backup_name = format!(
            "backup_{}_{}",
            self.current_project.metadata.name.replace(' ', "_"),
            timestamp
        );

        let backup_path = if let Some(ref dir) = self.last_saved_path {
            let path = std::path::Path::new(dir);
            let parent = path.parent().unwrap_or(std::path::Path::new("."));
            parent.join(&backup_name).to_string_lossy().to_string()
        } else {
            format!("{}.backup", backup_name)
        };

        self.save_to_file(&backup_path)?;
        Ok(backup_path)
    }

    /// Get project name
    pub fn project_name(&self) -> &str {
        &self.current_project.metadata.name
    }

    /// Set project name
    pub fn set_project_name(&mut self, name: &str) {
        self.current_project.metadata.name = name.to_string();
        self.current_project.is_modified = true;
    }

    /// Check if project is modified
    pub fn is_modified(&self) -> bool {
        self.current_project.is_modified
    }

    /// Mark project as modified
    pub fn mark_modified(&mut self) {
        self.current_project.is_modified = true;
    }

    /// Get BPM
    pub fn bpm(&self) -> u32 {
        self.current_project.metadata.bpm
    }

    /// Set BPM
    pub fn set_bpm(&mut self, bpm: u32) {
        self.current_project.metadata.bpm = bpm.clamp(20, 999);
        self.current_project.is_modified = true;
    }

    /// Add track
    pub fn add_track(&mut self, track: TrackState) {
        self.current_project.tracks.push(track);
        self.current_project.is_modified = true;
    }

    /// Remove track
    pub fn remove_track(&mut self, index: usize) -> Result<(), ProjectError> {
        if index >= self.current_project.tracks.len() {
            return Err(ProjectError::InvalidData(format!(
                "Track index {} out of bounds",
                index
            )));
        }
        self.current_project.tracks.remove(index);
        self.current_project.is_modified = true;
        Ok(())
    }

    /// Get track count
    pub fn track_count(&self) -> usize {
        self.current_project.tracks.len()
    }

    /// Add drum track
    pub fn add_drum_track(&mut self, track: DrumTrackState) {
        self.current_project.drum_tracks.push(track);
        self.current_project.is_modified = true;
    }

    /// Add pattern
    pub fn add_pattern(&mut self, pattern: PatternState) {
        self.current_project.patterns.push(pattern);
        self.current_project.is_modified = true;
    }

    /// Add send track
    pub fn add_send_track(&mut self, track: SendTrackState) {
        self.current_project.send_tracks.push(track);
        self.current_project.is_modified = true;
    }

    /// Add modulation routing
    pub fn add_modulation_routing(&mut self, routing: ModulationRouting) {
        self.current_project.modulation_routings.push(routing);
        self.current_project.is_modified = true;
    }

    /// Export project summary
    pub fn export_summary(&self) -> String {
        format!(
            "Project: {}\nBPM: {}\nTracks: {}\nDrum Tracks: {}\nPatterns: {}\nSend Tracks: {}\nModulation Routings: {}",
            self.current_project.metadata.name,
            self.current_project.metadata.bpm,
            self.current_project.tracks.len(),
            self.current_project.drum_tracks.len(),
            self.current_project.patterns.len(),
            self.current_project.send_tracks.len(),
            self.current_project.modulation_routings.len(),
        )
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_metadata_default() {
        let metadata = ProjectMetadata::default();
        assert_eq!(metadata.name, "Untitled Project");
        assert_eq!(metadata.bpm, 120);
        assert_eq!(metadata.time_signature_numerator, 4);
        assert_eq!(metadata.time_signature_denominator, 4);
        assert!(metadata.created_at > 0);
    }

    #[test]
    fn test_global_settings_default() {
        let settings = GlobalSettings::default();
        assert_eq!(settings.master_volume, 0.8);
        assert_eq!(settings.swing, 0.0);
        assert_eq!(settings.midi_channel, 0);
    }

    #[test]
    fn test_oscillator_state_default() {
        let osc = OscillatorState::default();
        assert_eq!(osc.osc_index, 0);
        assert_eq!(osc.waveform, "sine");
        assert_eq!(osc.volume, 0.7);
        assert!(!osc.muted);
    }

    #[test]
    fn test_filter_state_default() {
        let filter = FilterState::default();
        assert_eq!(filter.filter_type, "lowpass");
        assert_eq!(filter.cutoff, 2000.0);
        assert_eq!(filter.resonance, 1.0);
        assert!(filter.enabled);
    }

    #[test]
    fn test_envelope_state_default() {
        let env = EnvelopeState::default();
        assert_eq!(env.name, "Default");
        assert_eq!(env.attack, 0.01);
        assert_eq!(env.decay, 0.2);
        assert_eq!(env.sustain, 0.7);
        assert_eq!(env.release, 0.3);
    }

    #[test]
    fn test_lfo_state_default() {
        let lfo = LfoState::default();
        assert_eq!(lfo.waveform, "sine");
        assert_eq!(lfo.frequency, 4.0);
        assert_eq!(lfo.amplitude, 0.5);
        assert!(!lfo.sync);
    }

    #[test]
    fn test_effect_slot_state_default() {
        let slot = EffectSlotState::default();
        assert!(slot.effect_type.is_empty());
        assert!(!slot.enabled);
        assert_eq!(slot.mix, 0.5);
    }

    #[test]
    fn test_track_state_default() {
        let track = TrackState::default();
        assert_eq!(track.name, "Track 1");
        assert_eq!(track.track_type, "synth");
        assert_eq!(track.volume, 0.8);
        assert!(!track.mute);
        assert!(!track.solo);
    }

    #[test]
    fn test_send_track_state_default() {
        let send = SendTrackState::default();
        assert_eq!(send.name, "Reverb");
        assert_eq!(send.pre_post, "post");
        assert_eq!(send.return_level, 0.5);
        assert!(send.enabled);
    }

    #[test]
    fn test_modulation_routing_default() {
        let routing = ModulationRouting::default();
        assert!(routing.source.is_empty());
        assert!(routing.target.is_empty());
        assert_eq!(routing.amount, 0.0);
        assert!(routing.enabled);
    }

    #[test]
    fn test_piano_roll_note_default() {
        let note = PianoRollNote::default();
        assert_eq!(note.pitch, 60);
        assert_eq!(note.velocity, 100);
        assert_eq!(note.start_step, 0);
        assert_eq!(note.duration, 4);
    }

    #[test]
    fn test_pattern_state_default() {
        let pattern = PatternState::default();
        assert_eq!(pattern.name, "Pattern 1");
        assert_eq!(pattern.length, 16);
        assert_eq!(pattern.swing, 0.0);
        assert!(pattern.notes.is_empty());
    }

    #[test]
    fn test_drum_pattern_state_default() {
        let pattern = DrumPatternState::default();
        assert_eq!(pattern.track_index, 0);
        assert_eq!(pattern.sound_name, "Kick");
        assert_eq!(pattern.steps.len(), 16);
        assert!(pattern.steps.iter().all(|&b| !b));
    }

    #[test]
    fn test_drum_track_state_default() {
        let track = DrumTrackState::default();
        assert_eq!(track.name, "Drums");
        assert_eq!(track.volume, 0.8);
        assert!(!track.mute);
    }

    #[test]
    fn test_project_default() {
        let project = Project::default();
        assert_eq!(project.metadata.name, "Untitled Project");
        assert_eq!(project.tracks.len(), 0);
        assert_eq!(project.drum_tracks.len(), 0);
        assert_eq!(project.patterns.len(), 0);
        assert!(!project.is_modified);
    }

    #[test]
    fn test_project_serialization() {
        let mut project = Project::default();
        project.metadata.name = "Test Project".to_string();
        project.metadata.bpm = 140;
        project.tracks.push(TrackState::default());

        let json = ProjectManager::to_json(&project).unwrap();
        let parsed = ProjectManager::from_json(&json).unwrap();

        assert_eq!(project.metadata.name, parsed.metadata.name);
        assert_eq!(project.metadata.bpm, parsed.metadata.bpm);
        assert_eq!(project.tracks.len(), parsed.tracks.len());
    }

    #[test]
    fn test_project_manager_new() {
        let manager = ProjectManager::new();
        assert_eq!(manager.track_count(), 0);
        assert!(!manager.is_modified());
        assert!(manager.auto_save_enabled);
        assert_eq!(manager.auto_save_interval, 300);
    }

    #[test]
    fn test_project_manager_new_project() {
        let mut manager = ProjectManager::new();
        manager.current_project.metadata.name = "Modified".to_string();
        manager.mark_modified();

        manager.new_project();
        assert_eq!(manager.project_name(), "Untitled Project");
        assert!(!manager.is_modified());
    }

    #[test]
    fn test_project_manager_set_bpm() {
        let mut manager = ProjectManager::new();
        assert_eq!(manager.bpm(), 120);

        manager.set_bpm(180);
        assert_eq!(manager.bpm(), 180);
        assert!(manager.is_modified());

        // Test clamping
        manager.set_bpm(1000);
        assert_eq!(manager.bpm(), 999);

        manager.set_bpm(10);
        assert_eq!(manager.bpm(), 20);
    }

    #[test]
    fn test_project_manager_add_track() {
        let mut manager = ProjectManager::new();
        assert_eq!(manager.track_count(), 0);

        manager.add_track(TrackState::default());
        assert_eq!(manager.track_count(), 1);

        manager.add_track(TrackState::default());
        assert_eq!(manager.track_count(), 2);
    }

    #[test]
    fn test_project_manager_remove_track() {
        let mut manager = ProjectManager::new();
        manager.add_track(TrackState::default());
        manager.add_track(TrackState::default());
        assert_eq!(manager.track_count(), 2);

        manager.remove_track(0).unwrap();
        assert_eq!(manager.track_count(), 1);

        assert!(manager.remove_track(5).is_err());
    }

    #[test]
    fn test_project_manager_add_drum_track() {
        let mut manager = ProjectManager::new();
        manager.add_drum_track(DrumTrackState::default());
        assert_eq!(manager.current_project.drum_tracks.len(), 1);
    }

    #[test]
    fn test_project_manager_add_pattern() {
        let mut manager = ProjectManager::new();
        manager.add_pattern(PatternState::default());
        assert_eq!(manager.current_project.patterns.len(), 1);
    }

    #[test]
    fn test_project_manager_add_send_track() {
        let mut manager = ProjectManager::new();
        manager.add_send_track(SendTrackState::default());
        assert_eq!(manager.current_project.send_tracks.len(), 1);
    }

    #[test]
    fn test_project_manager_add_modulation_routing() {
        let mut manager = ProjectManager::new();
        manager.add_modulation_routing(ModulationRouting::default());
        assert_eq!(manager.current_project.modulation_routings.len(), 1);
    }

    #[test]
    fn test_project_manager_set_project_name() {
        let mut manager = ProjectManager::new();
        assert_eq!(manager.project_name(), "Untitled Project");

        manager.set_project_name("My Awesome Track");
        assert_eq!(manager.project_name(), "My Awesome Track");
        assert!(manager.is_modified());
    }

    #[test]
    fn test_project_manager_mark_modified() {
        let mut manager = ProjectManager::new();
        assert!(!manager.is_modified());

        manager.mark_modified();
        assert!(manager.is_modified());
    }

    #[test]
    fn test_project_error_display() {
        let errors = vec![
            ProjectError::InvalidFormat,
            ProjectError::VersionMismatch,
            ProjectError::MissingField("test".to_string()),
            ProjectError::SerializationError("error".to_string()),
            ProjectError::DeserializationError("error".to_string()),
            ProjectError::FileIoError("error".to_string()),
            ProjectError::TooLarge,
            ProjectError::InvalidData("error".to_string()),
        ];

        for error in errors {
            let msg = error.to_string();
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn test_project_summary() {
        let mut manager = ProjectManager::new();
        manager.set_project_name("Test Project");
        manager.add_track(TrackState::default());
        manager.add_drum_track(DrumTrackState::default());
        manager.add_pattern(PatternState::default());
        manager.add_send_track(SendTrackState::default());
        manager.add_modulation_routing(ModulationRouting::default());

        let summary = manager.export_summary();
        assert!(summary.contains("Test Project"));
        assert!(summary.contains("Tracks: 1"));
        assert!(summary.contains("Drum Tracks: 1"));
        assert!(summary.contains("Patterns: 1"));
        assert!(summary.contains("Send Tracks: 1"));
        assert!(summary.contains("Modulation Routings: 1"));
    }
}
