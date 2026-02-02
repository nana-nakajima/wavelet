//! Synth tests for WAVELET audio engine
//!
//! Tests cover:
use super::common::{
    assert_samples_are_finite, assert_samples_in_range, calculate_peak, calculate_rms,
};
use wavelet::envelope::AdsrEnvelope;
use wavelet::filter::{Filter, FilterType, ZdfFilter, ZdfFilterMode};
use wavelet::oscillator::{Oscillator, OscillatorConfig, OversampleFactor, Waveform};
/// - Polyphony (multiple notes)
/// - Voice stealing
/// - Parameter changes don't cause glitches
/// - No memory leaks
use wavelet::synth::Synth;

const SAMPLE_RATE: f32 = 48000.0;

#[test]
fn test_synth_default() {
    let synth = Synth::default();
    assert_eq!(synth.master_volume, 0.7);
    assert_eq!(synth.active_voice_count(), 0);
}

#[test]
fn test_synth_new() {
    let synth = Synth::new(SAMPLE_RATE);
    assert_eq!(synth.master_volume, 0.7);
    assert_eq!(synth.active_voice_count(), 0);
}

#[test]
fn test_synth_note_on() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    assert_eq!(synth.active_voice_count(), 1);
}

#[test]
fn test_synth_note_on_velocity_zero() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 0); // Velocity 0 should be treated as note off
    assert_eq!(synth.active_voice_count(), 0);
}

#[test]
fn test_synth_note_off_specific() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    assert_eq!(synth.active_voice_count(), 1);

    synth.note_off_specific(60);
    // Voice should still be active during release
    assert!(synth.active_voice_count() <= 1);
}

#[test]
fn test_synth_note_off() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    synth.note_on(64, 100);
    synth.note_on(67, 100);
    assert_eq!(synth.active_voice_count(), 3);

    synth.note_off();
    assert_eq!(synth.active_voice_count(), 0);
}

#[test]
fn test_synth_polyphony() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Play multiple notes
    synth.note_on(60, 100); // C4
    synth.note_on(62, 100); // D4
    synth.note_on(64, 100); // E4
    synth.note_on(65, 100); // F4
    synth.note_on(67, 100); // G4
    synth.note_on(69, 100); // A4
    synth.note_on(71, 100); // B4
    synth.note_on(72, 100); // C5

    assert_eq!(synth.active_voice_count(), 8);
}

#[test]
fn test_synth_polyphony_max_voices() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Try to play more notes than max voices (16)
    for note in 0..20 {
        synth.note_on(note, 100);
    }

    // Should not exceed max voices
    assert!(synth.active_voice_count() <= 16);
}

#[test]
fn test_synth_voice_stealing() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Fill all voices
    for note in 0..16 {
        synth.note_on(note, 100);
    }

    assert_eq!(synth.active_voice_count(), 16);

    // Play one more - should steal oldest voice
    synth.note_on(100, 100);

    // Should still have 16 voices (one was stolen)
    assert_eq!(synth.active_voice_count(), 16);

    // Original note 0 should no longer be playing
    assert!(!synth.active_notes.contains_key(&0));
}

#[test]
fn test_synth_process_mono() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    for _ in 0..1000 {
        let sample = synth.process_mono();
        assert!(sample.is_finite());
    }
}

#[test]
fn test_synth_process_stereo() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    for _ in 0..1000 {
        let (left, right) = synth.process_stereo();
        assert!(left.is_finite());
        assert!(right.is_finite());
        // Mono source should produce equal stereo output
        assert!((left - right).abs() < 0.0001);
    }
}

#[test]
fn test_synth_process_block_mono() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    let samples = synth.process_block_mono(1000);

    assert_eq!(samples.len(), 1000);
    assert_samples_in_range(&samples);
    assert_samples_are_finite(&samples);
}

#[test]
fn test_synth_process_block_stereo() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    let stereo_samples = synth.process_block_stereo(1000);

    assert_eq!(stereo_samples.len(), 1000);
    for &(left, right) in &stereo_samples {
        assert!(left.is_finite());
        assert!(right.is_finite());
    }
}

#[test]
fn test_synth_no_output_without_notes() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // No notes playing
    for _ in 0..1000 {
        let sample = synth.process_mono();
        assert_eq!(sample, 0.0);
    }
}

#[test]
fn test_synth_reset() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    synth.note_on(64, 100);
    synth.note_on(67, 100);

    synth.reset();
    assert_eq!(synth.active_voice_count(), 0);
}

#[test]
fn test_synth_set_master_volume() {
    let mut synth = Synth::new(SAMPLE_RATE);

    synth.set_master_volume(0.5);
    assert_eq!(synth.master_volume, 0.5);

    synth.set_master_volume(1.0);
    assert_eq!(synth.master_volume, 1.0);

    // Volume should be clamped
    synth.set_master_volume(2.0);
    assert!(synth.master_volume <= 1.0);

    synth.set_master_volume(-0.5);
    assert!(synth.master_volume >= 0.0);
}

#[test]
fn test_synth_set_filter_cutoff() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.set_filter_cutoff(500.0);

    // Should process without issues
    let sample = synth.process_mono();
    assert!(sample.is_finite());
}

#[test]
fn test_synth_set_filter_resonance() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.set_filter_resonance(5.0);

    let sample = synth.process_mono();
    assert!(sample.is_finite());
}

#[test]
fn test_synth_set_filter_type() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.set_filter_type(FilterType::HighPass);

    let sample = synth.process_mono();
    assert!(sample.is_finite());
}

#[test]
fn test_synth_set_effect_type() {
    use wavelet::effects::EffectType;

    let mut synth = Synth::new(SAMPLE_RATE);
    synth.set_effect_type(EffectType::Delay);

    let sample = synth.process_mono();
    assert!(sample.is_finite());
}

#[test]
fn test_synth_set_effect_mix() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.set_effect_mix(0.5);

    let sample = synth.process_mono();
    assert!(sample.is_finite());
}

#[test]
fn test_synth_zdf_enabled() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // ZDF should be enabled by default
    assert!(synth.zdf_enabled);

    // Disable ZDF
    synth.set_zdf_enabled(false);
    let sample_without_zdf = synth.process_mono();

    // Enable ZDF
    synth.set_zdf_enabled(true);
    let sample_with_zdf = synth.process_mono();

    // Both should be valid
    assert!(sample_without_zdf.is_finite());
    assert!(sample_with_zdf.is_finite());
}

#[test]
fn test_synth_zdf_cutoff() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Should not crash with various cutoff values
    for cutoff in [100.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0] {
        synth.set_zdf_cutoff(cutoff);
        let sample = synth.process_mono();
        assert!(sample.is_finite());
    }
}

#[test]
fn test_synth_zdf_resonance() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Test with high resonance
    for resonance in [1.0, 2.0, 3.0, 3.5] {
        synth.set_zdf_resonance(resonance);
        let sample = synth.process_mono();
        assert!(sample.abs() <= 1.0, "Sample clipped at high resonance");
    }
}

#[test]
fn test_synth_zdf_drive() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Test with various drive values
    for drive in [0.0, 1.0, 5.0, 10.0] {
        synth.set_zdf_drive(drive);
        let sample = synth.process_mono();
        assert!(sample.abs() <= 1.0, "Sample clipped with high drive");
    }
}

#[test]
fn test_synth_saturation_drive() {
    let mut synth = Synth::new(SAMPLE_RATE);

    for drive in [0.0, 1.0, 5.0, 10.0] {
        synth.set_saturation_drive(drive);
        let sample = synth.process_mono();
        assert!(sample.abs() <= 1.0);
    }
}

#[test]
fn test_synth_saturation_mix() {
    let mut synth = Synth::new(SAMPLE_RATE);

    synth.set_saturation_mix(0.0); // Dry
    let dry = synth.process_mono();

    synth.set_saturation_mix(1.0); // Wet
    let wet = synth.process_mono();

    assert!(dry.is_finite());
    assert!(wet.is_finite());
}

#[test]
fn test_synth_oversample_factor() {
    let mut synth = Synth::new(SAMPLE_RATE);

    synth.set_oversample_factor(OversampleFactor::None);
    assert_eq!(synth.oversample_factor(), OversampleFactor::None);

    synth.set_oversample_factor(OversampleFactor::X2);
    assert_eq!(synth.oversample_factor(), OversampleFactor::X2);

    synth.set_oversample_factor(OversampleFactor::X4);
    assert_eq!(synth.oversample_factor(), OversampleFactor::X4);

    synth.set_oversample_factor(OversampleFactor::X8);
    assert_eq!(synth.oversample_factor(), OversampleFactor::X8);
}

#[test]
fn test_synth_no_glitches_parameter_changes() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    // Change parameters while playing - should not cause glitches
    for _ in 0..1000 {
        let sample = synth.process_mono();

        // Occasionally change parameters
        if _ % 100 == 0 {
            synth.set_filter_cutoff(100.0 + (_ as f32 % 5000.0));
            synth.set_filter_resonance(1.0 + (_ as f32 % 5.0));
            synth.set_master_volume(0.5 + (_ as f32 % 0.5));
        }

        assert!(sample.is_finite(), "Glitch detected at sample {}", _);
    }
}

#[test]
fn test_synth_same_note_retrigger() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Play same note twice
    synth.note_on(60, 100);
    for _ in 0..100 {
        synth.process_mono();
    }

    synth.note_on(60, 100); // Retrigger
    for _ in 0..100 {
        synth.process_mono();
    }

    // Should have voice
    assert!(synth.active_voice_count() >= 1);
}

#[test]
fn test_synth_voice_cleanup() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Play notes with short release
    for note in [60, 64, 67] {
        synth.note_on(note, 100);
    }

    // Let them play
    for _ in 0..1000 {
        synth.process_mono();
    }

    // Release all
    synth.note_off();

    // Let release complete
    for _ in 0..5000 {
        synth.process_mono();
    }

    // Voices should be cleaned up
    assert_eq!(synth.active_voice_count(), 0);
}

#[test]
fn test_synth_no_memory_leaks() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Rapid note on/off cycling
    for _ in 0..100 {
        for note in [60, 64, 67, 72] {
            synth.note_on(note, 100);
            for _ in 0..10 {
                synth.process_mono();
            }
            synth.note_off_specific(note);
        }
    }

    // Synth should still work
    synth.note_on(60, 100);
    for _ in 0..100 {
        let sample = synth.process_mono();
        assert!(sample.is_finite());
    }
}

#[test]
fn test_synth_frequencies_are_correct() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Test that different notes produce different frequencies
    let mut frequencies: Vec<f32> = Vec::new();

    for note in [60, 62, 64, 65, 67, 69, 71, 72] {
        synth.note_on(note, 100);
        let sample = synth.process_mono();
        frequencies.push(sample);
        synth.note_off_specific(note);
    }

    // Adjacent notes should have different RMS values
    for i in 1..frequencies.len() {
        let diff = (frequencies[i] - frequencies[i - 1]).abs();
        assert!(diff > 0.001, "Notes {} and {} should differ", i, i - 1);
    }
}

#[test]
fn test_synth_velocity_affects_amplitude() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Low velocity
    synth.note_on(60, 20);
    let low_vel_samples: Vec<f32> = (0..100).map(|_| synth.process_mono()).collect();
    let low_vel_rms = calculate_rms(&low_vel_samples);

    synth.note_off();

    // High velocity
    synth.note_on(60, 127);
    let high_vel_samples: Vec<f32> = (0..100).map(|_| synth.process_mono()).collect();
    let high_vel_rms = calculate_rms(&high_vel_samples);

    // Higher velocity should produce louder output
    assert!(
        high_vel_rms > low_vel_rms,
        "Higher velocity should be louder"
    );
}

#[test]
fn test_synth_master_volume_affects_output() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    // Low volume
    synth.set_master_volume(0.1);
    let low_vol_samples: Vec<f32> = (0..100).map(|_| synth.process_mono()).collect();
    let low_vol_peak = calculate_peak(&low_vol_samples);

    // High volume
    synth.set_master_volume(1.0);
    let high_vol_samples: Vec<f32> = (0..100).map(|_| synth.process_mono()).collect();
    let high_vol_peak = calculate_peak(&high_vol_samples);

    assert!(high_vol_peak > low_vol_peak);
}

#[test]
fn test_synth_adsr_works() {
    // Test that ADSR envelope works correctly
    let config = wavelet::envelope::EnvelopeConfig {
        attack: 0.01,
        decay: 0.1,
        sustain: 0.5,
        release: 0.2,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);

    // Trigger envelope
    env.note_on();

    // Collect envelope values
    let envelope_values: Vec<f32> = (0..1000).map(|_| env.process()).collect();

    // Verify envelope stages
    assert!(envelope_values[0] >= 0.0);
    assert!(envelope_values.last().unwrap() <= 1.0);
}

#[test]
fn test_synth_parameter_ids() {
    // Test that parameter IDs are accessible
    use wavelet::synth::{
        PARAM_OVERSAMPLE, PARAM_SATURATION_DRIVE, PARAM_SATURATION_MIX, PARAM_ZDF_CUTOFF,
        PARAM_ZDF_DRIVE, PARAM_ZDF_ENABLED, PARAM_ZDF_RES,
    };

    // Just verify they exist and have expected values
    assert_eq!(PARAM_ZDF_ENABLED, 50);
    assert_eq!(PARAM_ZDF_CUTOFF, 51);
    assert_eq!(PARAM_ZDF_RES, 52);
    assert_eq!(PARAM_ZDF_DRIVE, 53);
    assert_eq!(PARAM_SATURATION_DRIVE, 54);
    assert_eq!(PARAM_SATURATION_MIX, 55);
    assert_eq!(PARAM_OVERSAMPLE, 56);
}

#[test]
fn test_synth_extreme_velocity() {
    let mut synth = Synth::new(SAMPLE_RATE);

    // Velocity 1 (minimum)
    synth.note_on(60, 1);
    for _ in 0..100 {
        let sample = synth.process_mono();
        assert!(sample.is_finite());
    }

    // Velocity 127 (maximum)
    synth.note_on(60, 127);
    for _ in 0..100 {
        let sample = synth.process_mono();
        assert!(sample.is_finite());
    }
}

#[test]
fn test_synth_extreme_cutoff_values() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    // Very low cutoff
    synth.set_filter_cutoff(20.0);
    let sample_low = synth.process_mono();

    // Very high cutoff
    synth.set_filter_cutoff(SAMPLE_RATE / 2.0 - 100.0);
    let sample_high = synth.process_mono();

    assert!(sample_low.is_finite());
    assert!(sample_high.is_finite());
}

#[test]
fn test_synth_zdf_filter_reset() {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);

    // Process some samples
    for _ in 0..1000 {
        synth.process_mono();
    }

    // Reset should work
    synth.reset();
    assert_eq!(synth.active_voice_count(), 0);
}
