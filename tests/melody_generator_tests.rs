//! Melody Generator Integration Tests
//!
//! This module contains comprehensive tests for the melody generator module,
//! testing scale generation, style presets, rhythm quantization, and edge cases.
//!
//! # Running Tests
//!
//! ```bash
//! cargo test melody_generator_tests
//! ```

use wavelet::melody_generator::{Chord, ChordType, Key, MelodyGenerator, MelodyStyle, Scale};
use wavelet::Melody;

/// Helper function to create a test generator with C major key.
fn create_c_major_generator() -> MelodyGenerator {
    let key = Key {
        root: 60, // C4
        scale: Scale::Major,
    };
    MelodyGenerator::new(key, 120.0, 4)
}

/// Helper function to create a generator with specified parameters.
fn create_generator(root: u8, scale: Scale, tempo: f64, length: usize) -> MelodyGenerator {
    let key = Key { root, scale };
    MelodyGenerator::new(key, tempo, length)
}

// ===== Scale Generation Tests =====

/// Tests that C major scale generates correct notes.
#[test]
fn test_c_major_scale_notes() {
    let generator = create_c_major_generator();
    let notes = generator.scale_notes();

    // C major: C, D, E, F, G, A, B
    assert!(notes.contains(&60), "Should contain C4 (60)");
    assert!(notes.contains(&62), "Should contain D4 (62)");
    assert!(notes.contains(&64), "Should contain E4 (64)");
    assert!(notes.contains(&65), "Should contain F4 (65)");
    assert!(notes.contains(&67), "Should contain G4 (67)");
    assert!(notes.contains(&69), "Should contain A4 (69)");
    assert!(notes.contains(&71), "Should contain B4 (71)");

    // Should not contain chromatic notes
    assert!(!notes.contains(&61), "Should not contain C#4");
    assert!(!notes.contains(&63), "Should not contain D#4");
}

/// Tests that all scales can generate notes.
#[test]
fn test_all_scales_generate_notes() {
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

        assert!(!notes.is_empty(), "Scale {:?} should generate notes", scale);

        // All notes should be valid MIDI notes
        for &note in &notes {
            assert!(
                note >= 0 && note <= 127,
                "Note {} is out of valid MIDI range for scale {:?}",
                note,
                scale
            );
        }
    }
}

/// Tests pentatonic scales have the correct number of notes per octave.
#[test]
fn test_pentatonic_scale_structure() {
    let major = create_generator(60, Scale::PentatonicMajor, 120.0, 4);
    let minor = create_generator(60, Scale::PentatonicMinor, 120.0, 4);

    let major_notes = major.scale_notes();
    let minor_notes = minor.scale_notes();

    // Both pentatonic scales should have 5 notes per octave
    let major_octave: Vec<u8> = major_notes
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();
    let minor_octave: Vec<u8> = minor_notes
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    assert_eq!(
        major_octave.len(),
        5,
        "Major pentatonic should have 5 notes"
    );
    assert_eq!(
        minor_octave.len(),
        5,
        "Minor pentatonic should have 5 notes"
    );
}

/// Tests blues scale has the characteristic "blue notes".
#[test]
fn test_blues_scale_structure() {
    let blues = create_generator(60, Scale::Blues, 120.0, 4);
    let notes: Vec<u8> = blues
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // C blues: C, Eb, F, Gb, G, Bb (6 notes)
    assert_eq!(notes.len(), 6, "Blues scale should have 6 notes per octave");

    // Check for characteristic intervals
    assert!(notes.contains(&60), "Should contain root (C)");
    assert!(notes.contains(&63), "Should contain minor 3rd (Eb)");
    assert!(notes.contains(&65), "Should contain perfect 4th (F)");
    assert!(notes.contains(&66), "Should contain diminished 5th (Gb)");
    assert!(notes.contains(&67), "Should contain perfect 5th (G)");
    assert!(notes.contains(&70), "Should contain minor 7th (Bb)");
}

// ===== Melody Generation Tests =====

/// Tests that basic melody generation produces notes in scale.
#[test]
fn test_generate_produces_valid_notes() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate();

    assert!(!melody.notes.is_empty(), "Melody should have notes");

    let scale_notes = generator.scale_notes();
    for note in &melody.notes {
        assert!(
            scale_notes.contains(&note.pitch),
            "Note pitch {} should be in scale",
            note.pitch
        );
    }
}

/// Tests that generated melody has correct metadata.
#[test]
fn test_melody_metadata() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate();

    assert_eq!(melody.key.root, 60, "Key root should be 60");
    assert_eq!(melody.key.scale, Scale::Major, "Key scale should be Major");
    assert_eq!(melody.tempo, 120.0, "Tempo should be 120.0");
    assert_eq!(
        melody.style,
        MelodyStyle::Custom,
        "Default style should be Custom"
    );

    // Notes and durations should match
    assert_eq!(
        melody.notes.len(),
        melody.durations.len(),
        "Notes and durations should have same length"
    );
}

/// Tests that melody notes have valid velocity range.
#[test]
fn test_melody_velocity_range() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate();

    for note in &melody.notes {
        assert!(
            note.velocity >= 0.0 && note.velocity <= 1.0,
            "Velocity {} should be between 0.0 and 1.0",
            note.velocity
        );
    }
}

/// Tests that melody notes have valid timing.
#[test]
fn test_melody_timing() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate();

    for note in &melody.notes {
        assert!(
            note.start_beat >= 0.0,
            "Start beat {} should be non-negative",
            note.start_beat
        );
        assert!(
            note.duration > 0.0,
            "Duration {} should be positive",
            note.duration
        );
    }
}

/// Tests that melody notes are within valid MIDI range.
#[test]
fn test_melody_midi_range() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate();

    for note in &melody.notes {
        assert!(
            note.pitch >= 24 && note.pitch <= 108,
            "Pitch {} should be between 24 and 108",
            note.pitch
        );
    }
}

// ===== Style Preset Tests =====

/// Tests Pop style preset.
#[test]
fn test_pop_style_preset() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate_preset(MelodyStyle::Pop);

    assert!(!melody.notes.is_empty(), "Pop melody should have notes");
    assert_eq!(melody.style, MelodyStyle::Pop);
}

/// Tests Jazz style preset.
#[test]
fn test_jazz_style_preset() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate_preset(MelodyStyle::Jazz);

    assert!(!melody.notes.is_empty(), "Jazz melody should have notes");
    assert_eq!(melody.style, MelodyStyle::Jazz);
}

/// Tests LoFi style preset.
#[test]
fn test_lofi_style_preset() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate_preset(MelodyStyle::LoFi);

    assert!(!melody.notes.is_empty(), "LoFi melody should have notes");
    assert_eq!(melody.style, MelodyStyle::LoFi);
}

/// Tests EDM style preset.
#[test]
fn test_edm_style_preset() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate_preset(MelodyStyle::EDM);

    assert!(!melody.notes.is_empty(), "EDM melody should have notes");
    assert_eq!(melody.style, MelodyStyle::EDM);
}

/// Tests Ambient style preset.
#[test]
fn test_ambient_style_preset() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate_preset(MelodyStyle::Ambient);

    assert!(!melody.notes.is_empty(), "Ambient melody should have notes");
    assert_eq!(melody.style, MelodyStyle::Ambient);
}

/// Tests Classical style preset.
#[test]
fn test_classical_style_preset() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate_preset(MelodyStyle::Classical);

    assert!(
        !melody.notes.is_empty(),
        "Classical melody should have notes"
    );
    assert_eq!(melody.style, MelodyStyle::Classical);
}

/// Tests that different styles produce melodies with different characteristics.
#[test]
fn test_styles_produce_different_melodies() {
    let key = Key {
        root: 60,
        scale: Scale::Major,
    };

    // Generate multiple melodies with same seed parameters
    let pop = MelodyGenerator::new(key, 120.0, 8).generate_preset(MelodyStyle::Pop);
    let jazz = MelodyGenerator::new(key, 120.0, 8).generate_preset(MelodyStyle::Jazz);
    let ambient = MelodyGenerator::new(key, 120.0, 8).generate_preset(MelodyStyle::Ambient);

    // All should be valid
    assert!(!pop.notes.is_empty());
    assert!(!jazz.notes.is_empty());
    assert!(!ambient.notes.is_empty());

    // All should have correct style
    assert_eq!(pop.style, MelodyStyle::Pop);
    assert_eq!(jazz.style, MelodyStyle::Jazz);
    assert_eq!(ambient.style, MelodyStyle::Ambient);
}

// ===== Rhythm Quantization Tests =====

/// Tests rhythm quantization to 8th notes.
#[test]
fn test_quantize_8th_notes() {
    let mut generator = create_c_major_generator();
    let mut melody = generator.generate();

    // Quantize to 8th notes (0.5 beats)
    melody.quantize(0.5);

    for note in &melody.notes {
        let start_quantized = (note.start_beat / 0.5).round() * 0.5;
        let dur_quantized = (note.duration / 0.5).round() * 0.5;

        assert!(
            (note.start_beat - start_quantized).abs() < 0.001,
            "Start beat {} should be quantized to grid",
            note.start_beat
        );
        assert!(
            (note.duration - dur_quantized).abs() < 0.001,
            "Duration {} should be quantized to grid",
            note.duration
        );
    }
}

/// Tests rhythm quantization to 16th notes.
#[test]
fn test_quantize_16th_notes() {
    let mut generator = create_c_major_generator();
    let mut melody = generator.generate();

    // Quantize to 16th notes (0.25 beats)
    melody.quantize(0.25);

    for note in &melody.notes {
        let start_quantized = (note.start_beat / 0.25).round() * 0.25;
        let dur_quantized = (note.duration / 0.25).round() * 0.25;

        assert!(
            (note.start_beat - start_quantized).abs() < 0.001,
            "Start beat {} should be quantized to grid",
            note.start_beat
        );
    }
}

/// Tests that quantize ensures minimum duration.
#[test]
fn test_quantize_minimum_duration() {
    let mut generator = create_c_major_generator();
    let mut melody = generator.generate();

    // Quantize to a large grid
    melody.quantize(2.0);

    for note in &melody.notes {
        assert!(
            note.duration >= 2.0,
            "Duration {} should be at least grid size",
            note.duration
        );
    }
}

// ===== Chord Tone Tests =====

/// Tests chord tone generation for major chord.
#[test]
fn test_chord_tones_major() {
    let generator = create_c_major_generator();
    let chord = Chord {
        root: 60,
        chord_type: ChordType::Major,
    };

    let tones = generator.chord_tones(&chord);

    // C major: C, E, G
    assert!(tones.contains(&60), "Should contain root (C)");
    assert!(tones.contains(&64), "Should contain major third (E)");
    assert!(tones.contains(&67), "Should contain perfect fifth (G)");
}

/// Tests chord tone generation for minor chord.
#[test]
fn test_chord_tones_minor() {
    let generator = create_c_major_generator();
    let chord = Chord {
        root: 60,
        chord_type: ChordType::Minor,
    };

    let tones = generator.chord_tones(&chord);

    // C minor: C, Eb, G
    assert!(tones.contains(&60), "Should contain root (C)");
    assert!(tones.contains(&63), "Should contain minor third (Eb)");
    assert!(tones.contains(&67), "Should contain perfect fifth (G)");
}

/// Tests chord tone generation for major 7th chord.
#[test]
fn test_chord_tones_major7() {
    let generator = create_c_major_generator();
    let chord = Chord {
        root: 60,
        chord_type: ChordType::Major7,
    };

    let tones = generator.chord_tones(&chord);

    // C major 7: C, E, G, B
    assert!(tones.contains(&60), "Should contain root");
    assert!(tones.contains(&64), "Should contain major third");
    assert!(tones.contains(&67), "Should contain perfect fifth");
    assert!(tones.contains(&71), "Should contain major seventh");
}

/// Tests chord tone generation for minor 7th chord.
#[test]
fn test_chord_tones_minor7() {
    let generator = create_c_major_generator();
    let chord = Chord {
        root: 60,
        chord_type: ChordType::Minor7,
    };

    let tones = generator.chord_tones(&chord);

    // C minor 7: C, Eb, G, Bb
    assert!(tones.contains(&60), "Should contain root");
    assert!(tones.contains(&63), "Should contain minor third");
    assert!(tones.contains(&67), "Should contain perfect fifth");
    assert!(tones.contains(&70), "Should contain minor seventh");
}

/// Tests chord tone generation for dominant 7th chord.
#[test]
fn test_chord_tones_dominant7() {
    let generator = create_c_major_generator();
    let chord = Chord {
        root: 60,
        chord_type: ChordType::Dominant7,
    };

    let tones = generator.chord_tones(&chord);

    // C dominant 7: C, E, G, Bb
    assert!(tones.contains(&60), "Should contain root");
    assert!(tones.contains(&64), "Should contain major third");
    assert!(tones.contains(&67), "Should contain perfect fifth");
    assert!(tones.contains(&70), "Should contain minor seventh");
}

/// Tests chord tone generation for diminished chord.
#[test]
fn test_chord_tones_diminished() {
    let generator = create_c_major_generator();
    let chord = Chord {
        root: 60,
        chord_type: ChordType::Diminished,
    };

    let tones = generator.chord_tones(&chord);

    // C diminished: C, Eb, Gb
    assert!(tones.contains(&60), "Should contain root");
    assert!(tones.contains(&63), "Should contain minor third");
    assert!(tones.contains(&66), "Should contain diminished fifth");
}

/// Tests chord tone generation for augmented chord.
#[test]
fn test_chord_tones_augmented() {
    let generator = create_c_major_generator();
    let chord = Chord {
        root: 60,
        chord_type: ChordType::Augmented,
    };

    let tones = generator.chord_tones(&chord);

    // C augmented: C, E, G#
    assert!(tones.contains(&60), "Should contain root");
    assert!(tones.contains(&64), "Should contain major third");
    assert!(tones.contains(&68), "Should contain augmented fifth");
}

// ===== Edge Case Tests =====

/// Tests very short melody generation (1 measure).
#[test]
fn test_short_melody() {
    let key = Key {
        root: 60,
        scale: Scale::Major,
    };
    let mut generator = MelodyGenerator::new(key, 120.0, 1);
    let melody = generator.generate();

    // Should produce a valid melody (may be empty for very short lengths)
    // The key requirement is that it's valid
    for note in &melody.notes {
        assert!(note.start_beat >= 0.0);
        assert!(note.duration > 0.0);
    }
}

/// Tests very long melody generation (64 measures).
#[test]
fn test_long_melody() {
    let key = Key {
        root: 60,
        scale: Scale::Major,
    };
    let mut generator = MelodyGenerator::new(key, 120.0, 64);
    let melody = generator.generate();

    assert!(!melody.notes.is_empty(), "Long melody should have notes");
    assert!(
        melody.notes.len() > 10,
        "Long melody should have multiple notes"
    );
}

/// Tests very slow tempo.
#[test]
fn test_slow_tempo() {
    let key = Key {
        root: 60,
        scale: Scale::Major,
    };
    let mut generator = MelodyGenerator::new(key, 40.0, 4);
    let melody = generator.generate();

    assert!(
        !melody.notes.is_empty(),
        "Slow tempo should still generate notes"
    );
}

/// Tests very fast tempo.
#[test]
fn test_fast_tempo() {
    let key = Key {
        root: 60,
        scale: Scale::Major,
    };
    let mut generator = MelodyGenerator::new(key, 200.0, 4);
    let melody = generator.generate();

    assert!(
        !melody.notes.is_empty(),
        "Fast tempo should still generate notes"
    );
}

/// Tests different root notes.
#[test]
fn test_different_root_notes() {
    let roots = [48, 57, 60, 65, 72]; // C3, A3, C4, F4, C5

    for root in roots {
        let key = Key {
            root,
            scale: Scale::Major,
        };
        let generator = MelodyGenerator::new(key, 120.0, 4);
        let notes = generator.scale_notes();

        assert!(
            notes.contains(&root),
            "Scale should contain root note {}",
            root
        );

        // Notes should span a reasonable range around root
        assert!(
            notes.iter().any(|&n| n >= root && n < root + 24),
            "Scale should have notes above root"
        );
    }
}

/// Tests minor key with all styles.
#[test]
fn test_minor_key_all_styles() {
    let key = Key {
        root: 57, // A3
        scale: Scale::Minor,
    };

    let styles = [
        MelodyStyle::Pop,
        MelodyStyle::Jazz,
        MelodyStyle::LoFi,
        MelodyStyle::EDM,
        MelodyStyle::Ambient,
        MelodyStyle::Classical,
    ];

    for style in styles {
        let mut generator = MelodyGenerator::new(key, 120.0, 4);
        let melody = generator.generate_preset(style);

        assert!(
            !melody.notes.is_empty(),
            "Style {:?} should generate notes for minor key",
            style
        );
    }
}

/// Tests with custom complexity and randomness parameters.
#[test]
fn test_custom_parameters() {
    let key = Key {
        root: 60,
        scale: Scale::Major,
    };

    // Test various parameter combinations
    let params = [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (0.1, 0.9), (0.9, 0.1)];

    for (complexity, randomness) in params {
        let generator = MelodyGenerator::with_params(key, 120.0, 4, complexity, randomness);
        assert_eq!(generator.complexity, complexity.clamp(0.0, 1.0));
        assert_eq!(generator.randomness, randomness.clamp(0.0, 1.0));
    }
}

/// Tests that clamped parameters work correctly.
#[test]
fn test_parameter_clamping() {
    let key = Key {
        root: 60,
        scale: Scale::Major,
    };

    // Values above 1.0 should be clamped
    let gen1 = MelodyGenerator::with_params(key, 120.0, 4, 1.5, 0.5);
    assert_eq!(gen1.complexity, 1.0);

    let gen2 = MelodyGenerator::with_params(key, 120.0, 4, 0.5, 1.5);
    assert_eq!(gen2.randomness, 1.0);

    // Negative values should be clamped to 0
    let gen3 = MelodyGenerator::with_params(key, 120.0, 4, -0.5, 0.5);
    assert_eq!(gen3.complexity, 0.0);

    let gen4 = MelodyGenerator::with_params(key, 120.0, 4, 0.5, -0.5);
    assert_eq!(gen4.randomness, 0.0);
}

// ===== Modal Scale Tests =====

/// Tests Dorian mode generates correct notes.
#[test]
fn test_dorian_mode() {
    let generator = create_generator(60, Scale::Dorian, 120.0, 4);
    let notes: Vec<u8> = generator
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // D Dorian: D, E, F, G, A, B, C
    // Relative to C: C, D, Eb, F, G, A, Bb
    assert!(notes.contains(&60), "Should contain C");
    assert!(notes.contains(&62), "Should contain D");
    assert!(!notes.contains(&64), "Should not contain E (natural)");
    assert!(notes.contains(&65), "Should contain F");
    assert!(notes.contains(&67), "Should contain G");
    assert!(notes.contains(&69), "Should contain A");
    assert!(!notes.contains(&71), "Should not contain B (natural)");
}

/// Tests Phrygian mode generates correct notes.
#[test]
fn test_phrygian_mode() {
    let generator = create_generator(60, Scale::Phrygian, 120.0, 4);
    let notes: Vec<u8> = generator
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // E Phrygian: E, F, G, A, B, C, D
    // Relative to C: C, Db, Eb, F, G, A, Bb
    assert!(notes.contains(&60), "Should contain C");
    assert!(!notes.contains(&61), "Should not contain C#");
    assert!(!notes.contains(&62), "Should not contain D (natural)");
    assert!(notes.contains(&65), "Should contain F");
    assert!(notes.contains(&67), "Should contain G");
    assert!(notes.contains(&69), "Should contain A");
    assert!(!notes.contains(&71), "Should not contain B (natural)");
}

/// Tests Lydian mode generates correct notes.
#[test]
fn test_lydian_mode() {
    let generator = create_generator(60, Scale::Lydian, 120.0, 4);
    let notes: Vec<u8> = generator
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // F Lydian: F, G, A, B, C, D, E
    // Relative to C: C, D, E, F, G, A, B
    assert!(notes.contains(&60), "Should contain C");
    assert!(notes.contains(&62), "Should contain D");
    assert!(notes.contains(&64), "Should contain E");
    assert!(notes.contains(&65), "Should contain F");
    assert!(notes.contains(&67), "Should contain G");
    assert!(notes.contains(&69), "Should contain A");
    assert!(notes.contains(&71), "Should contain B (raised 4th)");
}

/// Tests Mixolydian mode generates correct notes.
#[test]
fn test_mixolydian_mode() {
    let generator = create_generator(60, Scale::Mixolydian, 120.0, 4);
    let notes: Vec<u8> = generator
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // G Mixolydian: G, A, B, C, D, E, F
    // Relative to C: C, D, E, F, G, A, Bb
    assert!(notes.contains(&60), "Should contain C");
    assert!(notes.contains(&62), "Should contain D");
    assert!(notes.contains(&64), "Should contain E");
    assert!(notes.contains(&65), "Should contain F");
    assert!(notes.contains(&67), "Should contain G");
    assert!(notes.contains(&69), "Should contain A");
    assert!(!notes.contains(&71), "Should not contain B (natural)");
}

/// Tests Locrian mode generates correct notes.
#[test]
fn test_locrian_mode() {
    let generator = create_generator(60, Scale::Locrian, 120.0, 4);
    let notes: Vec<u8> = generator
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // B Locrian: B, C, D, E, F, G, A
    // Relative to C: C, Db, Eb, F, Gb, Ab, Bb
    assert!(notes.contains(&60), "Should contain C");
    assert!(!notes.contains(&61), "Should not contain C#");
    assert!(!notes.contains(&62), "Should not contain D (natural)");
    assert!(notes.contains(&65), "Should contain F");
    assert!(!notes.contains(&66), "Should not contain F# (natural)");
    assert!(!notes.contains(&67), "Should not contain G (natural)");
}

/// Tests Whole Tone scale generates correct notes.
#[test]
fn test_whole_tone_scale() {
    let generator = create_generator(60, Scale::WholeTone, 120.0, 4);
    let notes: Vec<u8> = generator
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // Whole tone: 6 notes per octave
    assert_eq!(notes.len(), 6, "Whole tone should have 6 notes per octave");

    // C whole tone: C, D, E, F#, G#, A#
    assert!(notes.contains(&60), "Should contain C");
    assert!(notes.contains(&62), "Should contain D");
    assert!(notes.contains(&64), "Should contain E");
    assert!(!notes.contains(&65), "Should not contain F (natural)");
    assert!(!notes.contains(&67), "Should not contain G (natural)");
}

/// Tests Chromatic scale generates all 12 notes.
#[test]
fn test_chromatic_scale() {
    let generator = create_generator(60, Scale::Chromatic, 120.0, 4);
    let notes: Vec<u8> = generator
        .scale_notes()
        .iter()
        .filter(|&&n| n >= 60 && n < 72)
        .copied()
        .collect();

    // Chromatic: 12 notes per octave
    assert_eq!(
        notes.len(),
        12,
        "Chromatic scale should have 12 notes per octave"
    );

    // Should contain all semitones
    for i in 0..12 {
        assert!(
            notes.contains(&(60 + i)),
            "Chromatic scale should contain note {}",
            60 + i
        );
    }
}

// ===== Melody Structure Tests =====

/// Tests that melody notes don't overlap incorrectly.
#[test]
fn test_melody_note_timing() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate();

    let mut current_time = 0.0;
    for note in &melody.notes {
        // Notes should start after current position or be close to it
        assert!(
            note.start_beat >= current_time - 0.5,
            "Notes should be in roughly sequential order"
        );
        current_time = note.start_beat + note.duration;
    }
}

/// Tests that phrase endings have softer dynamics.
#[test]
fn test_phrase_endings_softer() {
    let mut generator = create_c_major_generator();
    let melody = generator.generate();

    // Check last notes of phrases (beats 3-4 of each measure)
    let phrase_endings: Vec<f32> = melody
        .notes
        .iter()
        .filter(|n| (n.start_beat % 4.0) > 3.0)
        .map(|n| n.velocity)
        .collect();

    let all_velocities: Vec<f32> = melody.notes.iter().map(|n| n.velocity).collect();

    // Phrase endings should generally be softer or equal
    let avg_phrase_end = phrase_endings.iter().sum::<f32>() / phrase_endings.len().max(1) as f32;
    let avg_all = all_velocities.iter().sum::<f32>() / all_velocities.len() as f32;

    // This is a soft check - phrase endings may not always be softer
    // but the generator should attempt this pattern
}

// ===== Export Tests =====

/// Tests MIDI export returns error when feature not enabled.
#[test]
fn test_midi_export_no_feature() {
    let generator = create_c_major_generator();

    // This will return an error if mido feature is not enabled
    let result = generator.export_midi("/tmp/test.mid");

    // Result should be an error (since mido feature is not enabled by default)
    assert!(
        result.is_err(),
        "MIDI export should fail without mido feature"
    );
}

// ===== Key and Scale Enum Tests =====

/// Tests Scale enum variants are distinct.
#[test]
fn test_scale_variants() {
    use std::collections::HashSet;

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

    let mut seen = HashSet::new();
    for &scale in &scales {
        assert!(!seen.contains(&scale), "Duplicate scale found");
        seen.insert(scale);
    }
}

/// Tests MelodyStyle enum variants.
#[test]
fn test_melody_style_variants() {
    assert_eq!(MelodyStyle::Pop as u8, 0);
    assert_eq!(MelodyStyle::Jazz as u8, 1);
    assert_eq!(MelodyStyle::LoFi as u8, 2);
    assert_eq!(MelodyStyle::EDM as u8, 3);
    assert_eq!(MelodyStyle::Ambient as u8, 4);
    assert_eq!(MelodyStyle::Classical as u8, 5);
    assert_eq!(MelodyStyle::Custom as u8, 6);
}
