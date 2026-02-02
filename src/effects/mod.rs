//! Effects Module
//!
//! This module provides audio effects processing for the synthesizer.
//! Effects add character and space to sounds, creating depth and interest.
//!
//! # Supported Effects
//!
//! - **Reverb**: Creates space and depth through room reflections
//! - **Delay**: Echo effect for rhythmic interest
//! - **Chorus**: Modulates delay for thickening effect
//! - **Distortion**: Adds harmonic content through clipping
//! - **Phaser**: Sweeping phase cancellation
//! - **Flanger**: Modulated comb filtering

use std::f32::consts::PI;

/// Enumeration of supported effect types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    /// Room reverb simulation
    Reverb,
    
    /// Echo/delay effect
    Delay,
    
    /// Chorus thickening effect
    Chorus,
    
    /// Harmonic distortion
    Distortion,
    
    /// Phaser effect
    Phaser,
    
    /// Flanger effect
    Flanger,
}

/// Configuration structure for effect parameters.
#[derive(Debug, Clone, Copy)]
pub struct EffectConfig {
    /// Type of effect
    pub effect_type: EffectType,
    
    /// Wet/dry mix (0.0 = dry, 1.0 = wet)
    pub mix: f32,
    
    /// Effect intensity parameter
    pub intensity: f32,
    
    /// Time parameter (for delay/reverb)
    pub time: f32,
    
    /// Feedback amount (for delay/reverb)
    pub feedback: f32,
    
    /// Rate parameter (for modulation effects)
    pub rate: f32,
    
    /// Depth parameter (for modulation effects)
    pub depth: f32,
    
    /// Sample rate
    pub sample_rate: f32,
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            effect_type: EffectType::Delay,
            mix: 0.3,
            intensity: 0.5,
            time: 0.3,
            feedback: 0.4,
            rate: 0.5,
            depth: 0.3,
            sample_rate: 44100.0,
        }
    }
}

/// Base trait for all effects.
pub trait Effect {
    /// Process a single audio sample.
    fn process(&mut self, input: f32) -> f32;
    
    /// Process a buffer of audio samples.
    fn process_buffer(&mut self, samples: &mut [f32]);
    
    /// Reset effect state.
    fn reset(&mut self);
    
    /// Set the wet/dry mix.
    fn set_mix(&mut self, mix: f32);
    
    /// Set effect intensity.
    fn set_intensity(&mut self, intensity: f32);
}

/// Simple delay effect with feedback.
#[derive(Debug, Clone)]
pub struct Delay {
    /// Delay buffer
    buffer: Vec<f32>,
    
    /// Current write position
    write_pos: usize,
    
    /// Current read position
    read_pos: usize,
    
    /// Delay time in samples
    delay_samples: usize,
    
    /// Feedback amount
    feedback: f32,
    
    /// Wet/dry mix
    mix: f32,
    
    /// Sample rate
    sample_rate: f32,
}

impl Delay {
    /// Creates a new delay effect.
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_ms = 2000.0; // 2 seconds max
        let max_delay_samples = (max_delay_ms / 1000.0 * sample_rate) as usize;
        
        Self {
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            read_pos: 0,
            delay_samples: (0.3 * sample_rate) as usize,
            feedback: 0.4,
            mix: 0.3,
            sample_rate,
        }
    }
    
    /// Sets the delay time.
    pub fn set_delay_time(&mut self, time_ms: f32) {
        self.delay_samples = (time_ms / 1000.0 * self.sample_rate) as usize;
        self.read_pos = (self.write_pos + self.buffer.len() - self.delay_samples) % self.buffer.len();
    }
    
    /// Sets the feedback amount.
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95);
    }
}

impl Effect for Delay {
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.read_pos];
        
        // Write input plus feedback to buffer
        self.buffer[self.write_pos] = input + delayed * self.feedback;
        
        // Advance positions
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        self.read_pos = (self.read_pos + 1) % self.buffer.len();
        
        // Mix wet and dry
        input * (1.0 - self.mix) + delayed * self.mix
    }
    
    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }
    
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.read_pos = 0;
    }
    
    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    fn set_intensity(&mut self, intensity: f32) {
        self.set_feedback(intensity * 0.95);
    }
}

/// Simple reverb effect using a series of comb filters.
#[derive(Debug, Clone)]
pub struct Reverb {
    /// Comb filter delays
    delays: Vec<usize>,
    
    /// Comb filter buffers
    buffers: Vec<Vec<f32>>,
    
    /// Current write positions
    write_pos: Vec<usize>,
    
    /// Decay factor
    decay: f32,
    
    /// Wet/dry mix
    mix: f32,
    
    /// Sample rate
    sample_rate: f32,
}

impl Reverb {
    /// Creates a new reverb effect.
    pub fn new(sample_rate: f32) -> Self {
        let comb_delays = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        
        let buffers: Vec<Vec<f32>> = comb_delays
            .iter()
            .map(|&d| vec![0.0; d])
            .collect();
        
        let mut write_pos = vec![0; comb_delays.len()];
        for (i, &d) in comb_delays.iter().enumerate() {
            write_pos[i] = d - 1;
        }
        
        Self {
            delays: comb_delays.to_vec(),
            buffers,
            write_pos,
            decay: 0.7,
            mix: 0.2,
            sample_rate,
        }
    }
    
    /// Sets the reverb decay time.
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.1, 0.95);
    }
}

impl Effect for Reverb {
    fn process(&mut self, input: f32) -> f32 {
        let mut output = 0.0;
        
        // Process through each comb filter
        for (i, delay) in self.delays.iter().enumerate() {
            let buffer = &mut self.buffers[i];
            let write_pos = self.write_pos[i];
            
            let delayed = buffer[write_pos % delay];
            buffer[write_pos % delay] = input + delayed * self.decay;
            self.write_pos[i] = (write_pos + 1) % delay;
            
            output += delayed;
        }
        
        // Average and mix
        output /= self.delays.len() as f32;
        input * (1.0 - self.mix) + output * self.mix
    }
    
    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }
    
    fn reset(&mut self) {
        for buffer in &mut self.buffers {
            buffer.fill(0.0);
        }
        self.write_pos = self.delays.iter().map(|d| d - 1).collect();
    }
    
    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    fn set_intensity(&mut self, intensity: f32) {
        self.set_decay(intensity);
    }
}

/// Distortion effect using waveshaping.
#[derive(Debug, Clone)]
pub struct Distortion {
    /// Distortion amount
    amount: f32,
    
    /// Wet/dry mix
    mix: f32,
}

impl Distortion {
    /// Creates a new distortion effect.
    pub fn new() -> Self {
        Self {
            amount: 0.5,
            mix: 0.5,
        }
    }
    
    /// Applies waveshaping curve to input sample.
    fn apply_curve(&self, sample: f32) -> f32 {
        let x = sample.clamp(-1.0, 1.0);
        let k = self.amount * 20.0; // Gain factor
        
        // Soft clipping curve
        (PI * k * x).sin() / (PI + k * x.abs())
    }
}

impl Effect for Distortion {
    fn process(&mut self, input: f32) -> f32 {
        let distorted = self.apply_curve(input);
        input * (1.0 - self.mix) + distorted * self.mix
    }
    
    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }
    
    fn reset(&mut self) {
        // No state to reset for stateless waveshaping
    }
    
    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    fn set_intensity(&mut self, intensity: f32) {
        self.amount = intensity.clamp(0.0, 1.0);
    }
}

/// Wrapper type for effect management.
#[derive(Debug, Clone)]
pub struct EffectProcessor {
    /// Current effect type
    effect_type: EffectType,
    
    /// Delay effect instance
    delay: Delay,
    
    /// Reverb effect instance
    reverb: Reverb,
    
    /// Distortion effect instance
    distortion: Distortion,
}

impl EffectProcessor {
    /// Creates a new effect processor.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            effect_type: EffectType::Delay,
            delay: Delay::new(sample_rate),
            reverb: Reverb::new(sample_rate),
            distortion: Distortion::new(),
        }
    }
    
    /// Sets the active effect type.
    pub fn set_effect_type(&mut self, effect_type: EffectType) {
        self.effect_type = effect_type;
    }
    
    /// Gets the current effect type.
    pub fn effect_type(&self) -> EffectType {
        self.effect_type
    }
}

impl Effect for EffectProcessor {
    fn process(&mut self, input: f32) -> f32 {
        match self.effect_type {
            EffectType::Delay => self.delay.process(input),
            EffectType::Reverb => self.reverb.process(input),
            EffectType::Distortion => self.distortion.process(input),
            _ => input, // Placeholder for unimplemented effects
        }
    }
    
    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }
    
    fn reset(&mut self) {
        self.delay.reset();
        self.reverb.reset();
        self.distortion.reset();
    }
    
    fn set_mix(&mut self, mix: f32) {
        match self.effect_type {
            EffectType::Delay => self.delay.set_mix(mix),
            EffectType::Reverb => self.reverb.set_mix(mix),
            EffectType::Distortion => self.distortion.set_mix(mix),
            _ => {}
        }
    }
    
    fn set_intensity(&mut self, intensity: f32) {
        match self.effect_type {
            EffectType::Delay => self.delay.set_intensity(intensity),
            EffectType::Reverb => self.reverb.set_intensity(intensity),
            EffectType::Distortion => self.distortion.set_intensity(intensity),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_delay_default() {
        let delay = Delay::new(44100.0);
        assert_eq!(delay.delay_samples, (0.3 * 44100.0) as usize);
    }
    
    #[test]
    fn test_delay_process() {
        let mut delay = Delay::new(1000.0);
        delay.set_delay_time(100.0); // 100ms delay
        
        let output = delay.process(0.5);
        // First sample should have no delayed signal yet
        assert!((output - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_delay_set_mix() {
        let mut delay = Delay::new(44100.0);
        delay.set_mix(0.5);
        assert_eq!(delay.mix, 0.5);
    }
    
    #[test]
    fn test_reverb_default() {
        let reverb = Reverb::new(44100.0);
        assert_eq!(reverb.delays.len(), 8);
    }
    
    #[test]
    fn test_reverb_process() {
        let mut reverb = Reverb::new(1000.0);
        let output = reverb.process(0.5);
        // Should process without clipping
        assert!(output.abs() <= 1.0);
    }
    
    #[test]
    fn test_distortion_default() {
        let dist = Distortion::new();
        assert_eq!(dist.amount, 0.5);
    }
    
    #[test]
    fn test_distortion_process() {
        let mut dist = Distortion::new();
        let output = dist.process(0.5);
        // Should process without issues
        assert!(output.abs() <= 1.0);
    }
    
    #[test]
    fn test_effect_processor() {
        let mut fx = EffectProcessor::new(44100.0);
        
        // Test switching effects
        fx.set_effect_type(EffectType::Delay);
        assert_eq!(fx.effect_type(), EffectType::Delay);
        
        fx.set_effect_type(EffectType::Reverb);
        assert_eq!(fx.effect_type(), EffectType::Reverb);
    }
}
