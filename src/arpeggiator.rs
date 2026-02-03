//! Arpeggiator Module
//!
//! This module provides arpeggiator functionality for the WAVELET synthesizer.
//! An arpeggiator takes the notes currently held down and plays them in
//! a specific pattern (up, down, up-down, random, etc.).
//!
//! # Features
//! - 8 arpeggio patterns: Up, Down, Up-Down, Down-Up, Random, Order, Chord, Pattern
//! - Configurable note order and pattern length
//! - Syncable to MIDI clock
//! - Automatic chord holding when keys are released

use std::fmt;

/// Arpeggiator pattern types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpPattern {
    /// Notes played in ascending order
    #[default]
    Up,
    /// Notes played in descending order
    Down,
    /// Notes played up then down
    UpDown,
    /// Notes played down then up
    DownUp,
    /// Notes played in random order
    Random,
    /// Notes played in the order they were pressed
    Order,
    /// All notes played simultaneously (chord strum effect)
    Chord,
    /// Custom pattern (future extensibility)
    Pattern,
}

impl fmt::Display for ArpPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArpPattern::Up => write!(f, "Up"),
            ArpPattern::Down => write!(f, "Down"),
            ArpPattern::UpDown => write!(f, "Up-Down"),
            ArpPattern::DownUp => write!(f, "Down-Up"),
            ArpPattern::Random => write!(f, "Random"),
            ArpPattern::Order => write!(f, "Order"),
            ArpPattern::Chord => write!(f, "Chord"),
            ArpPattern::Pattern => write!(f, "Pattern"),
        }
    }
}

/// Arpeggiator configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct ArpConfig {
    /// Pattern type for the arpeggiator
    pub pattern: ArpPattern,

    /// BPM (beats per minute) - determines arpeggio speed
    /// Range: 20-300 BPM
    pub bpm: f32,

    /// Note value for each arpeggio step
    /// Represents the rhythmic division (e.g., 1/4 = quarter notes)
    pub note_value: ArpNoteValue,

    /// Octave range for the arpeggio
    /// How many octaves to span when playing patterns
    pub octave_span: u8,

    /// Whether to repeat the pattern continuously
    pub repeat: bool,

    /// Gate length (percentage of step duration)
    /// 100 = staccato, 100 = full length
    pub gate_length: f32,

    /// Swing percentage (0-50)
    /// 0 = straight timing, 50 = heavy swing
    pub swing: f32,

    /// Whether the arpeggiator is enabled
    pub enabled: bool,
}

impl Default for ArpConfig {
    fn default() -> Self {
        Self {
            pattern: ArpPattern::Up,
            bpm: 120.0,
            note_value: ArpNoteValue::Quarter,
            octave_span: 1,
            repeat: true,
            gate_length: 80.0,
            swing: 0.0,
            enabled: true, // Enabled by default for arpeggiator
        }
    }
}

/// Note values for arpeggio timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpNoteValue {
    /// 1/1 (whole note)
    Whole,
    /// 1/2 (half note)
    Half,
    /// 1/4 (quarter note)
    #[default]
    Quarter,
    /// 1/8 (eighth note)
    Eighth,
    /// 1/16 (sixteenth note)
    Sixteenth,
    /// 1/32 (thirty-second note)
    ThirtySecond,
}

impl fmt::Display for ArpNoteValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArpNoteValue::Whole => write!(f, "1/1"),
            ArpNoteValue::Half => write!(f, "1/2"),
            ArpNoteValue::Quarter => write!(f, "1/4"),
            ArpNoteValue::Eighth => write!(f, "1/8"),
            ArpNoteValue::Sixteenth => write!(f, "1/16"),
            ArpNoteValue::ThirtySecond => write!(f, "1/32"),
        }
    }
}

impl ArpNoteValue {
    /// Converts note value to duration in seconds at the given BPM.
    pub fn to_duration(&self, bpm: f32) -> f32 {
        let beat_duration = 60.0 / bpm;
        match self {
            ArpNoteValue::Whole => beat_duration * 4.0,
            ArpNoteValue::Half => beat_duration * 2.0,
            ArpNoteValue::Quarter => beat_duration,
            ArpNoteValue::Eighth => beat_duration / 2.0,
            ArpNoteValue::Sixteenth => beat_duration / 4.0,
            ArpNoteValue::ThirtySecond => beat_duration / 8.0,
        }
    }
}

/// Internal representation of an arpeggio note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ArpNote {
    /// MIDI note number (0-127)
    note: u8,

    /// Which voice index this note came from (for Order pattern)
    order_index: usize,
}

impl ArpNote {
    fn new(note: u8, order_index: usize) -> Self {
        Self { note, order_index }
    }
}

/// Arpeggiator state machine.
#[derive(Debug, Clone)]
pub struct Arpeggiator {
    /// Configuration
    config: ArpConfig,

    /// Current state
    state: ArpState,

    /// Held notes (sorted for pattern processing)
    held_notes: Vec<ArpNote>,

    /// Notes that were held when arp started
    initial_notes: Vec<ArpNote>,

    /// Current position in the arpeggio pattern
    position: usize,

    /// Current direction (for bidirectional patterns)
    direction: ArpDirection,

    /// Sample rate for timing calculations
    sample_rate: f32,

    /// Samples until next note trigger
    samples_until_next: f32,

    /// Samples per step (calculated from BPM and note value)
    samples_per_step: f32,

    /// Current swing modifier (alternates between 0 and swing)
    swing_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArpDirection {
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArpState {
    Idle,
    Playing,
    Held, // Holding chord when note released
}

impl Default for Arpeggiator {
    fn default() -> Self {
        Self {
            config: ArpConfig::default(),
            state: ArpState::Idle,
            held_notes: Vec::new(),
            initial_notes: Vec::new(),
            position: 0,
            direction: ArpDirection::Up,
            sample_rate: 44100.0,
            samples_until_next: 0.0,
            samples_per_step: 0.0,
            swing_index: 0,
        }
    }
}

impl Arpeggiator {
    /// Creates a new arpeggiator with the given sample rate.
    /// The arpeggiator is disabled by default.
    pub fn new(sample_rate: f32) -> Self {
        let mut config = ArpConfig::default();
        config.enabled = false;
        Self {
            config,
            sample_rate,
            ..Default::default()
        }
    }

    /// Creates a new arpeggiator with the given configuration.
    pub fn with_config(config: ArpConfig, sample_rate: f32) -> Self {
        let mut arp = Self {
            config,
            sample_rate,
            ..Default::default()
        };
        arp.update_timing();
        arp
    }

    /// Updates timing calculations when BPM or note value changes.
    fn update_timing(&mut self) {
        let step_duration = self.config.note_value.to_duration(self.config.bpm);
        self.samples_per_step = step_duration * self.sample_rate;
    }

    /// Sets the arpeggiator configuration.
    pub fn set_config(&mut self, config: ArpConfig) {
        self.config = config;
        self.update_timing();
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &ArpConfig {
        &self.config
    }

    /// Enables or disables the arpeggiator.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        if !enabled {
            self.stop();
        }
    }

    /// Checks if the arpeggiator is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Sets the BPM.
    pub fn set_bpm(&mut self, bpm: f32) {
        self.config.bpm = bpm.clamp(20.0, 300.0);
        self.update_timing();
    }

    /// Sets the pattern type.
    pub fn set_pattern(&mut self, pattern: ArpPattern) {
        self.config.pattern = pattern;
        self.position = 0;
        self.direction = ArpDirection::Up;
    }

    /// Adds a note to the held notes.
    pub fn note_on(&mut self, note: u8, _velocity: u8) {
        // Only care about note number, not velocity for arpeggio
        let order_index = self.held_notes.len();
        self.held_notes.push(ArpNote::new(note, order_index));
        self.held_notes.sort(); // Keep sorted for easy processing

        // If we were idle, start playing
        if self.state == ArpState::Idle {
            self.start();
        }

        // If not repeating, capture initial state
        if !self.config.repeat {
            self.initial_notes = self.held_notes.clone();
            self.position = 0;
        }
    }

    /// Removes a note from the held notes.
    pub fn note_off(&mut self, note: u8) {
        self.held_notes.retain(|n| n.note != note);

        // If we have no notes held, decide what to do
        if self.held_notes.is_empty() {
            match self.state {
                ArpState::Playing => {
                    // Continue to next cycle if repeating, otherwise stop
                    if !self.config.repeat {
                        self.state = ArpState::Held;
                    }
                }
                ArpState::Held => {
                    self.state = ArpState::Idle;
                }
                _ => {}
            }
        }
    }

    /// Starts the arpeggiator.
    fn start(&mut self) {
        self.state = ArpState::Playing;
        self.initial_notes = self.held_notes.clone();
        self.position = 0;
        self.direction = ArpDirection::Up;
        self.swing_index = 0;
        self.samples_until_next = 0.0;
    }

    /// Stops the arpeggiator.
    fn stop(&mut self) {
        self.state = ArpState::Idle;
        self.held_notes.clear();
        self.initial_notes.clear();
        self.position = 0;
    }

    /// Gets the next note to play based on the current pattern.
    fn get_next_note(&mut self) -> Option<u8> {
        if self.initial_notes.is_empty() {
            return None;
        }

        let pattern = self.config.pattern;
        let octave_span = self.config.octave_span;
        let num_notes = self.initial_notes.len();

        // Generate extended note set based on octave span
        // octave_span = 1 means just base octave
        // octave_span = 2 means base + 1 more octave (2 total)
        let mut extended_notes: Vec<ArpNote> = Vec::new();

        // Base octave notes and higher octaves
        for octave in 0..octave_span {
            let octave_offset = (octave * 12) as u8;
            for (i, note) in self.initial_notes.iter().enumerate() {
                let new_note = note.note.saturating_add(octave_offset);
                if new_note <= 127 {
                    extended_notes.push(ArpNote::new(new_note, i));
                }
            }
        }

        let total_notes = extended_notes.len();
        let note = match pattern {
            ArpPattern::Up => {
                if self.position >= total_notes {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                let note = extended_notes[self.position];
                self.position += 1;
                Some(note.note)
            }

            ArpPattern::Down => {
                if self.position >= total_notes {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                let note = extended_notes[total_notes - 1 - self.position];
                self.position += 1;
                Some(note.note)
            }

            ArpPattern::UpDown => {
                let cycle_len = total_notes * 2 - 2;
                if self.position >= cycle_len {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                let idx = if self.position < total_notes {
                    self.position
                } else {
                    cycle_len - self.position
                };
                let note = extended_notes[idx];
                self.position += 1;
                Some(note.note)
            }

            ArpPattern::DownUp => {
                let cycle_len = total_notes * 2 - 2;
                if self.position >= cycle_len {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                let idx = if self.position < total_notes {
                    total_notes - 1 - self.position
                } else {
                    self.position - total_notes + 1
                };
                let note = extended_notes[idx];
                self.position += 1;
                Some(note.note)
            }

            ArpPattern::Random => {
                if self.position >= total_notes {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                let _note = extended_notes[self.position];
                self.position += 1;
                // Use a simple hash for randomness
                let seed = self.position + (self.swing_index * 7919);
                let random_idx = seed % total_notes;
                Some(extended_notes[random_idx].note)
            }

            ArpPattern::Order => {
                // Play notes in the order they were pressed
                if self.position >= num_notes {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                // For order pattern, we only use base notes (no octave spread)
                let note = self.initial_notes.get(self.position)?;
                self.position += 1;
                Some(note.note)
            }

            ArpPattern::Chord => {
                // All notes at once (handled differently - returns multiple)
                // For now, return first note and caller handles chord
                if self.position >= 1 {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                self.position += 1;
                Some(extended_notes[0].note)
            }

            ArpPattern::Pattern => {
                // Future: custom patterns via external data
                // For now, fall back to Up
                if self.position >= total_notes {
                    if self.config.repeat {
                        self.position = 0;
                    } else {
                        return None;
                    }
                }
                let note = extended_notes[self.position];
                self.position += 1;
                Some(note.note)
            }
        };

        note
    }

    /// Gets all notes to play (for Chord pattern).
    pub fn get_current_chord(&self) -> Vec<u8> {
        self.initial_notes.iter().map(|n| n.note).collect()
    }

    /// Processes one sample and returns a note trigger if it's time.
    ///
    /// Returns `Some((note, velocity))` when a new note should trigger,
    /// or `None` if no note is playing.
    pub fn process(&mut self) -> Option<(u8, u8)> {
        if !self.config.enabled {
            return None;
        }

        // Handle Chord pattern specially
        if self.config.pattern == ArpPattern::Chord {
            // Chord plays all notes simultaneously at each step
            // We still need timing, but return all notes at once
            self.samples_until_next -= 1.0;

            if self.samples_until_next <= 0.0 {
                self.samples_until_next = self.samples_per_step;
                let chord = self.get_current_chord();
                if !chord.is_empty() {
                    // Return the first note; caller should handle rest
                    return Some((chord[0], 100));
                }
            }
            return None;
        }

        // Normal arpeggio processing
        self.samples_until_next -= 1.0;

        if self.samples_until_next <= 0.0 {
            // Apply swing timing modification
            let swing_delay = if self.config.swing > 0.0 && self.swing_index % 2 == 1 {
                (self.config.swing / 100.0) * self.samples_per_step * 0.5
            } else {
                0.0
            };

            self.samples_until_next = self.samples_per_step + swing_delay;
            self.swing_index = self.swing_index.wrapping_add(1);

            if let Some(note) = self.get_next_note() {
                // Calculate velocity based on gate length (accent pattern)
                let velocity = if self.config.gate_length >= 90.0 {
                    127
                } else if self.config.gate_length >= 70.0 {
                    100
                } else {
                    80
                };

                return Some((note, velocity as u8));
            }
        }

        None
    }

    /// Returns the current state as a string for UI display.
    pub fn state_string(&self) -> String {
        if !self.config.enabled {
            return "OFF".to_string();
        }

        match self.state {
            ArpState::Idle => "Ready".to_string(),
            ArpState::Playing => {
                format!("{} @ {:.0}", self.config.pattern, self.config.bpm)
            }
            ArpState::Held => "Holding".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arp_config_default() {
        let config = ArpConfig::default();
        assert_eq!(config.pattern, ArpPattern::Up);
        assert_eq!(config.bpm, 120.0);
        assert_eq!(config.octave_span, 1);
        assert!(config.enabled);
    }

    #[test]
    fn test_note_value_durations() {
        let bpm = 120.0;
        assert_eq!(ArpNoteValue::Whole.to_duration(bpm), 2.0);
        assert_eq!(ArpNoteValue::Half.to_duration(bpm), 1.0);
        assert_eq!(ArpNoteValue::Quarter.to_duration(bpm), 0.5);
        assert_eq!(ArpNoteValue::Eighth.to_duration(bpm), 0.25);
        assert_eq!(ArpNoteValue::Sixteenth.to_duration(bpm), 0.125);
    }

    #[test]
    fn test_arp_pattern_display() {
        assert_eq!(ArpPattern::Up.to_string(), "Up");
        assert_eq!(ArpPattern::UpDown.to_string(), "Up-Down");
        assert_eq!(ArpPattern::Random.to_string(), "Random");
    }

    #[test]
    fn test_arpeggiator_basic() {
        let arp = Arpeggiator::new(44100.0);
        assert!(!arp.is_enabled());

        let mut arp = Arpeggiator::with_config(ArpConfig::default(), 44100.0);
        assert!(arp.is_enabled());

        // Add some notes
        arp.note_on(60, 100); // C4
        arp.note_on(64, 100); // E4
        arp.note_on(67, 100); // G4

        // Process should return notes
        let note = arp.process();
        assert!(note.is_some());
        let (note_num, velocity) = note.unwrap();
        assert!(note_num >= 60 && note_num <= 67);
        assert!(velocity > 0);
    }

    #[test]
    fn test_arpeggiator_note_off() {
        let mut arp = Arpeggiator::with_config(ArpConfig::default(), 44100.0);

        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_off(60);

        // Should still have one note
        assert_eq!(arp.held_notes.len(), 1);
    }

    #[test]
    fn test_arpeggiator_octave_span() {
        // Create arpeggiator with very fast BPM for testing
        let mut config = ArpConfig::default();
        config.octave_span = 2; // 2 octaves (base + 1 more)
        config.pattern = ArpPattern::Up;
        config.bpm = 10000.0; // Very fast for testing
        config.note_value = ArpNoteValue::ThirtySecond; // Fastest note value

        let mut arp = Arpeggiator::with_config(config, 44100.0);

        arp.note_on(60, 100); // C4

        // With very fast BPM, we can collect notes quickly
        let mut all_notes: Vec<u8> = Vec::new();
        for _ in 0..100 {
            if let Some((n, _)) = arp.process() {
                all_notes.push(n);
            }
        }

        // Should contain C4 (60) and C5 (72)
        assert!(all_notes.contains(&60), "Should contain C4 (60), got: {:?}", all_notes);
        assert!(all_notes.contains(&72), "Should contain C5 (72), got: {:?}", all_notes);
    }
}
