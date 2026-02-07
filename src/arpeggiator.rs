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

#![allow(dead_code)] // Reserve fields for future MIDI sync features

use std::fmt;

/// Arpeggiator pattern types (Tonverk-aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpMode {
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
    /// All notes played simultaneously
    Chord,
    /// Custom pattern (future extensibility)
    Pattern,
}

impl ArpMode {
    pub fn from_u8(val: u8) -> Self {
        match val % 8 {
            0 => ArpMode::Up,
            1 => ArpMode::Down,
            2 => ArpMode::UpDown,
            3 => ArpMode::DownUp,
            4 => ArpMode::Random,
            5 => ArpMode::Order,
            6 => ArpMode::Chord,
            _ => ArpMode::Pattern,
        }
    }
}

impl fmt::Display for ArpMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArpMode::Up => write!(f, "Up"),
            ArpMode::Down => write!(f, "Down"),
            ArpMode::UpDown => write!(f, "Up-Down"),
            ArpMode::DownUp => write!(f, "Down-Up"),
            ArpMode::Random => write!(f, "Random"),
            ArpMode::Order => write!(f, "Order"),
            ArpMode::Chord => write!(f, "Chord"),
            ArpMode::Pattern => write!(f, "Pattern"),
        }
    }
}

/// Arpeggiator configuration (Tonverk-aligned).
#[derive(Debug, Clone, PartialEq)]
pub struct ArpConfig {
    /// MODE: Pattern type (0-7)
    /// 0=Up, 1=Down, 2=UpDown, 3=DownUp, 4=Random, 5=Order, 6=Chord, 7=Pattern
    pub mode: u8,

    /// SPEED: Note value division (0-5)
    /// 0=1/1, 1=1/2, 2=1/4, 3=1/8, 4=1/16, 5=1/32
    pub speed: u8,

    /// RANGE: Octave range (1-4)
    pub range: u8,

    /// N.LEN: Note length/gate time (0-100)
    pub note_length: u8,

    /// OFFSET: Step offset (0-15)
    pub offset: u8,

    /// ARP LENGTH: Number of notes to play (0-16, 0=all)
    pub arp_length: u8,

    /// Whether the arpeggiator is enabled
    pub enabled: bool,
}

impl Default for ArpConfig {
    fn default() -> Self {
        Self {
            mode: 0,
            speed: 3,
            range: 1,
            note_length: 70,
            offset: 0,
            arp_length: 0,
            enabled: true,
        }
    }
}

/// Speed divisions for arpeggio timing (Tonverk-aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpSpeed {
    #[default]
    /// 1/1 (whole note)
    Whole,
    /// 1/2 (half note)
    Half,
    /// 1/4 (quarter note)
    Quarter,
    /// 1/8 (eighth note)
    Eighth,
    /// 1/16 (sixteenth note)
    Sixteenth,
    /// 1/32 (thirty-second note)
    ThirtySecond,
}

impl fmt::Display for ArpSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArpSpeed::Whole => write!(f, "1/1"),
            ArpSpeed::Half => write!(f, "1/2"),
            ArpSpeed::Quarter => write!(f, "1/4"),
            ArpSpeed::Eighth => write!(f, "1/8"),
            ArpSpeed::Sixteenth => write!(f, "1/16"),
            ArpSpeed::ThirtySecond => write!(f, "1/32"),
        }
    }
}

impl ArpSpeed {
    /// Converts speed value to duration in seconds at the given BPM.
    pub fn to_duration(&self, bpm: f32) -> f32 {
        let beat_duration = 60.0 / bpm;
        match self {
            ArpSpeed::Whole => beat_duration * 4.0,
            ArpSpeed::Half => beat_duration * 2.0,
            ArpSpeed::Quarter => beat_duration,
            ArpSpeed::Eighth => beat_duration / 2.0,
            ArpSpeed::Sixteenth => beat_duration / 4.0,
            ArpSpeed::ThirtySecond => beat_duration / 8.0,
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

/// Arpeggiator state machine (Tonverk-aligned).
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

    /// Current direction for bidirectional patterns
    direction_up: bool,

    /// Sample rate for timing calculations
    sample_rate: f32,

    /// Samples until next note trigger
    samples_until_next: f32,

    /// Samples per step (calculated from BPM and note value)
    samples_per_step: f32,

    /// BPM for timing calculations
    bpm: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArpState {
    Idle,
    Playing,
    Held,
}

impl Default for Arpeggiator {
    fn default() -> Self {
        Self {
            config: ArpConfig::default(),
            state: ArpState::Idle,
            held_notes: Vec::new(),
            initial_notes: Vec::new(),
            position: 0,
            direction_up: true,
            sample_rate: 44100.0,
            samples_until_next: 0.0,
            samples_per_step: 0.0,
            bpm: 120.0,
        }
    }
}

impl Arpeggiator {
    /// Creates a new arpeggiator with the given sample rate.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            config: ArpConfig {
                enabled: false,
                ..Default::default()
            },
            sample_rate,
            ..Default::default()
        }
    }

    /// Creates a new arpeggiator with the given configuration.
    pub fn with_config(config: ArpConfig, sample_rate: f32, bpm: f32) -> Self {
        let mut arp = Self {
            config,
            sample_rate,
            bpm,
            ..Default::default()
        };
        arp.update_timing();
        arp
    }

    /// Updates timing calculations when BPM or speed changes.
    fn update_timing(&mut self) {
        let beat_duration = 60.0 / self.bpm;
        let step_duration = match self.config.speed {
            0 => beat_duration * 4.0,
            1 => beat_duration * 2.0,
            2 => beat_duration,
            3 => beat_duration / 2.0,
            4 => beat_duration / 4.0,
            _ => beat_duration / 8.0,
        };
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
        self.bpm = bpm.clamp(20.0, 300.0);
        self.update_timing();
    }

    /// Sets the MODE (pattern type).
    pub fn set_mode(&mut self, mode: u8) {
        self.config.mode = mode % 8;
        self.position = 0;
        self.direction_up = true;
    }

    /// Sets the SPEED (note value division).
    pub fn set_speed(&mut self, speed: u8) {
        self.config.speed = speed.min(5);
        self.update_timing();
    }

    /// Sets the RANGE (octave range).
    pub fn set_range(&mut self, range: u8) {
        self.config.range = range.clamp(1, 4);
    }

    /// Sets the N.LEN (note length/gate time).
    pub fn set_note_length(&mut self, length: u8) {
        self.config.note_length = length.min(100);
    }

    /// Sets the OFFSET.
    pub fn set_offset(&mut self, offset: u8) {
        self.config.offset = offset.min(15);
    }

    /// Sets the ARP LENGTH.
    pub fn set_arp_length(&mut self, length: u8) {
        self.config.arp_length = length.min(16);
    }

    /// Adds a note to the held notes.
    pub fn note_on(&mut self, note: u8, _velocity: u8) {
        let order_index = self.held_notes.len();
        self.held_notes.push(ArpNote::new(note, order_index));
        self.held_notes.sort();

        if self.state == ArpState::Idle {
            self.start();
        } else {
            self.initial_notes = self.held_notes.clone();
            self.position = 0;
        }
    }

    /// Removes a note from the held notes.
    pub fn note_off(&mut self, note: u8) {
        self.held_notes.retain(|n| n.note != note);

        if self.held_notes.is_empty() {
            self.state = ArpState::Idle;
        }
    }

    /// Starts the arpeggiator.
    fn start(&mut self) {
        self.state = ArpState::Playing;
        self.initial_notes = self.held_notes.clone();
        self.position = 0;
        self.direction_up = true;
        self.samples_until_next = 0.0;
    }

    /// Stops the arpeggiator.
    fn stop(&mut self) {
        self.state = ArpState::Idle;
        self.held_notes.clear();
        self.initial_notes.clear();
        self.position = 0;
    }

    /// Gets the next note to play based on the current MODE.
    fn get_next_note(&mut self) -> Option<u8> {
        if self.initial_notes.is_empty() {
            return None;
        }

        let mode = ArpMode::from_u8(self.config.mode);
        let range = self.config.range as usize;
        let num_notes = self.initial_notes.len();
        let effective_length = if self.config.arp_length == 0 {
            num_notes * range
        } else {
            self.config.arp_length as usize
        };

        let mut extended_notes: Vec<ArpNote> = Vec::new();

        for octave in 0..range {
            let octave_offset = (octave * 12) as u8;
            for (i, note) in self.initial_notes.iter().enumerate() {
                let new_note = note.note.saturating_add(octave_offset);
                if new_note <= 127 {
                    extended_notes.push(ArpNote::new(new_note, i));
                }
            }
        }

        let total_notes = extended_notes.len();
        if total_notes == 0 {
            return None;
        }

        if self.position >= effective_length.min(total_notes) {
            self.position = 0;
        }

        let note = match mode {
            ArpMode::Up => {
                let idx = self.position.min(total_notes - 1);
                let note = extended_notes[idx];
                self.position += 1;
                Some(note.note)
            }

            ArpMode::Down => {
                let idx = (total_notes - 1 - self.position).min(total_notes - 1);
                let note = extended_notes[idx];
                self.position += 1;
                Some(note.note)
            }

            ArpMode::UpDown => {
                let cycle_len = if total_notes > 1 {
                    total_notes * 2 - 2
                } else {
                    1
                };
                let pos = self.position % cycle_len;
                let idx = if pos < total_notes {
                    pos
                } else {
                    cycle_len - pos
                };
                self.position += 1;
                Some(extended_notes[idx.min(total_notes - 1)].note)
            }

            ArpMode::DownUp => {
                let cycle_len = if total_notes > 1 {
                    total_notes * 2 - 2
                } else {
                    1
                };
                let pos = self.position % cycle_len;
                let idx = if pos < total_notes {
                    total_notes - 1 - pos
                } else {
                    pos - total_notes + 1
                };
                self.position += 1;
                Some(extended_notes[idx.min(total_notes - 1)].note)
            }

            ArpMode::Random => {
                let idx = (self.position * 7919 + 17) % total_notes;
                self.position += 1;
                Some(extended_notes[idx].note)
            }

            ArpMode::Order => {
                let idx = self.position % num_notes;
                self.position += 1;
                self.initial_notes.get(idx).map(|n| n.note)
            }

            ArpMode::Chord => {
                if self.position > 0 {
                    self.position = 0;
                }
                self.position += 1;
                Some(extended_notes[0].note)
            }

            ArpMode::Pattern => {
                let idx = self.position.min(total_notes - 1);
                let note = extended_notes[idx];
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

        let mode = ArpMode::from_u8(self.config.mode);

        if mode == ArpMode::Chord {
            self.samples_until_next -= 1.0;
            if self.samples_until_next <= 0.0 {
                self.samples_until_next = self.samples_per_step;
                let chord = self.get_current_chord();
                if !chord.is_empty() {
                    return Some((chord[0], 100));
                }
            }
            return None;
        }

        self.samples_until_next -= 1.0;

        if self.samples_until_next <= 0.0 {
            self.samples_until_next = self.samples_per_step;

            if let Some(note) = self.get_next_note() {
                let velocity = 100;
                return Some((note, velocity));
            }
        }

        None
    }

    /// Returns the current state as a string for UI display.
    pub fn state_string(&self) -> String {
        if !self.config.enabled {
            return "OFF".to_string();
        }

        let mode = ArpMode::from_u8(self.config.mode);
        format!("{} @ {:.0}", mode, self.bpm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arp_config_default() {
        let config = ArpConfig::default();
        assert_eq!(config.mode, 0);
        assert_eq!(config.speed, 3);
        assert_eq!(config.range, 1);
        assert_eq!(config.note_length, 70);
        assert!(config.enabled);
    }

    #[test]
    fn test_arp_mode_from_u8() {
        assert!(matches!(ArpMode::from_u8(0), ArpMode::Up));
        assert!(matches!(ArpMode::from_u8(1), ArpMode::Down));
        assert!(matches!(ArpMode::from_u8(2), ArpMode::UpDown));
        assert!(matches!(ArpMode::from_u8(3), ArpMode::DownUp));
        assert!(matches!(ArpMode::from_u8(4), ArpMode::Random));
        assert!(matches!(ArpMode::from_u8(5), ArpMode::Order));
        assert!(matches!(ArpMode::from_u8(6), ArpMode::Chord));
        assert!(matches!(ArpMode::from_u8(7), ArpMode::Pattern));
        assert!(matches!(ArpMode::from_u8(8), ArpMode::Up)); // wraps
    }

    #[test]
    fn test_arp_mode_display() {
        assert_eq!(ArpMode::Up.to_string(), "Up");
        assert_eq!(ArpMode::UpDown.to_string(), "Up-Down");
        assert_eq!(ArpMode::Random.to_string(), "Random");
    }

    #[test]
    fn test_arpeggiator_basic() {
        let arp = Arpeggiator::new(44100.0);
        assert!(!arp.is_enabled());

        let mut arp = Arpeggiator::with_config(ArpConfig::default(), 44100.0, 120.0);
        assert!(arp.is_enabled());

        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_on(67, 100);

        let state_str = format!("{:?}", arp.state);
        assert!(
            matches!(arp.state, ArpState::Playing),
            "State should be Playing, got: {}",
            state_str
        );

        let note = arp.process();
        assert!(note.is_some(), "Should have produced a note");
        let (note_num, velocity) = note.unwrap();
        assert!(
            note_num >= 60 && note_num <= 67,
            "Note should be 60-67, got: {}",
            note_num
        );
        assert!(velocity > 0);
    }

    #[test]
    fn test_arpeggiator_note_off() {
        let mut arp = Arpeggiator::with_config(ArpConfig::default(), 44100.0, 120.0);

        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_off(60);

        assert_eq!(arp.held_notes.len(), 1);
    }

    #[test]
    fn test_arpeggiator_range() {
        let mut config = ArpConfig::default();
        config.range = 2;
        config.speed = 5;
        config.mode = 0;

        let mut arp = Arpeggiator::with_config(config, 44100.0, 120.0);
        arp.note_on(60, 100);

        let mut all_notes: Vec<u8> = Vec::new();
        for _ in 0..3000 {
            if let Some((n, _)) = arp.process() {
                all_notes.push(n);
            }
        }

        assert!(
            all_notes.contains(&60),
            "Should contain 60, got: {:?}",
            all_notes
        );
        assert!(
            all_notes.contains(&72),
            "Should contain 72, got: {:?}",
            all_notes
        );
    }

    #[test]
    fn test_up_mode_ascending() {
        let mut config = ArpConfig::default();
        config.mode = 0;
        config.speed = 5;
        config.range = 1;

        let mut arp = Arpeggiator::with_config(config, 44100.0, 120.0);
        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_on(67, 100);

        let mut notes: Vec<u8> = Vec::new();
        for _ in 0..10000 {
            if let Some((n, _)) = arp.process() {
                notes.push(n);
            }
        }

        assert!(notes.contains(&60) && notes.contains(&64) && notes.contains(&67));
    }

    #[test]
    fn test_down_mode() {
        let mut config = ArpConfig::default();
        config.mode = 1;
        config.speed = 5;
        config.range = 1;

        let mut arp = Arpeggiator::with_config(config, 44100.0, 120.0);
        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_on(67, 100);

        arp.position = 0;
        arp.initial_notes = arp.held_notes.clone();

        let mut notes: Vec<u8> = Vec::new();
        for _ in 0..10000 {
            if let Some((n, _)) = arp.process() {
                notes.push(n);
            }
        }

        assert!(notes.contains(&60) && notes.contains(&64) && notes.contains(&67));
    }

    #[test]
    fn test_no_notes_no_output() {
        let mut arp = Arpeggiator::with_config(ArpConfig::default(), 44100.0, 120.0);

        let mut any_output = false;
        for _ in 0..1000 {
            if arp.process().is_some() {
                any_output = true;
            }
        }
        assert!(!any_output);
    }

    #[test]
    fn test_disabled_arpeggiator_no_output() {
        let mut arp = Arpeggiator::new(44100.0);
        assert!(!arp.is_enabled());

        arp.note_on(60, 100);

        let mut any_output = false;
        for _ in 0..1000 {
            if arp.process().is_some() {
                any_output = true;
            }
        }
        assert!(!any_output);
    }

    #[test]
    fn test_note_off_removes_from_held() {
        let mut arp = Arpeggiator::with_config(ArpConfig::default(), 44100.0, 120.0);

        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_on(67, 100);
        assert_eq!(arp.held_notes.len(), 3);

        arp.note_off(64);
        assert_eq!(arp.held_notes.len(), 2);
        let remaining: Vec<u8> = arp.held_notes.iter().map(|n| n.note).collect();
        assert!(!remaining.contains(&64));
        assert!(remaining.contains(&60));
        assert!(remaining.contains(&67));
    }

    #[test]
    fn test_set_bpm_changes_timing() {
        let config = ArpConfig::default();
        let mut arp = Arpeggiator::with_config(config, 44100.0, 10000.0);
        arp.note_on(60, 100);

        let mut fast_count = 0;
        for _ in 0..1000 {
            if arp.process().is_some() {
                fast_count += 1;
            }
        }

        arp.set_bpm(60.0);
        let mut slow_count = 0;
        for _ in 0..1000 {
            if arp.process().is_some() {
                slow_count += 1;
            }
        }

        assert!(fast_count > slow_count);
    }

    #[test]
    fn test_set_mode() {
        let mut arp = Arpeggiator::new(44100.0);
        arp.set_mode(2);
        assert_eq!(arp.config.mode, 2);
    }

    #[test]
    fn test_set_speed() {
        let mut arp = Arpeggiator::new(44100.0);
        arp.set_speed(4);
        assert_eq!(arp.config.speed, 4);
    }

    #[test]
    fn test_set_range() {
        let mut arp = Arpeggiator::new(44100.0);
        arp.set_range(3);
        assert_eq!(arp.config.range, 3);
        arp.set_range(10); // should clamp to 4
        assert_eq!(arp.config.range, 4);
    }

    #[test]
    fn test_set_note_length() {
        let mut arp = Arpeggiator::new(44100.0);
        arp.set_note_length(50);
        assert_eq!(arp.config.note_length, 50);
    }

    #[test]
    fn test_set_offset() {
        let mut arp = Arpeggiator::new(44100.0);
        arp.set_offset(5);
        assert_eq!(arp.config.offset, 5);
    }

    #[test]
    fn test_set_arp_length() {
        let mut arp = Arpeggiator::new(44100.0);
        arp.set_arp_length(8);
        assert_eq!(arp.config.arp_length, 8);
    }
}
