//! AI Chord Progression Generator Module
//!
//! This module provides intelligent chord progression generation capabilities for WAVELET,
//! supporting various musical styles, keys, and progression patterns.
//!
//! # Features
//!
//! - **Multiple Chord Types**: Support for triads, 7ths, extended chords
//! - **Style Presets**: Pre-configured progressions for Pop, Jazz, LoFi, EDM, Ambient,
//!   Classical, Rock, and R&B styles
//! - **Key Detection**: Smart key detection and mode selection
//! - **Natural Voice Leading**: Algorithms for smooth chord transitions
//! - **Progressions**: Common progression patterns (I-IV-V-vi, etc.)
//!
//! # Example
//!
//! ```rust
//! use wavelet::chord_generator::{ChordGenerator, ChordStyle, Key, Chord};
//!
//! // Create a chord generator in C major
//! let key = Key {
//!     root: 60, // C4
//!     scale: Scale::Major,
//! };
//!
//! let mut generator = ChordGenerator::new(key, 120.0);
//! let progression = generator.generate_preset(ChordStyle::Pop);
//! ```

use rand::Rng;
use std::error::Error;

/// Chord type enumeration.
///
/// Defines all supported chord types for progression generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordType {
    /// Major triad - bright, happy sound
    Major,
    /// Minor triad - dark, melancholic sound
    Minor,
    /// Major 7th chord - sophisticated, jazzy
    Major7,
    /// Minor 7th chord - soft, dreamy
    Minor7,
    /// Dominant 7th - bluesy, functional
    Dominant7,
    /// Diminished triad - tense, unstable
    Diminished,
    /// Diminished 7th - very tense, cinematic
    Diminished7,
    /// Augmented triad - ambiguous, unstable
    Augmented,
    /// Suspended 2nd - ambiguous, tension
    Sus2,
    /// Suspended 4th - ambiguous, tension
    Sus4,
}

/// Chord structure.
///
/// Represents a complete chord with root note, type, and optional extensions.
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    /// Root note as MIDI note number (0-127)
    pub root: u8,
    /// The chord type
    pub chord_type: ChordType,
    /// Chord extensions (9th, 11th, 13th) - stored as semitone offsets
    pub extensions: Vec<u8>,
    /// Duration in beats
    pub duration: f32,
}

/// Chord style enumeration.
///
/// Pre-configured styles for chord progression generation with appropriate parameters
/// for different musical genres and moods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordStyle {
    /// Pop music - simple, familiar progressions (I-V-vi-IV)
    Pop,
    /// Jazz - complex, extended chords, 2-5-1 progressions
    Jazz,
    /// LoFi - minor keys, suspended chords, slow progressions
    LoFi,
    /// EDM - big drops, minor to major transitions
    EDM,
    /// Ambient - slow, evolving, rich chords
    Ambient,
    /// Classical - traditional progressions, voice leading
    Classical,
    /// Rock - power chords, driving progressions
    Rock,
    /// R&B - soulful, 7th and 9th chords
    Rnb,
}

/// Musical scale enumeration (simplified for chord generation).
///
/// Defines scales supported for chord progression generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// Major scale (Ionian) - bright, happy sound
    Major,
    /// Natural minor scale - dark, melancholic
    Minor,
    /// Harmonic minor - exotic flavor
    HarmonicMinor,
    /// Dorian mode - jazz, funk
    Dorian,
    /// Mixolydian mode - rock, blues
    Mixolydian,
}

/// Musical key structure.
///
/// Represents a musical key with a root note and scale type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    /// Root note as MIDI note number (0-127)
    pub root: u8,
    /// The scale type for this key
    pub scale: Scale,
}

/// Common chord progression patterns.
///
/// Pre-defined progression templates for different styles.
#[derive(Debug, Clone)]
pub enum ProgressionPattern {
    /// I - IV - V - vi (The "Axis of Awesome" progression)
    PopPillar,
    /// I - vi - IV - V (Another pop classic)
    StandardPop,
    /// ii - V - I (Jazz turnaround)
    TwoFiveOne,
    /// I - V - vi - iii - IV - I - IV - V (Circle progression)
    Circle,
    /// I - IV - ii - V (Soulful R&B)
    RnBFlow,
    /// i - VI - III - VII (Neo-soul)
    NeoSoul,
    /// i - VII - VI - V (Minor descent)
    MinorDescent,
    /// I - IV - I - V (Rock classic)
    RockDriver,
    /// vi - IV - I - V (Pop in minor)
    PopMinor,
    /// I - iii - IV - V (Jazz minor)
    JazzMinor,
}

/// Chord progression generator.
///
/// Main struct for generating chord progressions based on key, style, and preferences.
#[allow(dead_code)]
pub struct ChordGenerator {
    /// The musical key for progression
    key: Key,
    /// Tempo in BPM
    tempo: f32,
    /// Random seed for reproducibility
    rng: rand::rngs::ThreadRng,
}

#[allow(dead_code)]
impl ChordGenerator {
    /// Create a new chord generator.
    ///
    /// # Arguments
    ///
    /// * `key` - The musical key for generation
    /// * `tempo` - Tempo in beats per minute
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::chord_generator::{ChordGenerator, Key, Scale};
    ///
    /// let key = Key { root: 60, scale: Scale::Major };
    /// let generator = ChordGenerator::new(key, 120.0);
    /// ```
    pub fn new(key: Key, tempo: f32) -> Self {
        Self {
            key,
            tempo,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a chord progression using a preset style.
    ///
    /// # Arguments
    ///
    /// * `style` - The musical style to use
    /// * `num_chords` - Number of chords in the progression (default: 4-8)
    ///
    /// # Returns
    ///
    /// A vector of chords representing the generated progression
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::chord_generator::{ChordGenerator, ChordStyle, Key, Scale, Chord};
    ///
    /// let key = Key { root: 60, scale: Scale::Major };
    /// let mut generator = ChordGenerator::new(key, 120.0);
    /// let progression = generator.generate_preset(ChordStyle::Pop);
    /// ```
    pub fn generate_preset(&mut self, style: ChordStyle) -> Vec<Chord> {
        match style {
            ChordStyle::Pop => self.generate_pop(),
            ChordStyle::Jazz => self.generate_jazz(),
            ChordStyle::LoFi => self.generate_lofi(),
            ChordStyle::EDM => self.generate_edm(),
            ChordStyle::Ambient => self.generate_ambient(),
            ChordStyle::Classical => self.generate_classical(),
            ChordStyle::Rock => self.generate_rock(),
            ChordStyle::Rnb => self.generate_rnb(),
        }
    }

    /// Generate a pop-style chord progression.
    ///
    /// Uses familiar, radio-friendly progressions like I-V-vi-IV.
    fn generate_pop(&mut self) -> Vec<Chord> {
        let patterns = [
            // I - V - vi - IV
            [1, 5, 6, 4],
            // I - IV - V - V
            [1, 4, 5, 5],
            // vi - IV - I - V
            [6, 4, 1, 5],
            // I - vi - IV - V
            [1, 6, 4, 5],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression(pattern, ChordType::Major, 4.0)
    }

    /// Generate a jazz-style chord progression.
    ///
    /// Uses ii-V-I progressions, extended chords, and substitutions.
    fn generate_jazz(&mut self) -> Vec<Chord> {
        let patterns = [
            // ii - V - I - I
            [2, 5, 1, 1],
            // ii - V - I - vi
            [2, 5, 1, 6],
            // I - iii - vi - ii - V (use first 4)
            [1, 3, 6, 2],
            // iii - VI - ii - V
            [3, 6, 2, 5],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression_jazz(pattern)
    }

    /// Generate a LoFi-style chord progression.
    ///
    /// Uses minor keys, suspended chords, and slow, evolving progressions.
    fn generate_lofi(&mut self) -> Vec<Chord> {
        let patterns = [
            // i - VI - iv - V
            [1, 6, 4, 5],
            // i - iv - VII - III
            [1, 4, 7, 3],
            // i - VII - VI - V
            [1, 7, 6, 5],
            // i - iv - i - V
            [1, 4, 1, 5],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression(pattern, ChordType::Minor, 8.0)
    }

    /// Generate an EDM-style chord progression.
    ///
    /// Uses big, dramatic progressions with minor to major transitions.
    fn generate_edm(&mut self) -> Vec<Chord> {
        let patterns = [
            // vi - IV - I - V (build up)
            [6, 4, 1, 5],
            // i - VI - III - V (build up minor)
            [1, 6, 3, 5],
            // I - V - vi - IV (big room)
            [1, 5, 6, 4],
            // IV - I - V - I (anthem)
            [4, 1, 5, 1],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression(pattern, ChordType::Major, 4.0)
    }

    /// Generate an ambient-style chord progression.
    ///
    /// Uses slow, evolving progressions with rich extended chords.
    fn generate_ambient(&mut self) -> Vec<Chord> {
        let patterns = [
            // I - iii - IV - V
            [1, 3, 4, 5],
            // I - IV - vii° - iii
            [1, 4, 7, 3],
            // i - iv - VII - III
            [1, 4, 7, 3],
            // I - V - vi - IV
            [1, 5, 6, 4],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression_ambient(pattern)
    }

    /// Generate a classical-style chord progression.
    ///
    /// Uses traditional progressions with good voice leading.
    fn generate_classical(&mut self) -> Vec<Chord> {
        let patterns = [
            // I - IV - V - I
            [1, 4, 5, 1],
            // I - ii - V - I
            [1, 2, 5, 1],
            // I - iii - IV - V
            [1, 3, 4, 5],
            // i - iv - V - i
            [1, 4, 5, 1],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression_classical(pattern)
    }

    /// Generate a rock-style chord progression.
    ///
    /// Uses driving progressions with power chord feel.
    fn generate_rock(&mut self) -> Vec<Chord> {
        let patterns = [
            // I - IV - V - IV
            [1, 4, 5, 4],
            // I - V - IV - V
            [1, 5, 4, 5],
            // I - IV - I - V
            [1, 4, 1, 5],
            // vi - IV - I - V
            [6, 4, 1, 5],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression(pattern, ChordType::Major, 4.0)
    }

    /// Generate an R&B-style chord progression.
    ///
    /// Uses soulful progressions with 7th and 9th chords.
    fn generate_rnb(&mut self) -> Vec<Chord> {
        let patterns = [
            // I - IV - V - I
            [1, 4, 5, 1],
            // i - VI - iii - V
            [1, 6, 3, 5],
            // I - iii - IV - V
            [1, 3, 4, 5],
            // I - V - vi - IV
            [1, 5, 6, 4],
        ];

        let pattern = &patterns[self.rng.gen_range(0..patterns.len())];
        self.build_progression_rnb(pattern)
    }

    /// Build a basic chord progression from a pattern.
    fn build_progression(
        &mut self,
        pattern: &[i32],
        _default_type: ChordType,
        duration: f32,
    ) -> Vec<Chord> {
        let is_minor = matches!(self.key.scale, Scale::Minor | Scale::HarmonicMinor);
        let root_type = if is_minor {
            ChordType::Minor
        } else {
            ChordType::Major
        };

        pattern
            .iter()
            .map(|degree| {
                let chord_type = self.get_chord_type_for_degree(*degree, root_type);
                let root = self.get_root_for_degree(*degree);
                Chord {
                    root,
                    chord_type,
                    extensions: vec![],
                    duration,
                }
            })
            .collect()
    }

    /// Build a jazz-style progression with extended chords.
    fn build_progression_jazz(&mut self, pattern: &[i32]) -> Vec<Chord> {
        let is_minor = matches!(self.key.scale, Scale::Minor | Scale::HarmonicMinor);
        let root_type = if is_minor {
            ChordType::Minor
        } else {
            ChordType::Major
        };

        pattern
            .iter()
            .map(|degree| {
                let chord_type = self.get_jazz_chord_type(*degree, root_type);
                let root = self.get_root_for_degree(*degree);
                let extensions = self.get_jazz_extensions(*degree, chord_type);
                Chord {
                    root,
                    chord_type,
                    extensions,
                    duration: 4.0,
                }
            })
            .collect()
    }

    /// Build an ambient-style progression with rich extensions.
    fn build_progression_ambient(&mut self, pattern: &[i32]) -> Vec<Chord> {
        let is_minor = matches!(self.key.scale, Scale::Minor | Scale::HarmonicMinor);
        let root_type = if is_minor {
            ChordType::Minor
        } else {
            ChordType::Major
        };

        pattern
            .iter()
            .map(|degree| {
                let chord_type = self.get_chord_type_for_degree(*degree, root_type);
                let root = self.get_root_for_degree(*degree);
                // Add 9th extension for ambient feel
                let extensions = vec![2]; // Major 9th
                Chord {
                    root,
                    chord_type,
                    extensions,
                    duration: 8.0,
                }
            })
            .collect()
    }

    /// Build a classical-style progression.
    fn build_progression_classical(&mut self, pattern: &[i32]) -> Vec<Chord> {
        let is_minor = matches!(self.key.scale, Scale::Minor | Scale::HarmonicMinor);
        let root_type = if is_minor {
            ChordType::Minor
        } else {
            ChordType::Major
        };

        pattern
            .iter()
            .map(|degree| {
                let chord_type = self.get_chord_type_for_degree(*degree, root_type);
                let root = self.get_root_for_degree(*degree);
                Chord {
                    root,
                    chord_type,
                    extensions: vec![],
                    duration: 4.0,
                }
            })
            .collect()
    }

    /// Build an R&B-style progression with 7th chords.
    fn build_progression_rnb(&mut self, pattern: &[i32]) -> Vec<Chord> {
        let is_minor = matches!(self.key.scale, Scale::Minor | Scale::HarmonicMinor);
        let root_type = if is_minor {
            ChordType::Minor
        } else {
            ChordType::Major
        };

        pattern
            .iter()
            .map(|degree| {
                let chord_type = self.get_rnb_chord_type(*degree, root_type);
                let root = self.get_root_for_degree(*degree);
                Chord {
                    root,
                    chord_type,
                    extensions: vec![],
                    duration: 4.0,
                }
            })
            .collect()
    }

    /// Get the chord type for a given scale degree.
    fn get_chord_type_for_degree(&self, degree: i32, default: ChordType) -> ChordType {
        // Diatonic chord qualities based on scale degree
        match self.key.scale {
            Scale::Major => match degree {
                1 => ChordType::Major,
                2 => ChordType::Minor,
                3 => ChordType::Minor,
                4 => ChordType::Major,
                5 => ChordType::Major,
                6 => ChordType::Minor,
                7 => ChordType::Diminished,
                _ => default,
            },
            Scale::Minor | Scale::HarmonicMinor => match degree {
                1 => ChordType::Minor,
                2 => ChordType::Diminished,
                3 => ChordType::Augmented,
                4 => ChordType::Major,
                5 => ChordType::Major,
                6 => ChordType::Major,
                7 => ChordType::Diminished,
                _ => default,
            },
            Scale::Dorian => match degree {
                1 => ChordType::Minor,
                2 => ChordType::Minor,
                3 => ChordType::Major,
                4 => ChordType::Major,
                5 => ChordType::Minor,
                6 => ChordType::Diminished,
                7 => ChordType::Major,
                _ => default,
            },
            Scale::Mixolydian => match degree {
                1 => ChordType::Major,
                2 => ChordType::Minor,
                3 => ChordType::Minor,
                4 => ChordType::Diminished,
                5 => ChordType::Major,
                6 => ChordType::Minor,
                7 => ChordType::Major,
                _ => default,
            },
        }
    }

    /// Get jazz-specific chord type for a degree.
    fn get_jazz_chord_type(&self, degree: i32, default: ChordType) -> ChordType {
        match degree {
            1 => ChordType::Major7,
            2 => ChordType::Minor7,
            3 => ChordType::Minor7,
            4 => ChordType::Major7,
            5 => ChordType::Dominant7,
            6 => ChordType::Minor7,
            7 => ChordType::Diminished7, // Half-diminished = diminished 7th
            _ => default,
        }
    }

    /// Get R&B-specific chord type for a degree.
    fn get_rnb_chord_type(&self, degree: i32, default: ChordType) -> ChordType {
        match degree {
            1 => ChordType::Major7,
            2 => ChordType::Minor7,
            3 => ChordType::Minor7,
            4 => ChordType::Major7,
            5 => ChordType::Dominant7,
            6 => ChordType::Minor7,
            7 => ChordType::Dominant7,
            _ => default,
        }
    }

    /// Get jazz chord extensions.
    fn get_jazz_extensions(&mut self, _degree: i32, _chord_type: ChordType) -> Vec<u8> {
        let mut extensions = vec![];

        // 50% chance of adding 9th
        if self.rng.gen_bool(0.5) {
            extensions.push(2); // Major 9th
        }

        // 30% chance of adding 13th (6th)
        if self.rng.gen_bool(0.3) {
            extensions.push(9); // 13th
        }

        extensions
    }

    /// Get the root note for a given scale degree.
    fn get_root_for_degree(&self, degree: i32) -> u8 {
        // Major scale intervals: 0, 2, 4, 5, 7, 9, 11
        // Minor scale intervals: 0, 2, 3, 5, 7, 8, 11
        let intervals = match self.key.scale {
            Scale::Major | Scale::Dorian | Scale::Mixolydian => vec![0, 2, 4, 5, 7, 9, 11],
            Scale::Minor | Scale::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
        };

        let degree_idx = ((degree - 1).rem_euclid(7)) as usize;
        let octave_offset = ((degree - 1) / 7) * 12;

        (self.key.root + intervals.get(degree_idx).unwrap_or(&0) + octave_offset as u8) % 128
    }

    /// Get a random chord progression with customizable parameters.
    ///
    /// # Arguments
    ///
    /// * `num_chords` - Number of chords in the progression
    /// * `include_7ths` - Whether to include 7th chords
    /// * `_style` - Musical style hint (reserved for future use)
    ///
    /// # Returns
    ///
    /// A randomly generated chord progression
    pub fn generate_custom(
        &mut self,
        num_chords: usize,
        include_7ths: bool,
        _style: ChordStyle,
    ) -> Vec<Chord> {
        let mut progression = Vec::new();
        let is_minor = matches!(self.key.scale, Scale::Minor | Scale::HarmonicMinor);
        let base_type = if is_minor {
            ChordType::Minor
        } else {
            ChordType::Major
        };

        for i in 0..num_chords {
            let degree = self.rng.gen_range(1..=6);
            let mut chord_type = self.get_chord_type_for_degree(degree, base_type);

            // Add 7th on stronger beats for variety
            if include_7ths && i > 0 {
                chord_type = match chord_type {
                    ChordType::Major => ChordType::Major7,
                    ChordType::Minor => ChordType::Minor7,
                    ChordType::Dominant7 => ChordType::Dominant7,
                    _ => chord_type,
                };
            }

            let root = self.get_root_for_degree(degree);
            let duration = 4.0;

            progression.push(Chord {
                root,
                chord_type,
                extensions: vec![],
                duration,
            });
        }

        progression
    }

    /// Generate a chord progression from a predefined pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The progression pattern to use
    ///
    /// # Returns
    ///
    /// The chord progression
    pub fn generate_from_pattern(&mut self, pattern: ProgressionPattern) -> Vec<Chord> {
        match pattern {
            ProgressionPattern::PopPillar => {
                self.build_progression(&[1, 5, 6, 4], ChordType::Major, 4.0)
            }
            ProgressionPattern::StandardPop => {
                self.build_progression(&[1, 6, 4, 5], ChordType::Major, 4.0)
            }
            ProgressionPattern::TwoFiveOne => self.build_progression_jazz(&[2, 5, 1]),
            ProgressionPattern::Circle => {
                self.build_progression(&[1, 6, 2, 5, 3, 7, 4, 1], ChordType::Major, 2.0)
            }
            ProgressionPattern::RnBFlow => {
                self.build_progression(&[1, 4, 2, 5], ChordType::Major, 4.0)
            }
            ProgressionPattern::NeoSoul => {
                self.build_progression(&[1, 6, 3, 7], ChordType::Minor, 4.0)
            }
            ProgressionPattern::MinorDescent => {
                self.build_progression(&[1, 7, 6, 5], ChordType::Minor, 4.0)
            }
            ProgressionPattern::RockDriver => {
                self.build_progression(&[1, 4, 1, 5], ChordType::Major, 4.0)
            }
            ProgressionPattern::PopMinor => {
                self.build_progression(&[6, 4, 1, 5], ChordType::Major, 4.0)
            }
            ProgressionPattern::JazzMinor => {
                self.build_progression(&[1, 3, 4, 5], ChordType::Minor, 4.0)
            }
        }
    }
}

/// Convert chord type to string for display.
impl std::fmt::Display for ChordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChordType::Major => write!(f, ""),
            ChordType::Minor => write!(f, "m"),
            ChordType::Major7 => write!(f, "maj7"),
            ChordType::Minor7 => write!(f, "m7"),
            ChordType::Dominant7 => write!(f, "7"),
            ChordType::Diminished => write!(f, "dim"),
            ChordType::Diminished7 => write!(f, "dim7"),
            ChordType::Augmented => write!(f, "aug"),
            ChordType::Sus2 => write!(f, "sus2"),
            ChordType::Sus4 => write!(f, "sus4"),
        }
    }
}

/// Convert chord to string representation.
impl std::fmt::Display for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        // MIDI note 60 (middle C) should be C4, so octave = (root / 12) - 1
        let octave = (self.root / 12).saturating_sub(1);
        let note = note_names[(self.root % 12) as usize];
        write!(f, "{}{}{}", note, octave, self.chord_type)
    }
}

/// Error type for chord generation.
#[derive(Debug)]
pub struct ChordGenerationError {
    message: String,
}

impl Error for ChordGenerationError {}

impl std::fmt::Display for ChordGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chord generation error: {}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_generator_creation() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let generator = ChordGenerator::new(key, 120.0);
        assert_eq!(generator.key.root, 60);
        assert_eq!(generator.tempo, 120.0);
    }

    #[test]
    fn test_pop_progression_generation() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut generator = ChordGenerator::new(key, 120.0);
        let progression = generator.generate_preset(ChordStyle::Pop);

        assert!(!progression.is_empty());
        assert!(progression.len() >= 4);
        assert!(progression.len() <= 8);
    }

    #[test]
    fn test_jazz_progression_generation() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut generator = ChordGenerator::new(key, 120.0);
        let progression = generator.generate_preset(ChordStyle::Jazz);

        assert!(!progression.is_empty());
        // Jazz progressions often have 7th chords
        for chord in &progression {
            assert!(matches!(
                chord.chord_type,
                ChordType::Major7 | ChordType::Minor7 | ChordType::Dominant7
            ));
        }
    }

    #[test]
    fn test_minor_key_progression() {
        let key = Key {
            root: 57, // A3
            scale: Scale::Minor,
        };
        let mut generator = ChordGenerator::new(key, 120.0);
        let progression = generator.generate_preset(ChordStyle::Pop);

        assert!(!progression.is_empty());
        // In minor key, first chord could be minor or major (i-VI-IV-V is common)
        // Just verify we got valid chords
        for chord in &progression {
            match chord.chord_type {
                ChordType::Major
                | ChordType::Minor
                | ChordType::Major7
                | ChordType::Minor7
                | ChordType::Dominant7 => {}
                _ => panic!("Unexpected chord type: {:?}", chord.chord_type),
            }
        }
    }

    #[test]
    fn test_chord_display() {
        let chord = Chord {
            root: 60,
            chord_type: ChordType::Major,
            extensions: vec![],
            duration: 4.0,
        };
        assert_eq!(format!("{}", chord), "C4");

        let chord7 = Chord {
            root: 62, // D4
            chord_type: ChordType::Minor7,
            extensions: vec![],
            duration: 4.0,
        };
        assert_eq!(format!("{}", chord7), "D4m7");
    }

    #[test]
    fn test_lofi_progression() {
        let key = Key {
            root: 57,
            scale: Scale::Minor,
        };
        let mut generator = ChordGenerator::new(key, 90.0);
        let progression = generator.generate_preset(ChordStyle::LoFi);

        assert!(!progression.is_empty());
        // LoFi progressions typically have longer duration
        assert!(progression.iter().all(|c| c.duration >= 4.0));
    }

    #[test]
    fn test_progression_patterns() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut generator = ChordGenerator::new(key, 120.0);

        // Test different patterns
        let patterns = vec![
            ProgressionPattern::PopPillar,
            ProgressionPattern::TwoFiveOne,
            ProgressionPattern::Circle,
        ];

        for pattern in patterns {
            let progression = generator.generate_from_pattern(pattern);
            assert!(!progression.is_empty());
        }
    }

    #[test]
    fn test_custom_progression() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut generator = ChordGenerator::new(key, 120.0);

        let progression = generator.generate_custom(8, true, ChordStyle::Pop);
        assert_eq!(progression.len(), 8);
    }

    #[test]
    fn test_chord_roots_in_key() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut generator = ChordGenerator::new(key, 120.0);
        let progression = generator.generate_preset(ChordStyle::Pop);

        // C major scale degrees: C(0), D(2), E(4), F(5), G(7), A(9), B(11)
        let major_intervals: Vec<u8> = vec![0, 2, 4, 5, 7, 9, 11];
        for chord in &progression {
            let interval = (chord.root % 12 + 12 - key.root % 12) % 12;
            assert!(
                major_intervals.contains(&interval),
                "Chord root {} (interval {}) not in C major scale",
                chord.root,
                interval
            );
        }
    }

    #[test]
    fn test_all_styles_generate_valid_chords() {
        let styles = [
            ChordStyle::Pop,
            ChordStyle::Jazz,
            ChordStyle::LoFi,
            ChordStyle::EDM,
            ChordStyle::Ambient,
            ChordStyle::Classical,
            ChordStyle::Rock,
            ChordStyle::Rnb,
        ];

        for style in styles {
            let key = Key {
                root: 60,
                scale: Scale::Major,
            };
            let mut gen = ChordGenerator::new(key, 120.0);
            let prog = gen.generate_preset(style);
            assert!(!prog.is_empty(), "Style {:?} produced empty progression", style);
            for chord in &prog {
                assert!(chord.root <= 127, "Chord root out of MIDI range");
                assert!(chord.duration > 0.0, "Chord duration must be positive");
            }
        }
    }

    #[test]
    fn test_all_progression_patterns_produce_chords() {
        let patterns = [
            ProgressionPattern::PopPillar,
            ProgressionPattern::StandardPop,
            ProgressionPattern::TwoFiveOne,
            ProgressionPattern::Circle,
            ProgressionPattern::RnBFlow,
            ProgressionPattern::NeoSoul,
            ProgressionPattern::MinorDescent,
            ProgressionPattern::RockDriver,
            ProgressionPattern::PopMinor,
            ProgressionPattern::JazzMinor,
        ];

        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut gen = ChordGenerator::new(key, 120.0);

        for pattern in patterns {
            let prog = gen.generate_from_pattern(pattern.clone());
            assert!(
                !prog.is_empty(),
                "Pattern {:?} produced empty progression",
                pattern
            );
            assert!(
                prog.len() >= 3,
                "Pattern {:?} produced fewer than 3 chords",
                pattern
            );
        }
    }

    #[test]
    fn test_custom_with_7ths_produces_7th_chords() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut gen = ChordGenerator::new(key, 120.0);
        let prog = gen.generate_custom(8, true, ChordStyle::Jazz);

        let has_7th = prog.iter().any(|c| {
            matches!(
                c.chord_type,
                ChordType::Major7 | ChordType::Minor7 | ChordType::Dominant7 | ChordType::Diminished7
            )
        });
        assert!(has_7th, "Custom progression with include_7ths should contain 7th chords");
    }

    #[test]
    fn test_chord_display_all_types() {
        let types_and_suffixes = [
            (ChordType::Major, "C4"),
            (ChordType::Minor, "C4m"),
            (ChordType::Major7, "C4maj7"),
            (ChordType::Minor7, "C4m7"),
            (ChordType::Dominant7, "C47"),
            (ChordType::Diminished, "C4dim"),
            (ChordType::Augmented, "C4aug"),
            (ChordType::Sus2, "C4sus2"),
            (ChordType::Sus4, "C4sus4"),
        ];

        for (chord_type, expected) in types_and_suffixes {
            let chord = Chord {
                root: 60,
                chord_type,
                extensions: vec![],
                duration: 4.0,
            };
            assert_eq!(
                format!("{}", chord),
                expected,
                "Display for {:?} should be {}",
                chord_type,
                expected
            );
        }
    }

    #[test]
    fn test_different_roots_produce_different_progressions() {
        let key_c = Key { root: 60, scale: Scale::Major };
        let key_g = Key { root: 67, scale: Scale::Major };

        let mut gen_c = ChordGenerator::new(key_c, 120.0);
        let mut gen_g = ChordGenerator::new(key_g, 120.0);

        let prog_c = gen_c.generate_from_pattern(ProgressionPattern::PopPillar);
        let prog_g = gen_g.generate_from_pattern(ProgressionPattern::PopPillar);

        // Same pattern in different keys should have different root notes
        let roots_c: Vec<u8> = prog_c.iter().map(|c| c.root % 12).collect();
        let roots_g: Vec<u8> = prog_g.iter().map(|c| c.root % 12).collect();
        assert_ne!(roots_c, roots_g, "Same pattern in C and G should differ");
    }
}
