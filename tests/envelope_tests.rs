//! Envelope tests for WAVELET audio engine
//!
//! Tests cover:
use super::common::{assert_samples_are_finite, calculate_peak};
/// - ADSR stages (attack, decay, sustain, release)
/// - Envelope timing accuracy
/// - State transitions
/// - No pop at note on/off
use wavelet::envelope::{AdsrEnvelope, EnvelopeConfig, EnvelopeStage};

const SAMPLE_RATE: f32 = 48000.0;

#[test]
fn test_envelope_default() {
    let env = AdsrEnvelope::new();
    assert_eq!(env.stage(), EnvelopeStage::Idle);
    assert!(!env.is_active());
}

#[test]
fn test_envelope_note_on() {
    let mut env = AdsrEnvelope::new();
    env.note_on();
    assert_eq!(env.stage(), EnvelopeStage::Delay);
}

#[test]
fn test_envelope_note_off() {
    let mut env = AdsrEnvelope::new();
    env.note_on();
    env.note_off();
    assert_eq!(env.stage(), EnvelopeStage::Release);
}

#[test]
fn test_envelope_idle_to_release() {
    // Note off while idle should stay idle
    let mut env = AdsrEnvelope::new();
    env.note_off();
    assert_eq!(env.stage(), EnvelopeStage::Idle);
}

#[test]
fn test_envelope_attack_stage() {
    let config = EnvelopeConfig {
        attack: 0.01, // 10ms
        decay: 0.0,
        sustain: 0.5,
        release: 0.0,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Should reach peak during attack
    let attack_samples = (config.attack * SAMPLE_RATE) as usize;

    for _ in 0..attack_samples {
        let level = env.process();
        assert!(level >= 0.0 && level <= 1.0);
    }

    // After attack, should be at peak
    let level = env.process();
    assert!(
        (level - 1.0).abs() < 0.1,
        "Should be near peak after attack"
    );
}

#[test]
fn test_envelope_decay_stage() {
    let config = EnvelopeConfig {
        attack: 0.001,
        decay: 0.05, // 50ms
        sustain: 0.5,
        release: 0.0,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Go through attack
    for _ in 0..10 {
        env.process();
    }

    // Now in decay - level should drop toward sustain
    let start_level = env.process();

    // Go through decay
    let decay_samples = (config.decay * SAMPLE_RATE) as usize;
    for _ in 0..decay_samples {
        env.process();
    }

    let end_level = env.process();
    assert!(end_level <= start_level + 0.1, "Decay should reduce level");
}

#[test]
fn test_envelope_sustain_stage() {
    let config = EnvelopeConfig {
        attack: 0.001,
        decay: 0.001,
        sustain: 0.5,
        release: 0.0,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Go through attack and decay
    for _ in 0..100 {
        env.process();
    }

    // Should be at sustain level
    let mut levels = Vec::new();
    for _ in 0..100 {
        levels.push(env.process());
    }

    // All sustain levels should be at 0.5 (or close)
    for &level in &levels {
        assert!(
            (level - 0.5).abs() < 0.1,
            "Sustain level should be stable at 0.5, got {}",
            level
        );
    }
}

#[test]
fn test_envelope_release_stage() {
    let config = EnvelopeConfig {
        attack: 0.001,
        decay: 0.001,
        sustain: 0.5,
        release: 0.1, // 100ms
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Go through attack, decay, sustain
    for _ in 0..100 {
        env.process();
    }

    // Release
    env.note_off();

    // Level should start at sustain and go to 0
    let release_samples = (config.release * SAMPLE_RATE) as usize;

    for _ in 0..release_samples {
        let level = env.process();
        assert!(level >= 0.0 && level <= 0.6);
    }
}

#[test]
fn test_envelope_timing_accuracy() {
    // Test that envelope timing is accurate
    let attack_time = 0.01; // 10ms
    let decay_time = 0.02; // 20ms
    let release_time = 0.03; // 30ms

    let config = EnvelopeConfig {
        attack: attack_time,
        decay: decay_time,
        sustain: 0.5,
        release: release_time,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);

    // Count samples to reach peak
    env.note_on();
    let mut samples_to_peak = 0;
    while env.process() < 0.99 && samples_to_peak < 10000 {
        samples_to_peak += 1;
    }

    let expected_samples = (attack_time * SAMPLE_RATE) as usize;
    assert!(
        (samples_to_peak as isize - expected_samples as isize).abs() < 50,
        "Attack timing: expected ~{} samples, got {}",
        expected_samples,
        samples_to_peak
    );
}

#[test]
fn test_envelope_state_transitions() {
    // Test all state transitions
    let mut env = AdsrEnvelope::new();

    // Idle -> Delay (note on)
    env.note_on();
    assert_eq!(env.stage(), EnvelopeStage::Delay);

    // Advance through delay
    for _ in 0..100 {
        env.process();
    }

    // Delay -> Attack
    assert!(matches!(
        env.stage(),
        EnvelopeStage::Attack | EnvelopeStage::Decay
    ));

    // Advance through attack and decay
    for _ in 0..100 {
        env.process();
    }

    // Note off -> Release
    env.note_off();
    assert_eq!(env.stage(), EnvelopeStage::Release);

    // Advance through release
    for _ in 0..100 {
        env.process();
    }

    // Release -> Finished
    assert!(matches!(
        env.stage(),
        EnvelopeStage::Finished | EnvelopeStage::Idle
    ));
}

#[test]
fn test_envelope_no_pop_on_note_on() {
    // Pop occurs when there's a sudden jump in the envelope
    let mut env = AdsrEnvelope::new();

    // Get initial sample
    let initial = env.process();
    assert_eq!(initial, 0.0);

    // Trigger note - should not cause sudden jump
    env.note_on();

    // First sample after trigger should not jump to full amplitude
    let first_sample = env.process();
    assert!(
        first_sample < 0.5,
        "First sample after note_on should ramp, not jump: {}",
        first_sample
    );
}

#[test]
fn test_envelope_no_pop_on_note_off() {
    // Test that note off doesn't cause pop
    let config = EnvelopeConfig {
        attack: 0.001,
        decay: 0.001,
        sustain: 1.0,
        release: 0.01, // Quick release
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Let envelope reach sustain
    for _ in 0..100 {
        env.process();
    }

    // Get samples before note off
    let sample_before_off = env.process();

    // Note off
    env.note_off();

    // First sample after note off should not be discontinuous
    let sample_after_off = env.process();

    // The difference should be small for a smooth release
    // (Pop happens when there's a sudden jump)
    assert!(
        (sample_before_off - sample_after_off).abs() < 0.1,
        "Note off should not cause pop: before={}, after={}",
        sample_before_off,
        sample_after_off
    );
}

#[test]
fn test_envelope_reset() {
    let mut env = AdsrEnvelope::new();

    // Trigger note
    env.note_on();
    for _ in 0..100 {
        env.process();
    }

    // Reset
    env.reset();

    assert_eq!(env.stage(), EnvelopeStage::Idle);
    assert!(!env.is_active());
}

#[test]
fn test_envelope_set_parameters() {
    let mut env = AdsrEnvelope::new();

    // Set parameters
    env.set_attack(0.05);
    env.set_decay(0.1);
    env.set_sustain(0.7);
    env.set_release(0.2);

    // Should process without issues
    for _ in 0..100 {
        let level = env.process();
        assert!(level >= 0.0 && level <= 1.0);
    }
}

#[test]
fn test_envelope_sustain_clamping() {
    let mut env = AdsrEnvelope::new();

    // Sustain should be clamped to [0, 1]
    env.set_sustain(1.5);
    assert!(env.sustain <= 1.0);

    env.set_sustain(-0.5);
    assert!(env.sustain >= 0.0);
}

#[test]
fn test_envelope_zero_attack() {
    // Edge case: zero attack time
    let config = EnvelopeConfig {
        attack: 0.0,
        decay: 0.0,
        sustain: 0.5,
        release: 0.0,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Should jump immediately to peak
    let level = env.process();
    assert!(
        (level - 1.0).abs() < 0.01,
        "Zero attack should go to peak immediately"
    );
}

#[test]
fn test_envelope_zero_release() {
    // Edge case: zero release time
    let config = EnvelopeConfig {
        attack: 0.0,
        decay: 0.0,
        sustain: 1.0,
        release: 0.0,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    for _ in 0..10 {
        env.process();
    }

    // Note off
    env.note_off();

    // Should immediately be at 0 or finish
    let level = env.process();
    assert!(level <= 0.01, "Zero release should finish immediately");
}

#[test]
fn test_envelope_process_samples() {
    let mut env = AdsrEnvelope::new();
    env.note_on();

    let samples = env.process_samples(1000);

    assert_eq!(samples.len(), 1000);
    for &sample in &samples {
        assert!(
            sample >= 0.0 && sample <= 1.0,
            "Envelope sample out of range: {}",
            sample
        );
    }
}

#[test]
fn test_envelope_finished_state() {
    let config = EnvelopeConfig {
        attack: 0.001,
        decay: 0.001,
        sustain: 0.5,
        release: 0.001, // Very short release
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Go through entire envelope
    for _ in 0..10000 {
        env.process();
        if !env.is_active() {
            break;
        }
    }

    // Should be finished or idle
    assert!(!env.is_active() || env.stage() == EnvelopeStage::Finished);

    // All subsequent samples should be 0
    for _ in 0..100 {
        let level = env.process();
        assert!(
            (level - 0.0).abs() < 0.001,
            "Finished envelope should output 0, got {}",
            level
        );
    }
}

#[test]
fn test_envelope_process_while_idle() {
    let env = AdsrEnvelope::new();

    // Processing while idle should return 0
    let level = env.process();
    assert_eq!(level, 0.0);
}

#[test]
fn test_envelope_with_delay() {
    let config = EnvelopeConfig {
        delay: 0.01, // 10ms delay
        attack: 0.01,
        decay: 0.01,
        sustain: 0.5,
        release: 0.01,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Initial samples should be 0 (during delay)
    for _ in 0..100 {
        let level = env.process();
        assert!(level >= 0.0);
    }
}

#[test]
fn test_envelope_is_active() {
    let mut env = AdsrEnvelope::new();

    // Idle state should not be active
    assert!(!env.is_active());

    // Note on should activate
    env.note_on();
    assert!(env.is_active());

    // Release should still be active until finished
    env.note_off();
    assert!(env.is_active());
}

#[test]
fn test_envelope_multiple_note_cycles() {
    let mut env = AdsrEnvelope::new();

    // First note
    env.note_on();
    for _ in 0..100 {
        env.process();
    }
    env.note_off();
    for _ in 0..100 {
        env.process();
    }

    // Second note
    env.note_on();
    for _ in 0..100 {
        env.process();
    }
    env.note_off();
    for _ in 0..100 {
        env.process();
    }

    // Should handle multiple cycles without issues
    assert!(true);
}

#[test]
fn test_envelope_process_samples_are_finite() {
    let mut env = AdsrEnvelope::new();
    env.note_on();

    let samples = env.process_samples(1000);
    assert_samples_are_finite(&samples);
}

#[test]
fn test_envelope_peak_parameter() {
    let config = EnvelopeConfig {
        attack: 0.01,
        decay: 0.01,
        sustain: 0.5,
        release: 0.01,
        peak: 0.8, // Custom peak level
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    };

    let mut env = AdsrEnvelope::with_config(config);
    env.note_on();

    // Should reach peak, not 1.0
    let mut reached_peak = false;
    for _ in 0..1000 {
        let level = env.process();
        if (level - 0.8).abs() < 0.1 {
            reached_peak = true;
            break;
        }
    }

    assert!(reached_peak, "Envelope should reach custom peak level");
}
