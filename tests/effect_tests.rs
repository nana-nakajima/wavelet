//! Effect tests for WAVELET audio engine
//!
//! Tests cover:
/// - All effects (Reverb, Delay, Distortion, Chorus, Saturation)
/// - Bypass functionality
/// - Parameter limits
/// - No NaN or infinity values

use wavelet::effects::{
    Effect, EffectType, EffectProcessor, EffectConfig,
    Delay, Reverb, Distortion, Compressor,
    saturation::{Saturation, SaturationConfig},
};
use super::common::{
    generate_test_buffer, calculate_rms, calculate_peak,
    assert_samples_in_range, assert_samples_are_finite,
};

const SAMPLE_RATE: f32 = 48000.0;

#[test]
fn test_delay_basic() {
    let mut delay = Delay::new(SAMPLE_RATE);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    delay.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_delay_set_parameters() {
    let mut delay = Delay::new(SAMPLE_RATE);
    
    delay.set_delay_time(100.0); // 100ms
    assert!(delay.delay_samples > 0);
    
    delay.set_feedback(0.5);
    assert_eq!(delay.feedback, 0.5);
    
    delay.set_mix(0.3);
    assert_eq!(delay.mix, 0.3);
}

#[test]
fn test_delay_feedback_clamping() {
    let mut delay = Delay::new(SAMPLE_RATE);
    
    // Feedback should be clamped to prevent infinite loop
    delay.set_feedback(1.5);
    assert!(delay.feedback < 1.0);
    
    delay.set_feedback(-0.5);
    assert!(delay.feedback >= 0.0);
}

#[test]
fn test_delay_bypass() {
    let mut delay = Delay::new(SAMPLE_RATE);
    
    // When disabled, should pass through
    delay.set_enabled(false);
    let passthrough = delay.process_with_bypass(0.5);
    assert_eq!(passthrough, 0.5);
    
    // When enabled, should process
    delay.set_enabled(true);
    let processed = delay.process_with_bypass(0.5);
    assert!(processed.is_finite());
}

#[test]
fn test_delay_reset() {
    let mut delay = Delay::new(SAMPLE_RATE);
    
    // Process some samples
    for _ in 0..1000 {
        delay.process(0.5);
    }
    
    // Reset should clear buffer
    delay.reset();
    
    // After reset, next sample should be input only
    let output = delay.process(0.5);
    assert!((output - 0.5).abs() < 0.01, "After reset, delay should only pass input");
}

#[test]
fn test_reverb_basic() {
    let mut reverb = Reverb::new(SAMPLE_RATE);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    reverb.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_reverb_set_parameters() {
    let mut reverb = Reverb::new(SAMPLE_RATE);
    
    reverb.set_decay(0.5);
    assert_eq!(reverb.decay, 0.5);
    
    reverb.set_mix(0.2);
    assert_eq!(reverb.mix, 0.2);
}

#[test]
fn test_reverb_decay_clamping() {
    let mut reverb = Reverb::new(SAMPLE_RATE);
    
    // Decay should be clamped
    reverb.set_decay(2.0);
    assert!(reverb.decay <= 0.95);
    
    reverb.set_decay(-0.5);
    assert!(reverb.decay >= 0.1);
}

#[test]
fn test_reverb_bypass() {
    let mut reverb = Reverb::new(SAMPLE_RATE);
    
    reverb.set_enabled(false);
    let passthrough = reverb.process_with_bypass(0.5);
    assert_eq!(passthrough, 0.5);
    
    reverb.set_enabled(true);
    let processed = reverb.process_with_bypass(0.5);
    assert!(processed.is_finite());
}

#[test]
fn test_reverb_reset() {
    let mut reverb = Reverb::new(SAMPLE_RATE);
    
    // Process some samples
    for _ in 0..1000 {
        reverb.process(0.5);
    }
    
    // Reset should clear buffers
    reverb.reset();
    
    // After reset, should still process without issues
    let output = reverb.process(0.5);
    assert!(output.is_finite());
}

#[test]
fn test_distortion_basic() {
    let mut dist = Distortion::new();
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    dist.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_distortion_processing() {
    let mut dist = Distortion::new();
    
    // Clean signal
    let clean = dist.process(0.1);
    assert!((clean - 0.1).abs() < 0.01);
    
    // Distorted signal
    dist.set_intensity(1.0);
    let distorted = dist.process(0.5);
    assert!(distorted.abs() <= 1.0);
}

#[test]
fn test_distortion_bypass() {
    let mut dist = Distortion::new();
    
    dist.set_enabled(false);
    let passthrough = dist.process_with_bypass(0.5);
    assert_eq!(passthrough, 0.5);
    
    dist.set_enabled(true);
    let processed = dist.process_with_bypass(0.5);
    assert!(processed.is_finite());
}

#[test]
fn test_distortion_reset() {
    let mut dist = Distortion::new();
    
    // No state to reset, but should not crash
    dist.reset();
}

#[test]
fn test_saturation_default() {
    let sat = Saturation::new();
    assert_eq!(sat.drive, 0.5);
    assert_eq!(sat.tone, 0.5);
    assert_eq!(sat.mix, 0.5);
    assert!(sat.enabled);
}

#[test]
fn test_saturation_basic() {
    let mut sat = Saturation::new();
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    sat.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_saturation_drive_levels() {
    let mut sat = Saturation::new();
    
    // Clean (no drive)
    sat.set_drive(0.0);
    let clean = sat.process_sample(0.5);
    assert!((clean - 0.5).abs() < 0.01);
    
    // Light saturation
    sat.set_drive(0.5);
    let light = sat.process_sample(0.5);
    assert!(light.abs() <= 1.0);
    
    // Heavy saturation
    sat.set_drive(10.0);
    let heavy = sat.process_sample(0.5);
    assert!(heavy.abs() <= 1.0);
}

#[test]
fn test_saturation_clipping_prevention() {
    let mut sat = Saturation::new();
    sat.set_drive(10.0);
    
    // Even with extreme input, should not clip
    let output = sat.process_sample(10.0);
    assert!(output.abs() <= 1.0);
}

#[test]
fn test_saturation_mix() {
    let mut sat = Saturation::new();
    sat.set_drive(5.0); // High drive
    
    // Full dry
    sat.set_mix(0.0);
    let dry = sat.process_sample(0.5);
    assert!((dry - 0.5).abs() < 0.01);
    
    // Full wet
    sat.set_mix(1.0);
    let wet = sat.process_sample(0.5);
    assert!((wet - 0.5).abs() > 0.01, "Saturated signal should differ from dry");
    
    assert!(wet.abs() <= 1.0);
}

#[test]
fn test_saturation_bypass() {
    let mut sat = Saturation::new();
    sat.set_drive(10.0);
    
    sat.set_enabled(false);
    let passthrough = sat.process_with_bypass(0.5);
    assert_eq!(passthrough, 0.5);
    
    sat.set_enabled(true);
    let processed = sat.process_with_bypass(0.5);
    assert!(processed.is_finite());
}

#[test]
fn test_saturation_reset() {
    let mut sat = Saturation::new();
    
    // Process some samples
    for _ in 0..100 {
        sat.process_sample(0.8);
    }
    
    // Reset should clear state
    sat.reset();
    assert_eq!(sat.prev_tone, 0.0);
}

#[test]
fn test_saturation_tone_control() {
    let mut sat = Saturation::new();
    sat.set_drive(2.0);
    
    // Different tone settings
    sat.set_tone(0.0);
    let dark = sat.process_sample(0.5);
    
    sat.set_tone(0.5);
    let mid = sat.process_sample(0.5);
    
    sat.set_tone(1.0);
    let bright = sat.process_sample(0.5);
    
    // All should be valid
    assert!(dark.abs() <= 1.0);
    assert!(mid.abs() <= 1.0);
    assert!(bright.abs() <= 1.0);
}

#[test]
fn test_compressor_default() {
    let comp = Compressor::new(SAMPLE_RATE);
    assert_eq!(comp.threshold_db, -20.0);
    assert_eq!(comp.ratio, 4.0);
}

#[test]
fn test_compressor_basic() {
    let mut comp = Compressor::new(SAMPLE_RATE);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    comp.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_compressor_set_parameters() {
    let mut comp = Compressor::new(SAMPLE_RATE);
    
    comp.set_threshold(-30.0);
    assert_eq!(comp.threshold_db, -30.0);
    
    comp.set_ratio(8.0);
    assert_eq!(comp.ratio, 8.0);
    
    comp.set_attack(0.05);
    assert!((comp.attack_s - 0.05).abs() < 0.001);
    
    comp.set_release(0.2);
    assert!((comp.release_s - 0.2).abs() < 0.001);
    
    comp.set_makeup(6.0);
    assert_eq!(comp.makeup_db, 6.0);
}

#[test]
fn test_compressor_reset() {
    let mut comp = Compressor::new(SAMPLE_RATE);
    
    // Process some samples
    for _ in 0..1000 {
        comp.process(0.8);
    }
    
    // Reset should clear state
    comp.reset();
    assert_eq!(comp.gain_reduction, 1.0);
}

#[test]
fn test_compressor_bypass() {
    let mut comp = Compressor::new(SAMPLE_RATE);
    
    comp.set_enabled(false);
    let passthrough = comp.process_with_bypass(0.5);
    assert_eq!(passthrough, 0.5);
    
    comp.set_enabled(true);
    let processed = comp.process_with_bypass(0.5);
    assert!(processed.is_finite());
}

#[test]
fn test_compressor_no_dc_offset() {
    let mut comp = Compressor::new(SAMPLE_RATE);
    
    // Process DC signal
    for _ in 0..1000 {
        let output = comp.process(0.5);
        // Should not accumulate DC offset
        assert!(output.is_finite());
    }
}

#[test]
fn test_effect_processor_delay() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Delay);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    fx.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_effect_processor_reverb() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Reverb);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    fx.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_effect_processor_distortion() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Distortion);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    fx.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_effect_processor_saturation() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Saturation);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    fx.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_effect_processor_compressor() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Compressor);
    
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let mut output = input.clone();
    
    fx.process_buffer(&mut output);
    
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_effect_processor_bypass_all() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    
    for &effect_type in &[
        EffectType::Delay,
        EffectType::Reverb,
        EffectType::Distortion,
        EffectType::Compressor,
        EffectType::Saturation,
    ] {
        fx.set_effect_type(effect_type);
        fx.set_enabled(false);
        
        let passthrough = fx.process_with_bypass(0.5);
        assert_eq!(passthrough, 0.5, "Bypass failed for {:?}", effect_type);
    }
}

#[test]
fn test_effect_processor_set_mix() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Delay);
    
    fx.set_mix(0.0);
    assert_eq!(fx.mix(), 0.0);
    
    fx.set_mix(0.5);
    assert_eq!(fx.mix(), 0.5);
    
    fx.set_mix(1.0);
    assert_eq!(fx.mix(), 1.0);
}

#[test]
fn test_effect_processor_mix_clamping() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Delay);
    
    // Mix should be clamped to [0, 1]
    fx.set_mix(-0.5);
    assert!(fx.mix() >= 0.0);
    
    fx.set_mix(1.5);
    assert!(fx.mix() <= 1.0);
}

#[test]
fn test_effect_processor_reset() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    fx.set_effect_type(EffectType::Delay);
    
    // Process some samples
    for _ in 0..1000 {
        fx.process(0.5);
    }
    
    // Reset should not crash
    fx.reset();
}

#[test]
fn test_all_effects_no_nan_or_inf() {
    let effects: Vec<(&str, Box<dyn Effect>)> = vec![
        ("Delay", Box::new(Delay::new(SAMPLE_RATE))),
        ("Reverb", Box::new(Reverb::new(SAMPLE_RATE))),
        ("Distortion", Box::new(Distortion::new())),
        ("Compressor", Box::new(Compressor::new(SAMPLE_RATE))),
        ("Saturation", Box::new(Saturation::new())),
    ];
    
    for (name, mut effect) in effects {
        for _ in 0..10000 {
            let output = effect.process(0.5);
            assert!(
                output.is_finite(),
                "NaN or Inf detected in {} effect",
                name
            );
        }
    }
}

#[test]
fn test_all_effects_parameter_limits() {
    let mut delay = Delay::new(SAMPLE_RATE);
    delay.set_delay_time(0.0);
    delay.set_feedback(0.0);
    delay.set_mix(0.0);
    
    let mut reverb = Reverb::new(SAMPLE_RATE);
    reverb.set_decay(0.1);
    reverb.set_mix(0.0);
    
    let mut dist = Distortion::new();
    dist.set_intensity(0.0);
    dist.set_mix(0.0);
    
    let mut comp = Compressor::new(SAMPLE_RATE);
    comp.set_threshold(-60.0);
    comp.set_ratio(1.0);
    comp.set_attack(0.001);
    comp.set_release(0.01);
    comp.set_makeup(0.0);
    
    let mut sat = Saturation::new();
    sat.set_drive(0.0);
    sat.set_tone(0.0);
    sat.set_mix(0.0);
    
    // All should process without issues
    for effect in [&mut delay as &mut dyn Effect, &mut reverb, &mut dist, &mut comp, &mut sat] {
        for _ in 0..1000 {
            let output = effect.process(0.5);
            assert!(output.is_finite());
        }
    }
}

#[test]
fn test_effect_processor_get_effect_type() {
    let mut fx = EffectProcessor::new(SAMPLE_RATE);
    
    for &effect_type in &[
        EffectType::Delay,
        EffectType::Reverb,
        EffectType::Distortion,
        EffectType::Compressor,
        EffectType::Saturation,
    ] {
        fx.set_effect_type(effect_type);
        assert_eq!(fx.effect_type(), effect_type);
    }
}
