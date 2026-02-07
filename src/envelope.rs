//! Envelope Module
//!
//! This module provides envelope generator implementations for controlling
//! the amplitude and modulation of synthesizer parameters over time.

#![allow(dead_code)] // Reserve envelope fields for future features

/// Enumeration of possible envelope stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeStage {
    /// Envelope is idle, at zero level
    Idle,

    /// Initial delay before attack phase
    Delay,

    /// Rising to peak level
    Attack,

    /// Falling to sustain level
    Decay,

    /// Holding at sustain level
    Sustain,

    /// Releasing back to zero
    Release,

    /// Envelope has finished
    Finished,
}

/// Configuration for envelope timing and levels.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeConfig {
    /// Attack time in seconds
    pub attack: f32,

    /// Decay time in seconds
    pub decay: f32,

    /// Sustain level (0.0 to 1.0)
    pub sustain: f32,

    /// Release time in seconds
    pub release: f32,

    /// Initial delay time in seconds (optional)
    pub delay: f32,

    /// Hold time at sustain level (optional)
    pub sustain_hold: f32,

    /// Peak level for attack/decay (typically 1.0)
    pub peak: f32,

    /// Sample rate for timing calculations
    pub sample_rate: f32,
}

impl Default for EnvelopeConfig {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
            delay: 0.0,
            sustain_hold: 0.0,
            peak: 1.0,
            sample_rate: 44100.0,
        }
    }
}

/// ADSR (Attack, Decay, Sustain, Release) envelope generator.
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    /// Current envelope stage
    stage: EnvelopeStage,

    /// Current envelope level
    current_level: f32,

    /// Envelope parameters in samples
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    delay: f32,
    sustain_hold: f32,
    peak: f32,

    /// Timing state
    stage_time: f32,
    sample_rate: f32,
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope with default configuration.
    pub fn new() -> Self {
        Self::with_config(EnvelopeConfig::default())
    }

    /// Creates a new ADSR envelope with custom configuration.
    pub fn with_config(config: EnvelopeConfig) -> Self {
        let samples_per_second = config.sample_rate;

        Self {
            stage: EnvelopeStage::Idle,
            current_level: 0.0,
            attack: config.attack * samples_per_second,
            decay: config.decay * samples_per_second,
            sustain: config.sustain,
            release: config.release * samples_per_second,
            delay: config.delay * samples_per_second,
            sustain_hold: config.sustain_hold * samples_per_second,
            peak: config.peak,
            stage_time: 0.0,
            sample_rate: config.sample_rate,
        }
    }

    /// Triggers the envelope (note on event).
    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Delay;
        self.stage_time = 0.0;
    }

    /// Releases the envelope (note off event).
    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle && self.stage != EnvelopeStage::Release {
            self.stage = EnvelopeStage::Release;
            self.stage_time = 0.0;
        }
    }

    /// Processes one sample of the envelope.
    pub fn process(&mut self) -> f32 {
        self.advance_stage();
        self.current_level
    }

    /// Processes multiple envelope samples.
    pub fn process_samples(&mut self, count: usize) -> Vec<f32> {
        (0..count).map(|_| self.process()).collect()
    }

    /// Gets the current envelope stage.
    pub fn stage(&self) -> EnvelopeStage {
        self.stage
    }

    /// Checks if the envelope is still active (not idle or finished).
    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle && self.stage != EnvelopeStage::Finished
    }

    /// Resets the envelope to idle state.
    pub fn reset(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.current_level = 0.0;
        self.stage_time = 0.0;
    }

    /// Sets the attack time.
    pub fn set_attack(&mut self, attack: f32) {
        self.attack = attack * self.sample_rate;
    }

    /// Sets the decay time.
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay * self.sample_rate;
    }

    /// Sets the sustain level.
    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain.clamp(0.0, 1.0);
    }

    /// Sets the release time.
    pub fn set_release(&mut self, release: f32) {
        self.release = release * self.sample_rate;
    }

    /// Internal method to advance the envelope stage.
    fn advance_stage(&mut self) {
        self.stage_time += 1.0;

        match self.stage {
            EnvelopeStage::Idle => {
                self.current_level = 0.0;
            }

            EnvelopeStage::Delay => {
                if self.stage_time >= self.delay {
                    self.stage = EnvelopeStage::Attack;
                    self.stage_time = 0.0;
                }
            }

            EnvelopeStage::Attack => {
                if self.attack > 0.0 {
                    self.current_level += (self.peak - 0.0) / self.attack;
                } else {
                    self.current_level = self.peak;
                }

                if self.current_level >= self.peak {
                    self.current_level = self.peak;
                    self.stage = EnvelopeStage::Decay;
                    self.stage_time = 0.0;
                }
            }

            EnvelopeStage::Decay => {
                if self.decay > 0.0 {
                    self.current_level += (self.sustain - self.peak) / self.decay;
                } else {
                    self.current_level = self.sustain;
                }

                if self.current_level <= self.sustain {
                    self.current_level = self.sustain;
                    self.stage = EnvelopeStage::Sustain;
                    self.stage_time = 0.0;
                }
            }

            EnvelopeStage::Sustain => {
                // Sustain level is held until note off
                self.current_level = self.sustain;

                if self.stage_time >= self.sustain_hold {
                    // Stay at sustain, waiting for note_off
                }
            }

            EnvelopeStage::Release => {
                if self.release > 0.0 {
                    self.current_level += (0.0 - self.sustain) / self.release;
                } else {
                    self.current_level = 0.0;
                }

                if self.current_level <= 0.0 {
                    self.current_level = 0.0;
                    self.stage = EnvelopeStage::Finished;
                }
            }

            EnvelopeStage::Finished => {
                self.current_level = 0.0;
            }
        }
    }
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

/// Envelope trait for polymorphic envelope usage.
pub trait Envelope {
    /// Process one sample.
    fn process(&mut self) -> f32;

    /// Trigger the envelope.
    fn trigger(&mut self);

    /// Release the envelope.
    fn release(&mut self);

    /// Check if envelope is active.
    fn is_active(&self) -> bool;
}

impl Envelope for AdsrEnvelope {
    fn process(&mut self) -> f32 {
        self.process()
    }

    fn trigger(&mut self) {
        self.note_on();
    }

    fn release(&mut self) {
        self.note_off();
    }

    fn is_active(&self) -> bool {
        self.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_env(attack: f32, decay: f32, sustain: f32, release: f32, sr: f32) -> AdsrEnvelope {
        AdsrEnvelope::with_config(EnvelopeConfig {
            attack,
            decay,
            sustain,
            release,
            delay: 0.0,
            sustain_hold: 0.0,
            peak: 1.0,
            sample_rate: sr,
        })
    }

    // --- Full ADSR cycle with level checks at stage boundaries ---
    #[test]
    fn test_full_adsr_cycle() {
        let sr = 1000.0;
        // 10ms attack = 10 samples, 20ms decay = 20 samples, 50ms release = 50 samples
        let mut env = make_env(0.01, 0.02, 0.5, 0.05, sr);

        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert_eq!(env.process(), 0.0);

        env.note_on();

        // First process transitions from Delay to Attack (delay=0)
        env.process();
        assert_eq!(env.stage(), EnvelopeStage::Attack);

        // Attack phase: should ramp from 0 to ~1.0 over ~10 samples
        let mut attack_samples = Vec::new();
        for _ in 0..10 {
            attack_samples.push(env.process());
        }
        // Should be monotonically increasing during attack
        for w in attack_samples.windows(2) {
            assert!(w[1] >= w[0], "Attack should be monotonically increasing");
        }

        // Decay phase: should ramp from peak down to sustain
        let mut decay_samples = Vec::new();
        for _ in 0..25 {
            decay_samples.push(env.process());
        }

        // Sustain phase: should hold at sustain level
        assert_eq!(env.stage(), EnvelopeStage::Sustain);
        for _ in 0..100 {
            let level = env.process();
            assert!(
                (level - 0.5).abs() < 0.01,
                "Sustain should hold at 0.5, got {}",
                level
            );
        }

        // Release phase
        env.note_off();
        assert_eq!(env.stage(), EnvelopeStage::Release);

        let mut release_samples = Vec::new();
        for _ in 0..50 {
            release_samples.push(env.process());
        }
        // Should be monotonically decreasing during release
        for w in release_samples.windows(2) {
            assert!(w[1] <= w[0] + 0.001, "Release should be decreasing");
        }

        // Should reach zero and finish
        for _ in 0..10 {
            env.process();
        }
        assert_eq!(env.stage(), EnvelopeStage::Finished);
        assert!(!env.is_active());
        assert_eq!(env.process(), 0.0);
    }

    // --- Zero attack: instant peak (takes one sample to transition) ---
    #[test]
    fn test_zero_attack_instant_peak() {
        let mut env = make_env(0.0, 0.1, 0.5, 0.1, 1000.0);
        env.note_on();
        env.process(); // Delay -> Attack transition
        let level = env.process(); // Attack with zero time -> sets peak and transitions to Decay
        assert!(
            (level - 1.0).abs() < 0.01,
            "Zero attack should reach peak within 2 samples, got {}",
            level
        );
    }

    // --- Zero decay: instant sustain ---
    #[test]
    fn test_zero_decay_instant_sustain() {
        let mut env = make_env(0.0, 0.0, 0.6, 0.1, 1000.0);
        env.note_on();
        env.process(); // Delay -> Attack
        env.process(); // Attack -> Decay (zero attack)
        let level = env.process(); // Decay -> Sustain (zero decay)
        assert!(
            (level - 0.6).abs() < 0.05,
            "Zero decay should reach sustain quickly, got {}",
            level
        );
    }

    // --- Zero release: instant silence ---
    #[test]
    fn test_zero_release_instant_silence() {
        let mut env = make_env(0.0, 0.0, 0.7, 0.0, 1000.0);
        env.note_on();
        // Get to sustain
        for _ in 0..5 {
            env.process();
        }
        env.note_off();
        let level = env.process();
        assert!(
            level < 0.01,
            "Zero release should go to silence immediately, got {}",
            level
        );
    }

    // --- note_off during attack ---
    #[test]
    fn test_note_off_during_attack() {
        let mut env = make_env(0.1, 0.1, 0.5, 0.05, 1000.0);
        env.note_on();
        // Process a few attack samples
        for _ in 0..5 {
            env.process();
        }
        // Release during attack
        env.note_off();
        assert_eq!(env.stage(), EnvelopeStage::Release);
        // Should start decreasing from current level
        let level_before = env.process();
        let level_after = env.process();
        assert!(
            level_after <= level_before,
            "Should decrease after note_off during attack"
        );
    }

    // --- Re-trigger during release ---
    #[test]
    fn test_retrigger_during_release() {
        let mut env = make_env(0.01, 0.01, 0.5, 0.1, 1000.0);
        env.note_on();
        for _ in 0..50 {
            env.process();
        }
        env.note_off();
        for _ in 0..10 {
            env.process();
        }
        // Re-trigger
        env.note_on();
        assert_eq!(env.stage(), EnvelopeStage::Delay);
        // Should start a new attack
        for _ in 0..15 {
            env.process();
        }
        let level = env.process();
        assert!(
            level > 0.3,
            "Re-trigger should start new attack, got {}",
            level
        );
    }

    // --- Delay phase ---
    #[test]
    fn test_delay_phase() {
        let mut env = AdsrEnvelope::with_config(EnvelopeConfig {
            attack: 0.01,
            decay: 0.01,
            sustain: 0.5,
            release: 0.01,
            delay: 0.05, // 50ms delay = 50 samples at 1000 Hz
            peak: 1.0,
            sustain_hold: 0.0,
            sample_rate: 1000.0,
        });

        env.note_on();
        // During delay, output should be 0
        for _ in 0..49 {
            let level = env.process();
            assert_eq!(level, 0.0, "During delay, level should be 0");
        }
        // After delay, should transition to attack
        for _ in 0..5 {
            env.process();
        }
        assert_eq!(env.stage(), EnvelopeStage::Attack);
    }

    // --- Reset returns to idle ---
    #[test]
    fn test_reset_to_idle() {
        let mut env = make_env(0.01, 0.01, 0.5, 0.01, 1000.0);
        env.note_on();
        for _ in 0..20 {
            env.process();
        }
        env.reset();
        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert!(!env.is_active());
        assert_eq!(env.process(), 0.0);
    }

    // --- process_samples matches individual ---
    #[test]
    fn test_process_samples_matches_individual() {
        let config = EnvelopeConfig {
            attack: 0.01,
            decay: 0.02,
            sustain: 0.5,
            release: 0.05,
            sample_rate: 1000.0,
            ..Default::default()
        };

        let mut env1 = AdsrEnvelope::with_config(config);
        env1.note_on();
        let individual: Vec<f32> = (0..100).map(|_| env1.process()).collect();

        let mut env2 = AdsrEnvelope::with_config(config);
        env2.note_on();
        let batch = env2.process_samples(100);

        for (i, (a, b)) in individual.iter().zip(batch.iter()).enumerate() {
            assert!((a - b).abs() < 1e-6, "Mismatch at {}: {} vs {}", i, a, b);
        }
    }

    // --- Parameter setters ---
    #[test]
    fn test_set_sustain_clamping() {
        let mut env = AdsrEnvelope::new();
        env.set_sustain(1.5);
        // Sustain should be clamped to 1.0
        env.note_on();
        for _ in 0..10000 {
            env.process();
        }
        let level = env.process();
        assert!(
            (level - 1.0).abs() < 0.01,
            "Sustain clamped to 1.0, got {}",
            level
        );
    }
}
