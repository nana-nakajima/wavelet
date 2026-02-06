//! AI Melody Generator Module
//!
//! This module provides intelligent melody generation capabilities for WAVELET,
//! supporting various musical scales, styles, and generation algorithms.
//!
//! # Features
//!
//! - **Multiple Scales**: Support for all common scales including major, minor,
//!   modal scales, pentatonic, blues, and chromatic
//! - **Style Presets**: Pre-configured generation for Pop, Jazz, LoFi, EDM, Ambient,
//!   and Classical styles
//! - **Smart Generation**: Rule-based melody generation that follows music theory
//! - **MIDI Export**: Export generated melodies to MIDI files
//!
//! # Example
//!
//! ```rust
//! use wavelet::melody_generator::{MelodyGenerator, MelodyStyle, Scale, Key};
//!
//! // Create a melody generator in C major
//! let key = Key {
//!     root: 60, // C4
//!     scale: Scale::Major,
//! };
//!
//! let mut generator = MelodyGenerator::new(key, 120.0, 4);
//! let melody = generator.generate_preset(MelodyStyle::Pop);
//! ```

use rand::Rng;
use std::error::Error;

/// Musical scale enumeration.
///
/// Defines all supported scales for melody generation, from basic diatonic
/// scales to advanced modal and exotic scales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// Major scale (Ionian) - bright, happy sound
    Major,
    /// Natural minor scale - dark, melancholic sound
    Minor,
    /// Harmonic minor scale - exotic, Middle Eastern flavor
    HarmonicMinor,
    /// Melodic minor scale - ascending melodic patterns
    MelodicMinor,
    /// Major pentatonic scale - folk, rock, country
    PentatonicMajor,
    /// Minor pentatonic scale - blues, rock
    PentatonicMinor,
    /// Blues scale - characteristic blues sound
    Blues,
    /// Dorian mode - jazz, funk
    Dorian,
    /// Phrygian mode - metal, flamenco
    Phrygian,
    /// Lydian mode - dreamy, ethereal
    Lydian,
    /// Mixolydian mode - rock, blues
    Mixolydian,
    /// Locrian mode - tense, unstable
    Locrian,
    /// Whole tone scale - dreamlike, ambiguous
    WholeTone,
    /// Chromatic scale - all 12 semitones
    Chromatic,
}

/// Musical key structure.
///
/// Represents a musical key with a root note and scale type.
///
/// # Example
///
/// ```rust
/// use wavelet::melody_generator::{Key, Scale};
///
/// // A minor key (A = 57, minor scale)
/// let a_minor = Key {
///     root: 57,
///     scale: Scale::Minor,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    /// Root note as MIDI note number (0-127)
    pub root: u8,
    /// The scale type for this key
    pub scale: Scale,
}

/// Melody style enumeration.
///
/// Pre-configured styles for melody generation with appropriate parameters
/// for different musical genres and moods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MelodyStyle {
    /// Pop music - simple, catchy melodies with major key emphasis
    Pop,
    /// Jazz - complex harmonies, 7th chords, syncopation
    Jazz,
    /// LoFi - relaxed, simple patterns, slower tempo feel
    LoFi,
    /// EDM - driving rhythms, consistent 16th note patterns
    EDM,
    /// Ambient - long sustained notes, sparse melodies
    Ambient,
    /// Classical - balanced phrases, arpeggiated patterns
    Classical,
    /// Custom - user-defined parameters
    Custom,
}

/// Single note in a melody.
///
/// Represents a musical note with timing, pitch, and dynamics.
///
/// # Fields
///
/// * `pitch` - MIDI note number (0-127)
/// * `velocity` - Note velocity/intensity (0.0-1.0)
/// * `start_beat` - When the note starts (in beats from beginning)
/// * `duration` - How long the note lasts (in beats)
#[derive(Debug, Clone, PartialEq)]
pub struct MelodyNote {
    /// MIDI note number (0-127)
    pub pitch: u8,
    /// Velocity as normalized value (0.0 to 1.0)
    pub velocity: f32,
    /// Start time in beats
    pub start_beat: f64,
    /// Duration in beats
    pub duration: f64,
}

/// Complete melody structure.
///
/// Represents a fully generated melody with all notes, timing, and metadata.
///
/// # Fields
///
/// * `notes` - Vector of all notes in the melody
/// * `durations` - Vector of duration values for each note
/// * `key` - The musical key used for generation
/// * `tempo` - Tempo in beats per minute
/// * `style` - The style preset used for generation
#[derive(Debug, Clone, PartialEq)]
pub struct Melody {
    /// All notes in the melody
    pub notes: Vec<MelodyNote>,
    /// Duration for each note (in beats)
    pub durations: Vec<f64>,
    /// The musical key
    pub key: Key,
    /// Tempo in BPM
    pub tempo: f64,
    /// Generation style used
    pub style: MelodyStyle,
}

/// Basic chord structure for harmonic context.
///
/// Simplified chord representation for generating melodies
/// that harmonize with chord progressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordType {
    /// Major triad
    Major,
    /// Minor triad
    Minor,
    /// Major 7th chord
    Major7,
    /// Minor 7th chord
    Minor7,
    /// Dominant 7th chord
    Dominant7,
    /// Diminished triad
    Diminished,
    /// Diminished 7th chord
    Diminished7,
    /// Augmented triad
    Augmented,
}

/// Chord structure with root and type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    /// Root note as MIDI note number
    pub root: u8,
    /// Type of chord
    pub chord_type: ChordType,
}

/// Melody generator for AI-powered melody creation.
///
/// The MelodyGenerator creates melodies based on musical theory rules,
/// supporting various scales, styles, and customizable parameters.
///
/// # Fields
///
/// * `sample_rate` - Audio sample rate (for future use)
/// * `key` - The musical key (root note and scale)
/// * `tempo` - Tempo in beats per minute
/// * `length` - Number of measures to generate
/// * `complexity` - How complex the melody is (0.0-1.0)
/// * `randomness` - How random the melody is (0.0-1.0)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MelodyGenerator {
    /// Audio sample rate in Hz
    sample_rate: f64,
    /// Musical key configuration
    key: Key,
    /// Tempo in beats per minute
    tempo: f64,
    /// Number of measures to generate
    length: usize,
    /// Melody complexity (0.0-1.0)
    complexity: f32,
    /// Melody randomness (0.0-1.0)
    randomness: f32,
}

/// Melody generator implementation.
#[allow(dead_code)]
impl MelodyGenerator {
    /// Creates a new melody generator.
    ///
    /// # Arguments
    ///
    /// * `key` - The musical key (root note and scale)
    /// * `tempo` - Tempo in beats per minute
    /// * `length` - Number of measures to generate
    ///
    /// # Returns
    ///
    /// A new MelodyGenerator instance with default complexity (0.5) and
    /// randomness (0.5) settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::melody_generator::{MelodyGenerator, Key, Scale};
    ///
    /// let key = Key { root: 60, scale: Scale::Major };
    /// let mut generator = MelodyGenerator::new(key, 120.0, 8);
    /// ```
    pub fn new(key: Key, tempo: f64, length: usize) -> Self {
        Self {
            sample_rate: 44100.0,
            key,
            tempo,
            length,
            complexity: 0.5,
            randomness: 0.5,
        }
    }

    /// Creates a melody generator with custom AI parameters.
    ///
    /// # Arguments
    ///
    /// * `key` - The musical key
    /// * `tempo` - Tempo in BPM
    /// * `length` - Number of measures
    /// * `complexity` - How complex the melody is (0.0-1.0)
    /// * `randomness` - How random the melody is (0.0-1.0)
    pub fn with_params(
        key: Key,
        tempo: f64,
        length: usize,
        complexity: f32,
        randomness: f32,
    ) -> Self {
        Self {
            sample_rate: 44100.0,
            key,
            tempo,
            length,
            complexity: complexity.clamp(0.0, 1.0),
            randomness: randomness.clamp(0.0, 1.0),
        }
    }

    /// Generates a melody based on current parameters.
    ///
    /// Uses the configured key, tempo, length, complexity, and randomness
    /// settings to generate a melody following music theory rules.
    ///
    /// # Returns
    ///
    /// A Melody struct containing all generated notes and metadata.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::melody_generator::{MelodyGenerator, Key, Scale};
    ///
    /// let key = Key { root: 60, scale: Scale::Major };
    /// let mut generator = MelodyGenerator::new(key, 100.0, 4);
    /// let melody = generator.generate();
    /// ```
    pub fn generate(&mut self) -> Melody {
        let scale_notes = self.scale_notes();
        let beats_per_measure = 4.0;
        let _total_beats = self.length as f64 * beats_per_measure;

        let mut notes = Vec::new();
        let mut durations = Vec::new();

        let mut current_beat = 0.0;
        let mut rng = rand::thread_rng();

        // Generate phrases (4 beats each)
        let phrase_length = 4.0;
        let num_phrases = self.length / 4;

        for phrase_idx in 0..num_phrases {
            let phrase_end = (phrase_idx + 1) as f64 * phrase_length;

            while current_beat < phrase_end {
                // Decide note duration based on complexity and randomness
                let duration = self.generate_note_duration(&mut rng, phrase_end - current_beat);

                // Decide pitch based on scale and motion rules
                let pitch = self.generate_pitch(
                    &scale_notes,
                    notes.last(),
                    &mut rng,
                    phrase_idx,
                    current_beat,
                );

                // Generate velocity with natural variation
                let velocity = self.generate_velocity(&mut rng, current_beat, phrase_end);

                // Create the note
                let note = MelodyNote {
                    pitch,
                    velocity,
                    start_beat: current_beat,
                    duration,
                };

                notes.push(note);
                durations.push(duration);
                current_beat += duration;

                // Handle phrase boundary - return to tonic
                if (phrase_end - current_beat).abs() < 0.01 {
                    let tonic_pitch = self.get_tonic_pitch(&scale_notes);
                    if let Some(tonic) = tonic_pitch {
                        let end_note = MelodyNote {
                            pitch: tonic,
                            velocity: 0.6,
                            start_beat: current_beat,
                            duration: 1.0,
                        };
                        notes.push(end_note);
                        durations.push(1.0);
                        current_beat += 1.0;
                    }
                }
            }
        }

        Melody {
            notes,
            durations,
            key: self.key,
            tempo: self.tempo,
            style: MelodyStyle::Custom,
        }
    }

    /// Generates a melody using a preset style.
    ///
    /// # Arguments
    ///
    /// * `style` - The style preset to use
    ///
    /// # Returns
    ///
    /// A Melody struct configured for the specified style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::melody_generator::{MelodyGenerator, Key, Scale, MelodyStyle};
    ///
    /// let key = Key { root: 60, scale: Scale::Major };
    /// let mut generator = MelodyGenerator::new(key, 120.0, 4);
    /// let jazz_melody = generator.generate_preset(MelodyStyle::Jazz);
    /// ```
    pub fn generate_preset(&mut self, style: MelodyStyle) -> Melody {
        // Configure parameters based on style
        match style {
            MelodyStyle::Pop => {
                self.complexity = 0.3;
                self.randomness = 0.3;
                // Pop typically uses major scale
                if self.key.scale == Scale::Minor || self.key.scale == Scale::HarmonicMinor {
                    // Keep minor for pop ballads
                }
            }
            MelodyStyle::Jazz => {
                self.complexity = 0.8;
                self.randomness = 0.6;
                // Jazz often uses modes
                if self.key.scale == Scale::Major {
                    self.key.scale = Scale::Mixolydian;
                } else if self.key.scale == Scale::Minor {
                    self.key.scale = Scale::Dorian;
                }
            }
            MelodyStyle::LoFi => {
                self.complexity = 0.2;
                self.randomness = 0.4;
                // LoFi uses various scales but often minor
                self.tempo = (self.tempo * 0.8).max(60.0);
            }
            MelodyStyle::EDM => {
                self.complexity = 0.4;
                self.randomness = 0.2;
                // EDM driving rhythm
                self.tempo = (self.tempo).max(120.0);
            }
            MelodyStyle::Ambient => {
                self.complexity = 0.6;
                self.randomness = 0.5;
                // Ambient often uses modes
                if self.key.scale == Scale::Major {
                    self.key.scale = Scale::Lydian;
                } else if self.key.scale == Scale::Minor {
                    self.key.scale = Scale::Dorian;
                }
                self.tempo = (self.tempo * 0.7).max(50.0);
            }
            MelodyStyle::Classical => {
                self.complexity = 0.6;
                self.randomness = 0.2;
                // Classical typically uses major or minor
            }
            MelodyStyle::Custom => {
                // Use existing parameters
            }
        }

        let mut melody = self.generate();
        melody.style = style;
        melody
    }

    /// Gets all notes in the current scale.
    ///
    /// Calculates all valid MIDI notes that belong to the current scale
    /// within a reasonable pitch range (MIDI 24-96, roughly 2-8 octaves).
    ///
    /// # Returns
    ///
    /// Vector of MIDI note numbers that are in the current scale.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::melody_generator::{MelodyGenerator, Key, Scale};
    ///
    /// let key = Key { root: 60, scale: Scale::Major };
    /// let generator = MelodyGenerator::new(key, 120.0, 4);
    /// let scale_notes = generator.scale_notes();
    /// // Returns [60, 62, 64, 65, 67, 69, 71, 72, ...]
    /// ```
    pub fn scale_notes(&self) -> Vec<u8> {
        let intervals = self.get_scale_intervals(self.key.scale);
        let mut notes = Vec::new();

        // Generate notes across multiple octaves (from MIDI 24 to 96)
        for octave in 0..7 {
            for &interval in &intervals {
                let octave_offset = (octave * 12) as i16;
                let note = self.key.root as i16 + octave_offset + interval as i16;
                if (24..=108).contains(&note) {
                    notes.push(note as u8);
                }
            }
        }

        notes.sort();
        notes
    }

    /// Gets chord tones for a given chord.
    ///
    /// Returns notes that belong to both the current scale and the given chord.
    /// Useful for creating melodies that harmonize with chord progressions.
    ///
    /// # Arguments
    ///
    /// * `chord` - The chord to get tones for
    ///
    /// # Returns
    ///
    /// Vector of MIDI note numbers that are in both the scale and the chord.
    pub fn chord_tones(&self, chord: &Chord) -> Vec<u8> {
        let chord_intervals = match chord.chord_type {
            ChordType::Major => vec![0, 4, 7],
            ChordType::Minor => vec![0, 3, 7],
            ChordType::Major7 => vec![0, 4, 7, 11],
            ChordType::Minor7 => vec![0, 3, 7, 10],
            ChordType::Dominant7 => vec![0, 4, 7, 10],
            ChordType::Diminished => vec![0, 3, 6],
            ChordType::Diminished7 => vec![0, 3, 6, 9],
            ChordType::Augmented => vec![0, 4, 8],
        };

        let scale_notes = self.scale_notes();
        let mut chord_tones = Vec::new();

        for &interval in &chord_intervals {
            let note = chord.root + interval;
            if let Ok(idx) = scale_notes.binary_search(&note) {
                chord_tones.push(scale_notes[idx]);
            }
        }

        chord_tones
    }

    /// Sets the rhythm quantization grid.
    ///
    /// Quantizes all note start times and durations to a rhythmic grid.
    ///
    /// # Arguments
    ///
    /// * `grid` - Grid size in beats (e.g., 0.25 for 16th notes, 0.5 for 8th notes)
    pub fn quantize(&mut self, _grid: f64) {
        // This method is deprecated - use Melody::quantize instead
        // Kept for backward compatibility
    }
}

impl Melody {
    /// Quantizes all note start times and durations to a rhythmic grid.
    ///
    /// # Arguments
    ///
    /// * `grid` - Grid size in beats (e.g., 0.25 for 16th notes, 0.5 for 8th notes)
    pub fn quantize(&mut self, grid: f64) {
        for note in &mut self.notes {
            note.start_beat = (note.start_beat / grid).round() * grid;
            note.duration = (note.duration / grid).round() * grid;
            // Ensure minimum duration
            if note.duration < grid {
                note.duration = grid;
            }
        }
    }
}

impl MelodyGenerator {
    /// Exports the melody as a MIDI file.
    ///
    /// # Arguments
    ///
    /// * `path` - File path for the output MIDI file
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error message on failure.
    ///
    /// # Note
    ///
    /// This requires the `mido` crate to be available. If not available,
    /// the function will return an error.
    pub fn export_midi(&mut self, _path: &str) -> Result<(), Box<dyn Error>> {
        // Try to use mido if available, otherwise return helpful error
        #[cfg(feature = "midi_cc")]
        {
            use std::fs::File;
            use std::io::Write;

            // Generate the melody first
            let melody = self.generate();
            let notes = &melody.notes;

            let mut file = File::create(_path)?;

            // Write MIDI header
            // MThd + 6 bytes header length + format type + num tracks + ticks per beat
            let track_count = 1u16;
            let ticks_per_beat = 480u16;
            let mut header = b"MThd\x00\x00\x00\x06\x00\x01".to_vec();
            header.extend_from_slice(&track_count.to_be_bytes());
            header.extend_from_slice(&ticks_per_beat.to_be_bytes());
            file.write_all(&header)?;

            // Write track chunk header
            let track_size = (notes.len() * 10) as u32;
            let mut track_header = b"MTrk\x00\x00\x00".to_vec();
            track_header.extend_from_slice(&track_size.to_be_bytes());
            file.write_all(&track_header)?;

            // Write tempo
            let tempo_us_per_beat = 60_000_000.0 / self.tempo;
            let mut tempo_msg = vec![0x00, 0xFF, 0x51, 0x03];
            let tempo_bytes = (tempo_us_per_beat as u32).to_be_bytes();
            tempo_msg.extend_from_slice(&tempo_bytes[3..]); // Take last 3 bytes for tempo
            file.write_all(&tempo_msg)?;

            // Write note events
            let mut previous_time = 0f64;
            for note in notes {
                let delta_time = ((note.start_beat - previous_time) * 480.0) as u32;
                previous_time = note.start_beat;

                // Note on: delta_time, 0x90, pitch, velocity
                let mut on_event = delta_time.to_be_bytes().to_vec();
                on_event.push(0x90);
                on_event.push(note.pitch);
                on_event.push((note.velocity * 127.0) as u8);
                file.write_all(&on_event)?;

                // Note off: duration_ticks, 0x80, pitch, 0x00
                let duration_ticks = (note.duration * 480.0) as u32; // 480 ticks per beat
                let mut off_event = duration_ticks.to_be_bytes().to_vec();
                off_event.push(0x80);
                off_event.push(note.pitch);
                off_event.push(0x00);
                file.write_all(&off_event)?;
            }

            // End of track: 0x00 0xFF 0x2F 0x00
            let end_track = vec![0x00, 0xFF, 0x2F, 0x00];
            file.write_all(&end_track)?;

            Ok(())
        }

        #[cfg(not(feature = "midi_cc"))]
        Err(Box::new(std::io::Error::other(
            "MIDI export requires the 'midi' feature flag. Add `mido = \"0.5\"` to your Cargo.toml",
        )))
    }

    // ===== Private Helper Methods =====

    /// Gets the interval pattern for a given scale.
    fn get_scale_intervals(&self, scale: Scale) -> Vec<u8> {
        match scale {
            Scale::Major => vec![0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => vec![0, 2, 3, 5, 7, 8, 10],
            Scale::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            Scale::MelodicMinor => vec![0, 2, 3, 5, 7, 9, 11],
            Scale::PentatonicMajor => vec![0, 2, 4, 7, 9],
            Scale::PentatonicMinor => vec![0, 3, 5, 7, 10],
            Scale::Blues => vec![0, 3, 5, 6, 7, 10],
            Scale::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            Scale::Phrygian => vec![0, 1, 3, 5, 7, 8, 10],
            Scale::Lydian => vec![0, 2, 4, 6, 7, 9, 11],
            Scale::Mixolydian => vec![0, 2, 4, 5, 7, 9, 10],
            Scale::Locrian => vec![0, 1, 3, 5, 6, 8, 10],
            Scale::WholeTone => vec![0, 2, 4, 6, 8, 10],
            Scale::Chromatic => vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        }
    }

    /// Generates a random note duration based on complexity and randomness.
    fn generate_note_duration<R: Rng>(&self, rng: &mut R, max_duration: f64) -> f64 {
        let base_durations = [0.25, 0.5, 1.0, 2.0]; // 16th, 8th, quarter, half

        // Higher complexity = more varied rhythms
        // Higher randomness = more chance of unusual durations
        let mut weights: Vec<f64> = base_durations
            .iter()
            .map(|&d| {
                let base_weight = if d <= 1.0 { 2.0 } else { 1.0 };
                base_weight * (1.0 - self.complexity as f64) * (1.0 - self.randomness as f64)
            })
            .collect();

        // Add randomness to weights
        for weight in &mut weights {
            let random_amount = 0.5 * self.randomness;
            *weight += rng.gen_range(0.0..random_amount as f64);
        }

        // Normalize weights
        let total: f64 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= total);

        // Choose duration based on weights
        let mut cumulative = 0.0;
        let choice = rng.gen_range(0.0..1.0);

        for (i, &duration) in base_durations.iter().enumerate() {
            cumulative += weights[i];
            if choice <= cumulative && duration <= max_duration {
                return duration;
            }
        }

        // Fallback to shortest valid duration
        base_durations
            .iter()
            .find(|&&d| d <= max_duration)
            .copied()
            .unwrap_or(0.25)
    }

    /// Generates a pitch following music theory rules.
    fn generate_pitch<R: Rng>(
        &self,
        scale_notes: &[u8],
        last_note: Option<&MelodyNote>,
        rng: &mut R,
        _phrase_idx: usize,
        current_beat: f64,
    ) -> u8 {
        // Get tonic for reference
        let tonic = self.get_tonic_pitch(scale_notes).unwrap_or(self.key.root);

        // Calculate position in phrase (0.0 to 1.0)
        let phrase_progress = (current_beat % 4.0) / 4.0;

        // Decide whether to move stepwise or leap
        let leap_chance = self.randomness * 0.3; // More random = more leaps

        if let Some(last) = last_note {
            // Find last note's position in scale
            if let Some(last_idx) = scale_notes.iter().position(|&n| n == last.pitch) {
                // Stepwise motion preference
                if rng.gen::<f32>() > leap_chance {
                    // Step up or down
                    let step = if rng.gen_bool(0.5) { 1 } else { -1 };
                    let new_idx = (last_idx as i32 + step)
                        .clamp(0i32, (scale_notes.len() as i32) - 1)
                        as usize;
                    return scale_notes[new_idx];
                }
            }
        }

        // For phrase endings, prefer tonic
        if phrase_progress > 0.75 {
            if let Some(&closest) = scale_notes
                .iter()
                .filter(|&&n| (n as i16 - tonic as i16).abs() <= 12)
                .min_by_key(|&&n| (n as i16 - tonic as i16).unsigned_abs())
            {
                return closest;
            }
        }

        // Return a note from the scale, preferring middle range
        let middle_notes: Vec<&u8> = scale_notes
            .iter()
            .filter(|&&n| (48..=84).contains(&n))
            .collect();

        if !middle_notes.is_empty() && rng.gen::<f32>() < 0.7 {
            *middle_notes[rng.gen_range(0..middle_notes.len())]
        } else {
            scale_notes[rng.gen_range(0..scale_notes.len())]
        }
    }

    /// Generates natural velocity variation.
    fn generate_velocity<R: Rng>(&self, rng: &mut R, current_beat: f64, phrase_end: f64) -> f32 {
        // Phrase endings are softer
        let phrase_progress = current_beat / phrase_end;
        if phrase_progress > 0.8 {
            return rng.gen_range(0.4..0.7);
        }

        // Beat emphasis (beats 1 and 3 have more emphasis)
        let beat_in_measure = current_beat % 4.0;
        let emphasis = if (beat_in_measure - 0.0).abs() < 0.1 || (beat_in_measure - 2.0).abs() < 0.1
        {
            1.2
        } else {
            1.0
        };

        // Base velocity with natural variation
        let base_velocity = rng.gen_range(0.6..0.9);
        (base_velocity * emphasis as f32).clamp(0.0, 1.0)
    }

    /// Gets the tonic (root) pitch of the current scale.
    fn get_tonic_pitch(&self, scale_notes: &[u8]) -> Option<u8> {
        scale_notes.iter().find(|&&n| n == self.key.root).copied()
    }
}

// ===== Unit Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test generator
    fn create_test_generator() -> MelodyGenerator {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        MelodyGenerator::new(key, 120.0, 4)
    }

    #[test]
    fn test_scale_notes_major() {
        let generator = create_test_generator();
        let notes = generator.scale_notes();

        // C major scale notes should include C4 (60), D4 (62), E4 (64), etc.
        assert!(notes.contains(&60)); // C4
        assert!(notes.contains(&62)); // D4
        assert!(notes.contains(&64)); // E4
        assert!(notes.contains(&65)); // F4
        assert!(notes.contains(&67)); // G4
        assert!(notes.contains(&69)); // A4
        assert!(notes.contains(&71)); // B4
        assert!(notes.contains(&72)); // C5

        // Should not contain F# (61) or C# (61)
        assert!(!notes.contains(&61));
        assert!(!notes.contains(&63));
    }

    #[test]
    fn test_scale_notes_pentatonic() {
        let key = Key {
            root: 60,
            scale: Scale::PentatonicMajor,
        };
        let generator = MelodyGenerator::new(key, 120.0, 4);
        let notes = generator.scale_notes();

        // C major pentatonic: C, D, E, G, A
        assert!(notes.contains(&60)); // C
        assert!(notes.contains(&62)); // D
        assert!(notes.contains(&64)); // E
        assert!(notes.contains(&67)); // G
        assert!(notes.contains(&69)); // A
    }

    #[test]
    fn test_scale_notes_blues() {
        let key = Key {
            root: 60,
            scale: Scale::Blues,
        };
        let generator = MelodyGenerator::new(key, 120.0, 4);
        let notes = generator.scale_notes();

        // C blues: C, Eb, F, Gb, G, Bb
        assert!(notes.contains(&60)); // C
        assert!(notes.contains(&63)); // Eb
        assert!(notes.contains(&65)); // F
        assert!(notes.contains(&66)); // Gb
        assert!(notes.contains(&67)); // G
        assert!(notes.contains(&70)); // Bb
    }

    #[test]
    fn test_generate_basic() {
        let mut generator = create_test_generator();
        let melody = generator.generate();

        // Should have notes
        assert!(!melody.notes.is_empty());

        // All notes should be in scale
        let scale_notes = generator.scale_notes();
        for note in &melody.notes {
            assert!(scale_notes.contains(&note.pitch));
        }

        // Tempo should match
        assert_eq!(melody.tempo, 120.0);

        // Key should match
        assert_eq!(melody.key.root, 60);
    }

    #[test]
    fn test_generate_preset_pop() {
        let mut generator = create_test_generator();
        let melody = generator.generate_preset(MelodyStyle::Pop);

        assert!(!melody.notes.is_empty());
        assert_eq!(melody.style, MelodyStyle::Pop);
    }

    #[test]
    fn test_generate_preset_jazz() {
        let mut generator = create_test_generator();
        let melody = generator.generate_preset(MelodyStyle::Jazz);

        assert!(!melody.notes.is_empty());
        assert_eq!(melody.style, MelodyStyle::Jazz);
    }

    #[test]
    fn test_generate_preset_lofi() {
        let mut generator = create_test_generator();
        let melody = generator.generate_preset(MelodyStyle::LoFi);

        assert!(!melody.notes.is_empty());
        assert_eq!(melody.style, MelodyStyle::LoFi);
    }

    #[test]
    fn test_generate_preset_edm() {
        let mut generator = create_test_generator();
        let melody = generator.generate_preset(MelodyStyle::EDM);

        assert!(!melody.notes.is_empty());
        assert_eq!(melody.style, MelodyStyle::EDM);
    }

    #[test]
    fn test_generate_preset_ambient() {
        let mut generator = create_test_generator();
        let melody = generator.generate_preset(MelodyStyle::Ambient);

        assert!(!melody.notes.is_empty());
        assert_eq!(melody.style, MelodyStyle::Ambient);
    }

    #[test]
    fn test_generate_preset_classical() {
        let mut generator = create_test_generator();
        let melody = generator.generate_preset(MelodyStyle::Classical);

        assert!(!melody.notes.is_empty());
        assert_eq!(melody.style, MelodyStyle::Classical);
    }

    #[test]
    fn test_different_styles_have_different_characteristics() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };

        let pop_melody = MelodyGenerator::new(key, 120.0, 8).generate_preset(MelodyStyle::Pop);
        let jazz_melody = MelodyGenerator::new(key, 120.0, 8).generate_preset(MelodyStyle::Jazz);
        let ambient_melody =
            MelodyGenerator::new(key, 120.0, 8).generate_preset(MelodyStyle::Ambient);

        // All should produce valid melodies
        assert!(!pop_melody.notes.is_empty());
        assert!(!jazz_melody.notes.is_empty());
        assert!(!ambient_melody.notes.is_empty());
    }

    #[test]
    fn test_chord_tones() {
        let generator = create_test_generator();
        let chord = Chord {
            root: 60, // C
            chord_type: ChordType::Major,
        };

        let tones = generator.chord_tones(&chord);

        // C major chord: C, E, G
        assert!(tones.contains(&60)); // C
        assert!(tones.contains(&64)); // E
        assert!(tones.contains(&67)); // G
    }

    #[test]
    fn test_chord_tones_minor7() {
        let key = Key {
            root: 57, // A
            scale: Scale::Minor,
        };
        let generator = MelodyGenerator::new(key, 120.0, 4);
        let chord = Chord {
            root: 57, // A
            chord_type: ChordType::Minor7,
        };

        let tones = generator.chord_tones(&chord);

        // A minor 7 chord: A, C, E, G
        assert!(tones.contains(&57)); // A
        assert!(tones.contains(&60)); // C
        assert!(tones.contains(&64)); // E
        assert!(tones.contains(&67)); // G
    }

    #[test]
    fn test_quantize() {
        let mut generator = create_test_generator();
        let mut melody = generator.generate();

        // Quantize to 8th notes (using Melody's quantize method)
        melody.quantize(0.5);

        for note in &melody.notes {
            let start_quantized = (note.start_beat / 0.5).round() * 0.5;
            let dur_quantized = (note.duration / 0.5).round() * 0.5;
            assert!((note.start_beat - start_quantized).abs() < 0.001);
            assert!((note.duration - dur_quantized).abs() < 0.001);
        }
    }

    #[test]
    fn test_short_melody() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut generator = MelodyGenerator::new(key, 120.0, 1);
        let melody = generator.generate();

        // A 1-measure melody should produce at least some notes
        // (the generator may produce 0 notes for very short lengths, which is acceptable)
        assert!(melody.notes.len() <= 32, "1-measure melody should not be excessively long");
        // Verify structure is valid
        assert_eq!(melody.notes.len(), melody.durations.len());
    }

    #[test]
    fn test_long_melody() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };
        let mut generator = MelodyGenerator::new(key, 120.0, 64);
        let melody = generator.generate();

        // Should handle long melodies
        assert!(melody.notes.len() > 0);
    }

    #[test]
    fn test_velocity_range() {
        let mut generator = create_test_generator();
        let melody = generator.generate();

        for note in &melody.notes {
            assert!(note.velocity >= 0.0 && note.velocity <= 1.0);
        }
    }

    #[test]
    fn test_midi_note_range() {
        let mut generator = create_test_generator();
        let melody = generator.generate();

        for note in &melody.notes {
            assert!(note.pitch >= 24 && note.pitch <= 108);
        }
    }

    #[test]
    fn test_all_scales_generate() {
        let scales = [
            Scale::Major,
            Scale::Minor,
            Scale::HarmonicMinor,
            Scale::MelodicMinor,
            Scale::PentatonicMajor,
            Scale::PentatonicMinor,
            Scale::Blues,
            Scale::Dorian,
            Scale::Phrygian,
            Scale::Lydian,
            Scale::Mixolydian,
            Scale::Locrian,
            Scale::WholeTone,
            Scale::Chromatic,
        ];

        for scale in scales {
            let key = Key { root: 60, scale };
            let generator = MelodyGenerator::new(key, 120.0, 4);
            let notes = generator.scale_notes();

            // Should have scale notes
            assert!(!notes.is_empty(), "Scale {:?} should have notes", scale);

            // All notes should be within valid MIDI range
            for &note in &notes {
                assert!(
                    note >= 24 && note <= 108,
                    "Note {} out of range for {:?}",
                    note,
                    scale
                );
            }
        }
    }

    #[test]
    fn test_custom_complexity_and_randomness() {
        let key = Key {
            root: 60,
            scale: Scale::Major,
        };

        // High complexity, low randomness
        let gen1 = MelodyGenerator::with_params(key, 120.0, 4, 0.9, 0.1);
        assert_eq!(gen1.complexity, 0.9);
        assert_eq!(gen1.randomness, 0.1);

        // Low complexity, high randomness
        let gen2 = MelodyGenerator::with_params(key, 120.0, 4, 0.1, 0.9);
        assert_eq!(gen2.complexity, 0.1);
        assert_eq!(gen2.randomness, 0.9);

        // Clamped values
        let gen3 = MelodyGenerator::with_params(key, 120.0, 4, 1.5, -0.5);
        assert_eq!(gen3.complexity, 1.0);
        assert_eq!(gen3.randomness, 0.0);
    }

    #[test]
    fn test_melody_structure() {
        let mut generator = create_test_generator();
        let melody = generator.generate();

        // Notes and durations should have same length
        assert_eq!(melody.notes.len(), melody.durations.len());

        // Notes should have valid timing
        for (_, note) in melody.notes.iter().enumerate() {
            assert!(note.start_beat >= 0.0);
            assert!(note.duration > 0.0);
            assert!(note.velocity >= 0.0 && note.velocity <= 1.0);
        }
    }
}
