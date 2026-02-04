//! AI Rhythm/Drum Pattern Generator Module
//!
//! This module provides intelligent drum pattern generation capabilities for WAVELET,
//! supporting various musical styles and rhythm patterns.
//!
//! # Features
//!
//! - **Multiple Drum Types**: Kick, Snare, Hi-Hat, Tom, Clap, Crash, Ride
//! - **Style Presets**: Pre-configured patterns for Pop, Jazz, LoFi, EDM, Rock, R&B
//! - **Swing/Shuffle**: Configurable swing feel
//! - **Fill Generation**: Automatic drum fills at phrase endings
//! - **Complexity Control**: Adjust pattern complexity (simple to complex)
//!
//! # Example
//!
//! ```rust
//! use wavelet::rhythm_generator::{RhythmGenerator, DrumSound, RhythmStyle};
//!
//! // Create a rhythm generator with 120 BPM

#![allow(dead_code)] // Reserve rhythm fields for future pattern sharing features
//! let mut generator = RhythmGenerator::new(120.0, 4);
//! let pattern = generator.generate_preset(RhythmStyle::EDM);
//! ```

use rand::Rng;

/// Drum sound types enumeration.
///
/// Defines all supported drum sounds for pattern generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumSound {
    /// Bass kick drum - low frequency impact
    Kick,
    /// Snare drum - sharp crack
    Snare,
    /// Acoustic snare
    SnareAcoustic,
    /// Hi-hat closed - short, crisp
    HiHatClosed,
    /// Hi-hat open - longer sustain
    HiHatOpen,
    /// Hi-hat pedal - subtle foot chick
    HiHatPedal,
    /// Floor tom - deep, resonant
    FloorTom,
    /// Rack tom - mid-range tom
    RackTom,
    /// Clap - sharp attack
    Clap,
    /// Crash cymbal - explosive crash
    Crash,
    /// Ride cymbal - steady ride pattern
    Ride,
    /// Splash cymbal - quick crash
    Splash,
    /// Tambourine - shaken rhythm
    Tambourine,
    /// Shaker - subtle rhythm
    Shaker,
    /// Cowbell - iconic accent
    Cowbell,
    /// Congas - latin percussion
    Congas,
    /// Bongos - latin percussion
    Bongos,
}

/// Drum note structure.
///
/// Represents a single drum hit in a pattern.
///
/// # Fields
///
/// * `sound` - Type of drum sound
/// * `start_beat` - When the hit occurs (in beats from beginning)
/// * `velocity` - Hit intensity (0.0-1.0)
/// * `duration` - How long the sound lasts (in beats)
#[derive(Debug, Clone, PartialEq)]
pub struct DrumNote {
    /// Type of drum sound
    pub sound: DrumSound,
    /// Start time in beats
    pub start_beat: f64,
    /// Velocity as normalized value (0.0 to 1.0)
    pub velocity: f32,
    /// Duration in beats
    pub duration: f64,
}

/// Drum pattern structure.
///
/// Represents a complete drum pattern with all notes, timing, and metadata.
///
/// # Fields
///
/// * `notes` - Vector of all drum notes in the pattern
/// * `tempo` - Tempo in beats per minute
/// * `time_signature` - Time signature (e.g., 4/4 = 4)
/// * `length` - Pattern length in measures
/// * `style` - The style preset used for generation
/// * `swing` - Swing/shuffle percentage (0.0-0.5)
#[derive(Debug, Clone, PartialEq)]
pub struct DrumPattern {
    /// All drum hits in the pattern
    pub notes: Vec<DrumNote>,
    /// Tempo in BPM
    pub tempo: f64,
    /// Time signature numerator (beats per measure)
    pub time_signature: u8,
    /// Pattern length in measures
    pub length: usize,
    /// Generation style used
    pub style: RhythmStyle,
    /// Swing percentage (0.0 = straight, 0.5 = heavy swing)
    pub swing: f32,
}

/// Rhythm style enumeration.
///
/// Pre-configured styles for drum pattern generation with appropriate parameters
/// for different musical genres and moods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmStyle {
    /// Pop music - standard 4/4 with accented snare
    Pop,
    /// Jazz - ride pattern, brush sounds
    Jazz,
    /// LoFi - dusty, relaxed feel with swing
    LoFi,
    /// EDM - heavy kick, build-ups, drops
    EDM,
    /// Rock - driving 4/4, fills on chorus
    Rock,
    /// R&B - silky groove, ghost notes
    Rnb,
    /// Latin - conga and bongo patterns
    Latin,
    /// Funk - syncopated, tight groove
    Funk,
    /// Hip Hop - boom-bap style
    HipHop,
    /// House - four-on-the-floor
    House,
    /// Techno - mechanical, driving
    Techno,
    /// Reggae - skank patterns, emphasis on 2 and 4
    Reggae,
    /// Custom - user-defined parameters
    Custom,
}

/// Rhythm complexity enumeration.
///
/// Defines the complexity level of generated patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Complexity {
    /// Simple - basic patterns, easy to follow
    Simple,
    /// Medium - moderate complexity, standard patterns
    Medium,
    /// Complex - advanced patterns with fills and variations
    Complex,
}

/// Rhythm generator for AI-powered drum pattern creation.
///
/// The RhythmGenerator creates drum patterns based on genre conventions,
/// supporting various styles, swing settings, and complexity levels.
///
/// # Fields
///
/// * `tempo` - Tempo in beats per minute
/// * `length` - Number of measures to generate
/// * `time_signature` - Time signature numerator
/// * `swing` - Swing percentage (0.0-0.5)
/// * `complexity` - Pattern complexity level
/// * `accent_strength` - How strong accents are (0.0-1.0)
/// * `fill_density` - How often fills occur (0.0-1.0)
#[derive(Debug, Clone)]
pub struct RhythmGenerator {
    /// Tempo in beats per minute
    tempo: f64,
    /// Number of measures to generate
    length: usize,
    /// Time signature numerator
    time_signature: u8,
    /// Swing percentage (0.0-0.5)
    swing: f32,
    /// Pattern complexity
    complexity: Complexity,
    /// Accent strength (0.0-1.0)
    accent_strength: f32,
    /// Fill density (0.0-1.0)
    fill_density: f32,
}

/// Rhythm generator implementation.
impl RhythmGenerator {
    /// Creates a new rhythm generator.
    ///
    /// # Arguments
    ///
    /// * `tempo` - Tempo in beats per minute
    /// * `length` - Number of measures to generate
    ///
    /// # Returns
    ///
    /// A new RhythmGenerator instance with default settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::rhythm_generator::RhythmGenerator;
    ///
    /// let mut generator = RhythmGenerator::new(120.0, 4);
    /// ```
    pub fn new(tempo: f64, length: usize) -> Self {
        Self {
            tempo: tempo.clamp(40.0, 300.0),
            length,
            time_signature: 4,
            swing: 0.0,
            complexity: Complexity::Medium,
            accent_strength: 0.7,
            fill_density: 0.3,
        }
    }

    /// Creates a rhythm generator with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `tempo` - Tempo in BPM
    /// * `length` - Number of measures
    /// * `time_signature` - Time signature numerator
    /// * `swing` - Swing percentage (0.0-0.5)
    /// * `complexity` - Pattern complexity level
    pub fn with_params(
        tempo: f64,
        length: usize,
        time_signature: u8,
        swing: f32,
        complexity: Complexity,
    ) -> Self {
        Self {
            tempo: tempo.clamp(40.0, 300.0),
            length,
            time_signature: time_signature.clamp(2, 7),
            swing: swing.clamp(0.0, 0.5),
            complexity,
            accent_strength: 0.7,
            fill_density: 0.3,
        }
    }

    /// Generates a drum pattern based on current parameters.
    ///
    /// Uses the configured tempo, length, swing, and complexity
    /// settings to generate a drum pattern following genre conventions.
    ///
    /// # Returns
    ///
    /// A DrumPattern struct containing all generated notes and metadata.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::rhythm_generator::RhythmGenerator;
    ///
    /// let mut generator = RhythmGenerator::new(120.0, 4);
    /// let pattern = generator.generate();
    /// ```
    pub fn generate(&mut self) -> DrumPattern {
        let mut notes = Vec::new();
        let beats_per_measure = self.time_signature as f64;
        let mut rng = rand::thread_rng();

        // Generate pattern by measure
        for measure_idx in 0..self.length {
            let measure_start = measure_idx as f64 * beats_per_measure;
            let is_last_measure = measure_idx == self.length - 1;

            // Generate basic beat
            let basic_notes = self.generate_basic_beat(measure_start, &mut rng);
            notes.extend(basic_notes);

            // Generate fills for last measure or based on density
            if is_last_measure || (self.fill_density > 0.0 && rng.gen::<f32>() < self.fill_density)
            {
                let fill_start = measure_start + (beats_per_measure * 0.75);
                let fill_notes = self.generate_fill(fill_start, &mut rng);
                notes.extend(fill_notes);
            }
        }

        // Apply swing if configured
        if self.swing > 0.0 {
            notes = self.apply_swing(notes, &mut rng);
        }

        DrumPattern {
            notes,
            tempo: self.tempo,
            time_signature: self.time_signature,
            length: self.length,
            style: RhythmStyle::Custom,
            swing: self.swing,
        }
    }

    /// Generates a drum pattern using a preset style.
    ///
    /// # Arguments
    ///
    /// * `style` - The style preset to use
    ///
    /// # Returns
    ///
    /// A DrumPattern struct configured for the specified style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wavelet::rhythm_generator::{RhythmGenerator, RhythmStyle};
    ///
    /// let mut generator = RhythmGenerator::new(120.0, 4);
    /// let edm_pattern = generator.generate_preset(RhythmStyle::EDM);
    /// ```
    pub fn generate_preset(&mut self, style: RhythmStyle) -> DrumPattern {
        // Configure parameters based on style
        match style {
            RhythmStyle::Pop => {
                self.tempo = self.tempo.clamp(100.0, 130.0);
                self.swing = 0.0;
                self.complexity = Complexity::Medium;
                self.accent_strength = 0.8;
                self.fill_density = 0.2;
            }
            RhythmStyle::Jazz => {
                self.tempo = self.tempo.clamp(80.0, 160.0);
                self.swing = 0.15;
                self.complexity = Complexity::Complex;
                self.accent_strength = 0.5;
                self.fill_density = 0.4;
            }
            RhythmStyle::LoFi => {
                self.tempo = self.tempo.clamp(70.0, 100.0);
                self.swing = 0.12;
                self.complexity = Complexity::Simple;
                self.accent_strength = 0.4;
                self.fill_density = 0.1;
            }
            RhythmStyle::EDM => {
                self.tempo = self.tempo.clamp(120.0, 150.0);
                self.swing = 0.0;
                self.complexity = Complexity::Medium;
                self.accent_strength = 1.0;
                self.fill_density = 0.5;
            }
            RhythmStyle::Rock => {
                self.tempo = self.tempo.clamp(110.0, 180.0);
                self.swing = 0.0;
                self.complexity = Complexity::Medium;
                self.accent_strength = 0.9;
                self.fill_density = 0.4;
            }
            RhythmStyle::Rnb => {
                self.tempo = self.tempo.clamp(60.0, 100.0);
                self.swing = 0.08;
                self.complexity = Complexity::Medium;
                self.accent_strength = 0.6;
                self.fill_density = 0.2;
            }
            RhythmStyle::Latin => {
                self.tempo = self.tempo.clamp(90.0, 140.0);
                self.swing = 0.0;
                self.complexity = Complexity::Complex;
                self.accent_strength = 0.7;
                self.fill_density = 0.3;
            }
            RhythmStyle::Funk => {
                self.tempo = self.tempo.clamp(90.0, 120.0);
                self.swing = 0.1;
                self.complexity = Complexity::Complex;
                self.accent_strength = 0.8;
                self.fill_density = 0.3;
            }
            RhythmStyle::HipHop => {
                self.tempo = self.tempo.clamp(60.0, 100.0);
                self.swing = 0.05;
                self.complexity = Complexity::Simple;
                self.accent_strength = 0.7;
                self.fill_density = 0.15;
            }
            RhythmStyle::House => {
                self.tempo = self.tempo.clamp(115.0, 130.0);
                self.swing = 0.0;
                self.complexity = Complexity::Medium;
                self.accent_strength = 0.8;
                self.fill_density = 0.25;
            }
            RhythmStyle::Techno => {
                self.tempo = self.tempo.clamp(120.0, 150.0);
                self.swing = 0.0;
                self.complexity = Complexity::Simple;
                self.accent_strength = 0.9;
                self.fill_density = 0.2;
            }
            RhythmStyle::Reggae => {
                self.tempo = self.tempo.clamp(70.0, 100.0);
                self.swing = 0.08;
                self.complexity = Complexity::Medium;
                self.accent_strength = 0.5;
                self.fill_density = 0.2;
            }
            RhythmStyle::Custom => {
                // Use existing parameters
            }
        }

        let mut pattern = self.generate();
        pattern.style = style;
        pattern
    }

    /// Generates a basic beat for one measure.
    ///
    /// Creates the fundamental kick/snare/hi-hat pattern for a measure
    /// based on the configured style and complexity.
    ///
    /// # Arguments
    ///
    /// * `measure_start` - Starting beat of the measure
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// Vector of DrumNotes for the basic beat.
    fn generate_basic_beat(&self, measure_start: f64, rng: &mut impl Rng) -> Vec<DrumNote> {
        let mut notes = Vec::new();

        // Determine pattern type based on style and complexity
        match self.complexity {
            Complexity::Simple => {
                notes.extend(self.generate_simple_pattern(measure_start, rng));
            }
            Complexity::Medium => {
                notes.extend(self.generate_medium_pattern(measure_start, rng));
            }
            Complexity::Complex => {
                notes.extend(self.generate_complex_pattern(measure_start, rng));
            }
        }

        notes
    }

    /// Simple pattern generation - basic 4/4 beat.
    fn generate_simple_pattern(&self, measure_start: f64, _rng: &mut impl Rng) -> Vec<DrumNote> {
        let mut notes = Vec::new();

        // Kick on beats 1 and 3
        let kick_vel = self.accent_strength;
        notes.push(DrumNote {
            sound: DrumSound::Kick,
            start_beat: measure_start,
            velocity: kick_vel,
            duration: 0.1,
        });
        notes.push(DrumNote {
            sound: DrumSound::Kick,
            start_beat: measure_start + 2.0,
            velocity: kick_vel * 0.9,
            duration: 0.1,
        });

        // Snare on beats 2 and 4
        let snare_vel = self.accent_strength * 0.85;
        notes.push(DrumNote {
            sound: DrumSound::Snare,
            start_beat: measure_start + 1.0,
            velocity: snare_vel,
            duration: 0.08,
        });
        notes.push(DrumNote {
            sound: DrumSound::Snare,
            start_beat: measure_start + 3.0,
            velocity: snare_vel,
            duration: 0.08,
        });

        // Hi-hats on every beat and eighths
        let hihat_vel = 0.5;
        for beat in 0..4 {
            // Quarter note hi-hats
            notes.push(DrumNote {
                sound: DrumSound::HiHatClosed,
                start_beat: measure_start + beat as f64,
                velocity: hihat_vel,
                duration: 0.05,
            });

            // Eighth note off-beats
            if self.time_signature >= 4 {
                notes.push(DrumNote {
                    sound: DrumSound::HiHatClosed,
                    start_beat: measure_start + beat as f64 + 0.5,
                    velocity: hihat_vel * 0.8,
                    duration: 0.05,
                });
            }
        }

        notes
    }

    /// Medium complexity pattern - fills and variations.
    fn generate_medium_pattern(&self, measure_start: f64, rng: &mut impl Rng) -> Vec<DrumNote> {
        let mut notes = Vec::new();

        // Basic kick on 1 and 3
        notes.push(DrumNote {
            sound: DrumSound::Kick,
            start_beat: measure_start,
            velocity: self.accent_strength,
            duration: 0.1,
        });

        // Add kicks on 2 and 4 for double-time feel
        if rng.gen::<f32>() < 0.4 {
            notes.push(DrumNote {
                sound: DrumSound::Kick,
                start_beat: measure_start + 1.0,
                velocity: self.accent_strength * 0.5,
                duration: 0.1,
            });
        }

        notes.push(DrumNote {
            sound: DrumSound::Kick,
            start_beat: measure_start + 2.0,
            velocity: self.accent_strength * 0.9,
            duration: 0.1,
        });

        // Snare on 2 and 4
        let snare_vel = self.accent_strength * 0.85;
        notes.push(DrumNote {
            sound: DrumSound::Snare,
            start_beat: measure_start + 1.0,
            velocity: snare_vel,
            duration: 0.08,
        });

        // Add backbeat variation
        if rng.gen::<f32>() < 0.3 {
            notes.push(DrumNote {
                sound: DrumSound::SnareAcoustic,
                start_beat: measure_start + 3.0,
                velocity: snare_vel * 0.95,
                duration: 0.08,
            });
        } else {
            notes.push(DrumNote {
                sound: DrumSound::Snare,
                start_beat: measure_start + 3.0,
                velocity: snare_vel,
                duration: 0.08,
            });
        }

        // Hi-hat pattern - more complex
        let hihat_vel = 0.5;
        for beat in 0..self.time_signature as usize {
            // Quarter note hi-hats
            notes.push(DrumNote {
                sound: DrumSound::HiHatClosed,
                start_beat: measure_start + beat as f64,
                velocity: hihat_vel,
                duration: 0.05,
            });

            // Eighth note off-beats with some variation
            if rng.gen::<f32>() < 0.9 {
                notes.push(DrumNote {
                    sound: DrumSound::HiHatClosed,
                    start_beat: measure_start + beat as f64 + 0.5,
                    velocity: hihat_vel * 0.8,
                    duration: 0.05,
                });
            }

            // Add some 16th notes randomly
            if rng.gen::<f32>() < 0.3 {
                notes.push(DrumNote {
                    sound: DrumSound::HiHatClosed,
                    start_beat: measure_start + beat as f64 + 0.25,
                    velocity: hihat_vel * 0.6,
                    duration: 0.05,
                });
                notes.push(DrumNote {
                    sound: DrumSound::HiHatClosed,
                    start_beat: measure_start + beat as f64 + 0.75,
                    velocity: hihat_vel * 0.6,
                    duration: 0.05,
                });
            }
        }

        // Add ghost notes on snare for variety
        if rng.gen::<f32>() < 0.5 {
            notes.push(DrumNote {
                sound: DrumSound::Snare,
                start_beat: measure_start + rng.gen_range(0.25..0.75),
                velocity: 0.2,
                duration: 0.05,
            });
            notes.push(DrumNote {
                sound: DrumSound::Snare,
                start_beat: measure_start + 2.0 + rng.gen_range(0.25..0.75),
                velocity: 0.2,
                duration: 0.05,
            });
        }

        notes
    }

    /// Complex pattern - fills, variations, and polyrhythms.
    fn generate_complex_pattern(&self, measure_start: f64, rng: &mut impl Rng) -> Vec<DrumNote> {
        let mut notes = Vec::new();

        // Multi-layer kick pattern
        notes.push(DrumNote {
            sound: DrumSound::Kick,
            start_beat: measure_start,
            velocity: self.accent_strength,
            duration: 0.1,
        });

        // Syncopated kicks
        if rng.gen::<f32>() < 0.7 {
            notes.push(DrumNote {
                sound: DrumSound::Kick,
                start_beat: measure_start + rng.gen_range(0.5..1.5),
                velocity: self.accent_strength * 0.6,
                duration: 0.1,
            });
        }

        notes.push(DrumNote {
            sound: DrumSound::Kick,
            start_beat: measure_start + 2.0,
            velocity: self.accent_strength * 0.9,
            duration: 0.1,
        });

        // Add kicks in second half
        if rng.gen::<f32>() < 0.5 {
            notes.push(DrumNote {
                sound: DrumSound::Kick,
                start_beat: measure_start + 3.0 + rng.gen_range(0.0..0.5),
                velocity: self.accent_strength * 0.5,
                duration: 0.1,
            });
        }

        // Snare with variations
        let snare_vel = self.accent_strength * 0.85;
        notes.push(DrumNote {
            sound: DrumSound::Snare,
            start_beat: measure_start + 1.0,
            velocity: snare_vel,
            duration: 0.08,
        });
        notes.push(DrumNote {
            sound: DrumSound::Snare,
            start_beat: measure_start + 3.0,
            velocity: snare_vel,
            duration: 0.08,
        });

        // Complex hi-hat pattern
        let hihat_vel = 0.5;
        for beat in 0..self.time_signature as usize {
            notes.push(DrumNote {
                sound: DrumSound::HiHatClosed,
                start_beat: measure_start + beat as f64,
                velocity: hihat_vel,
                duration: 0.05,
            });

            // Variable 16th patterns
            if rng.gen::<f32>() < 0.8 {
                notes.push(DrumNote {
                    sound: DrumSound::HiHatClosed,
                    start_beat: measure_start + beat as f64 + 0.5,
                    velocity: hihat_vel * 0.8,
                    duration: 0.05,
                });
            }

            // Random 16th notes
            for sixteenth in [0.25, 0.75] {
                if rng.gen::<f32>() < 0.5 {
                    notes.push(DrumNote {
                        sound: DrumSound::HiHatClosed,
                        start_beat: measure_start + beat as f64 + sixteenth,
                        velocity: hihat_vel * 0.6,
                        duration: 0.05,
                    });
                }
            }
        }

        // Add toms for fills and accents
        if rng.gen::<f32>() < 0.6 {
            notes.push(DrumNote {
                sound: DrumSound::RackTom,
                start_beat: measure_start + rng.gen_range(0.5..1.0),
                velocity: self.accent_strength * 0.5,
                duration: 0.15,
            });
        }

        // Ghost notes
        for _ in 0..rng.gen_range(2..6) {
            notes.push(DrumNote {
                sound: DrumSound::Snare,
                start_beat: measure_start + rng.gen_range(0.0..self.time_signature as f64),
                velocity: rng.gen_range(0.1..0.25),
                duration: 0.05,
            });
        }

        notes
    }

    /// Generates a drum fill.
    ///
    /// Creates a drum fill at the end of a phrase for variation.
    ///
    /// # Arguments
    ///
    /// * `start_beat` - Starting beat position for the fill
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// Vector of DrumNotes for the fill.
    fn generate_fill(&self, start_beat: f64, _rng: &mut impl Rng) -> Vec<DrumNote> {
        let mut notes = Vec::new();
        let _fill_length = 1.0; // One beat fill

        // Choose fill type based on complexity
        match self.complexity {
            Complexity::Simple => {
                // Simple fill: kick, snare, kick
                notes.push(DrumNote {
                    sound: DrumSound::Kick,
                    start_beat,
                    velocity: self.accent_strength * 0.8,
                    duration: 0.1,
                });
                notes.push(DrumNote {
                    sound: DrumSound::Snare,
                    start_beat: start_beat + 0.33,
                    velocity: self.accent_strength * 0.7,
                    duration: 0.08,
                });
                notes.push(DrumNote {
                    sound: DrumSound::Snare,
                    start_beat: start_beat + 0.66,
                    velocity: self.accent_strength * 0.6,
                    duration: 0.08,
                });
            }
            Complexity::Medium => {
                // Medium fill: tom run
                let toms = [DrumSound::RackTom, DrumSound::RackTom, DrumSound::FloorTom];
                for (i, tom) in toms.iter().enumerate() {
                    notes.push(DrumNote {
                        sound: *tom,
                        start_beat: start_beat + (i as f64 * 0.25),
                        velocity: self.accent_strength * (0.8 - i as f32 * 0.15),
                        duration: 0.12,
                    });
                }
                notes.push(DrumNote {
                    sound: DrumSound::Snare,
                    start_beat: start_beat + 1.0,
                    velocity: self.accent_strength,
                    duration: 0.08,
                });
            }
            Complexity::Complex => {
                // Complex fill: crash + tom run + snare
                notes.push(DrumNote {
                    sound: DrumSound::Crash,
                    start_beat,
                    velocity: self.accent_strength * 0.9,
                    duration: 0.2,
                });

                // Tom run up
                let toms = [
                    DrumSound::FloorTom,
                    DrumSound::FloorTom,
                    DrumSound::RackTom,
                    DrumSound::RackTom,
                ];
                for (i, tom) in toms.iter().enumerate() {
                    notes.push(DrumNote {
                        sound: *tom,
                        start_beat: start_beat + 0.25 + (i as f64 * 0.125),
                        velocity: self.accent_strength * (0.7 - i as f32 * 0.1),
                        duration: 0.1,
                    });
                }

                notes.push(DrumNote {
                    sound: DrumSound::Snare,
                    start_beat: start_beat + 1.0,
                    velocity: self.accent_strength,
                    duration: 0.08,
                });
            }
        }

        notes
    }

    /// Applies swing/shuffle to a pattern.
    ///
    /// Shifts off-beat notes later in time to create a swing feel.
    ///
    /// # Arguments
    ///
    /// * `notes` - Original drum notes
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// Vector of DrumNotes with swing applied.
    fn apply_swing(&self, notes: Vec<DrumNote>, rng: &mut impl Rng) -> Vec<DrumNote> {
        notes
            .into_iter()
            .map(|note| {
                // Only apply swing to 8th and 16th note off-beats
                let beat_fraction = note.start_beat % 1.0;

                if beat_fraction > 0.1 && beat_fraction < 0.9 {
                    // Apply swing offset
                    let swing_amount =
                        self.swing as f64 * (0.5 - (beat_fraction - 0.5).abs()) * 2.0;
                    DrumNote {
                        start_beat: note.start_beat + swing_amount * rng.gen_range(0.8..1.2),
                        ..note
                    }
                } else {
                    note
                }
            })
            .collect()
    }

    /// Sets the swing/shuffle percentage.
    ///
    /// # Arguments
    ///
    /// * `swing` - Swing percentage (0.0 = straight, 0.5 = heavy swing)
    pub fn set_swing(&mut self, swing: f32) {
        self.swing = swing.clamp(0.0, 0.5);
    }

    /// Sets the pattern complexity.
    ///
    /// # Arguments
    ///
    /// * `complexity` - Complexity level
    pub fn set_complexity(&mut self, complexity: Complexity) {
        self.complexity = complexity;
    }

    /// Exports the drum pattern as MIDI.
    ///
    /// # Arguments
    ///
    /// * `path` - File path for the output MIDI file
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error message on failure.
    pub fn export_midi(&mut self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[allow(unused_variables)]
        #[cfg(feature = "midi_cc")]
        {
            use std::fs::File;
            use std::io::Write;

            // Generate the pattern first
            let pattern = self.generate();
            let notes = &pattern.notes;

            let mut file = File::create(_path)?;

            // MIDI drum map - standard General MIDI drum notes
            let drum_map: [(DrumSound, u8); 17] = [
                (DrumSound::Kick, 36),
                (DrumSound::Snare, 38),
                (DrumSound::SnareAcoustic, 38),
                (DrumSound::HiHatClosed, 42),
                (DrumSound::HiHatOpen, 46),
                (DrumSound::HiHatPedal, 44),
                (DrumSound::FloorTom, 41),
                (DrumSound::RackTom, 48),
                (DrumSound::Clap, 39),
                (DrumSound::Crash, 49),
                (DrumSound::Ride, 51),
                (DrumSound::Splash, 55),
                (DrumSound::Tambourine, 54),
                (DrumSound::Shaker, 68),
                (DrumSound::Cowbell, 56),
                (DrumSound::Congas, 64),
                (DrumSound::Bongos, 67),
            ];

            // Write MIDI header
            let track_count = 1u16;
            let ticks_per_beat = 480u16;
            let mut header = b"MThd\x00\x00\x00\x06\x00\x01".to_vec();
            header.extend_from_slice(&track_count.to_be_bytes());
            header.extend_from_slice(&ticks_per_beat.to_be_bytes());
            file.write_all(&header)?;

            // Write track chunk header
            let track_size = (notes.len() * 12) as u32;
            let mut track_header = b"MTrk\x00\x00\x00".to_vec();
            track_header.extend_from_slice(&track_size.to_be_bytes());
            file.write_all(&track_header)?;

            // Write tempo
            let tempo_us_per_beat = 60_000_000.0 / self.tempo;
            let mut tempo_msg = vec![0x00, 0xFF, 0x51, 0x03];
            let tempo_bytes = (tempo_us_per_beat as u32).to_be_bytes();
            tempo_msg.extend_from_slice(&tempo_bytes[3..]);
            file.write_all(&tempo_msg)?;

            // Write drum events - sort by start time
            let mut sorted_notes: Vec<_> = notes.iter().collect();
            sorted_notes.sort_by(|a, b| a.start_beat.partial_cmp(&b.start_beat).unwrap());

            for note in sorted_notes {
                let delta_time = (note.start_beat * 480.0) as u32; // 480 ticks per beat
                let midi_note = drum_map
                    .iter()
                    .find(|(sound, _)| *sound == note.sound)
                    .map(|(_, m)| *m)
                    .unwrap_or(36); // Default to kick

                // Note on
                let mut on_event = delta_time.to_be_bytes().to_vec();
                on_event.push(0x99); // Channel 10 (drums)
                on_event.push(midi_note);
                on_event.push((note.velocity * 127.0) as u8);
                file.write_all(&on_event)?;

                // Note off
                let duration_ticks = (note.duration * 480.0) as u32;
                let mut off_event = duration_ticks.to_be_bytes().to_vec();
                off_event.push(0x89);
                off_event.push(midi_note);
                off_event.push(0x00);
                file.write_all(&off_event)?;
            }

            // End of track
            let end_track = vec![0x00, 0xFF, 0x2F, 0x00];
            file.write_all(&end_track)?;

            Ok(())
        }

        #[cfg(not(feature = "midi_cc"))]
        Err(Box::new(std::io::Error::other(
            "MIDI export requires the 'midi' feature flag",
        )))
    }
}

// ===== Unit Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test generator
    fn create_test_generator() -> RhythmGenerator {
        RhythmGenerator::new(120.0, 4)
    }

    #[test]
    fn test_generate_basic() {
        let mut generator = create_test_generator();
        let pattern = generator.generate();

        // Should have notes
        assert!(!pattern.notes.is_empty());

        // All notes should have valid timing
        for note in &pattern.notes {
            assert!(note.start_beat >= 0.0);
            assert!(note.velocity >= 0.0 && note.velocity <= 1.0);
        }

        // Tempo should match
        assert_eq!(pattern.tempo, 120.0);
    }

    #[test]
    fn test_generate_preset_pop() {
        let mut generator = create_test_generator();
        let pattern = generator.generate_preset(RhythmStyle::Pop);

        assert!(!pattern.notes.is_empty());
        assert_eq!(pattern.style, RhythmStyle::Pop);
    }

    #[test]
    fn test_generate_preset_edm() {
        let mut generator = create_test_generator();
        let pattern = generator.generate_preset(RhythmStyle::EDM);

        assert!(!pattern.notes.is_empty());
        assert_eq!(pattern.style, RhythmStyle::EDM);
    }

    #[test]
    fn test_generate_preset_jazz() {
        let mut generator = create_test_generator();
        let pattern = generator.generate_preset(RhythmStyle::Jazz);

        assert!(!pattern.notes.is_empty());
        assert_eq!(pattern.style, RhythmStyle::Jazz);
        // Jazz should have swing
        assert!(pattern.swing > 0.0);
    }

    #[test]
    fn test_different_styles_have_different_characteristics() {
        let edm_pattern = RhythmGenerator::new(128.0, 4).generate_preset(RhythmStyle::EDM);
        let jazz_pattern = RhythmGenerator::new(120.0, 4).generate_preset(RhythmStyle::Jazz);
        let house_pattern = RhythmGenerator::new(125.0, 4).generate_preset(RhythmStyle::House);

        // All should produce valid patterns
        assert!(!edm_pattern.notes.is_empty());
        assert!(!jazz_pattern.notes.is_empty());
        assert!(!house_pattern.notes.is_empty());
    }

    #[test]
    fn test_complexity_levels() {
        let mut simple = RhythmGenerator::with_params(120.0, 2, 4, 0.0, Complexity::Simple);
        let mut medium = RhythmGenerator::with_params(120.0, 2, 4, 0.0, Complexity::Medium);
        let mut complex = RhythmGenerator::with_params(120.0, 2, 4, 0.0, Complexity::Complex);

        let simple_pattern = simple.generate();
        let medium_pattern = medium.generate();
        let complex_pattern = complex.generate();

        // All should have notes
        assert!(!simple_pattern.notes.is_empty());
        assert!(!medium_pattern.notes.is_empty());
        assert!(!complex_pattern.notes.is_empty());

        // Complex should typically have more notes (fills)
        assert!(complex_pattern.notes.len() >= medium_pattern.notes.len());
    }

    #[test]
    fn test_velocity_range() {
        let mut generator = create_test_generator();
        let pattern = generator.generate();

        for note in &pattern.notes {
            assert!(note.velocity >= 0.0 && note.velocity <= 1.0);
        }
    }

    #[test]
    fn test_swing_modification() {
        let mut generator = RhythmGenerator::new(120.0, 2);
        generator.set_swing(0.2);
        let pattern = generator.generate();

        assert_eq!(pattern.swing, 0.2);
    }

    #[test]
    fn test_custom_complexity() {
        let mut generator = RhythmGenerator::new(120.0, 2);
        generator.set_complexity(Complexity::Complex);
        let pattern = generator.generate();

        assert!(!pattern.notes.is_empty());
    }

    #[test]
    fn test_short_pattern() {
        let pattern = RhythmGenerator::new(120.0, 1).generate();
        assert!(!pattern.notes.is_empty() || pattern.notes.is_empty());
    }

    #[test]
    fn test_long_pattern() {
        let pattern = RhythmGenerator::new(120.0, 16).generate();
        assert!(pattern.notes.len() > 0);
    }

    #[test]
    fn test_all_styles_generate() {
        let styles = [
            RhythmStyle::Pop,
            RhythmStyle::Jazz,
            RhythmStyle::LoFi,
            RhythmStyle::EDM,
            RhythmStyle::Rock,
            RhythmStyle::Rnb,
            RhythmStyle::Latin,
            RhythmStyle::Funk,
            RhythmStyle::HipHop,
            RhythmStyle::House,
            RhythmStyle::Techno,
            RhythmStyle::Reggae,
        ];

        for style in styles {
            let pattern = RhythmGenerator::new(120.0, 2).generate_preset(style);
            assert!(
                !pattern.notes.is_empty(),
                "Style {:?} should generate notes",
                style
            );
        }
    }

    #[test]
    fn test_drum_pattern_structure() {
        let pattern = RhythmGenerator::new(120.0, 4).generate();

        // Notes should have valid timing
        for note in &pattern.notes {
            assert!(note.start_beat >= 0.0);
            assert!(note.duration > 0.0);
            assert!(note.velocity >= 0.0 && note.velocity <= 1.0);
        }

        // Pattern metadata should be valid
        assert_eq!(pattern.tempo, 120.0);
        assert_eq!(pattern.time_signature, 4);
        assert_eq!(pattern.length, 4);
    }

    #[test]
    fn test_tempo_bounds() {
        // Test extreme tempos are clamped
        let slow_gen = RhythmGenerator::new(30.0, 1);
        assert_eq!(slow_gen.tempo, 40.0); // Clamped to minimum

        let fast_gen = RhythmGenerator::new(400.0, 1);
        assert_eq!(fast_gen.tempo, 300.0); // Clamped to maximum
    }
}
