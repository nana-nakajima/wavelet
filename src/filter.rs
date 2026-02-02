//! Filter Module
//!
//! This module provides various filter implementations for shaping the frequency
//! content of audio signals. Filters are essential for creating the tonal character
//! of a synthesizer sound.
//!
//! # Filter Types
//!
//! The filter module supports several classic filter types:
//! - **Low-pass**: Passes low frequencies, attenuates highs
//! - **High-pass**: Passes high frequencies, attenuates lows
//! - **Band-pass**: Passes a band of frequencies, attenuates others
//! - **Notch (Band-reject)**: Attenuates a band of frequencies
//! - **All-pass**: Passes all frequencies but changes phase relationship
//!
//! # Filter Characteristics
//!
//! Key filter parameters include:
//! - **Cutoff Frequency**: The frequency at which attenuation begins
//! - **Resonance (Q)**: Emphasis of frequencies near the cutoff
//! - **Filter Slope**: How sharply frequencies are attenuated (dB/octave)
//!
//! # Biquad Filters
//!
//! This module uses biquad filter implementations for accurate and efficient
//! filtering. A biquad filter is a second-order IIR (Infinite Impulse Response)
//! filter that can be configured as any of the standard filter types.

use std::f32::consts::PI;

/// Enumeration of supported filter types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    /// Low-pass filter - passes frequencies below cutoff
    LowPass,
    
    /// High-pass filter - passes frequencies above cutoff
    HighPass,
    
    /// Band-pass filter - passes frequencies within a band
    BandPass,
    
    /// Notch (band-reject) filter - attenuates frequencies within a band
    Notch,
    
    /// All-pass filter - passes all frequencies with phase shift
    AllPass,
}

/// Configuration structure for filter parameters.
#[derive(Debug, Clone, Copy)]
pub struct FilterConfig {
    /// The type of filter to apply
    pub filter_type: FilterType,
    
    /// Cutoff frequency in Hz (typically 20 to 20000)
    pub cutoff_frequency: f32,
    
    /// Resonance factor (Q value, typically 0.1 to 20)
    /// Higher values create more emphasis at cutoff frequency
    pub resonance: f32,
    
    /// Filter gain for shelf and peaking filters
    pub gain: f32,
    
    /// Sample rate for coefficient calculations
    pub sample_rate: f32,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            filter_type: FilterType::LowPass,
            cutoff_frequency: 1000.0,
            resonance: 1.0,
            gain: 0.0,
            sample_rate: 44100.0,
        }
    }
}

/// Biquad filter state structure for IIR filter implementation.
///
/// A biquad filter uses 5 coefficients (a0, a1, a2, b0, b1, b2) and
/// maintains 2 delay samples (z-1) for the feedback path.
///
/// # Processing
///
/// The filter processes samples using the direct form II transposed structure:
/// y[n] = (b0*x[n] + b1*x[n-1] + b2*x[n-2] + a1*y[n-1] + a2*y[n-2]) / a0
#[derive(Debug, Clone)]
pub struct BiquadFilter {
    /// Filter coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    
    /// Filter state (previous inputs and outputs)
    x1: f32, // x[n-1]
    x2: f32, // x[n-2]
    y1: f32, // y[n-1]
    y2: f32, // y[n-2]
    
    /// Current filter type
    filter_type: FilterType,
    
    /// Current cutoff frequency
    cutoff: f32,
    
    /// Current resonance value
    resonance: f32,
    
    /// Sample rate for coefficient updates
    sample_rate: f32,
}

impl BiquadFilter {
    /// Creates a new biquad filter with default configuration.
    pub fn new() -> Self {
        Self::with_config(FilterConfig::default())
    }
    
    /// Creates a new biquad filter with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Filter configuration parameters
    ///
    /// # Returns
    ///
    /// A configured BiquadFilter instance
    pub fn with_config(config: FilterConfig) -> Self {
        let mut filter = Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            filter_type: config.filter_type,
            cutoff: config.cutoff_frequency,
            resonance: config.resonance,
            sample_rate: config.sample_rate,
        };
        
        filter.calculate_coefficients();
        filter
    }
    
    /// Processes a single audio sample through the filter.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio sample
    ///
    /// # Returns
    ///
    /// Filtered output sample
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Direct form II transposed structure
        let output = self.b0 * input + self.x1 + self.y1 * self.a1 + self.y2 * self.a2;
        
        // Update state
        self.x1 = self.b1 * input - output * self.a1 + self.x2;
        self.x2 = self.b2 * input - output * self.a2;
        self.y1 = output;
        self.y2 = self.y1;
        
        output
    }
    
    /// Processes a buffer of audio samples.
    ///
    /// # Arguments
    ///
    /// * `samples` - Mutable slice of audio samples to process
    pub fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
    
    /// Sets the filter type.
    ///
    /// # Arguments
    ///
    /// * `filter_type` - New filter type to apply
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
        self.calculate_coefficients();
    }
    
    /// Sets the cutoff frequency.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - New cutoff frequency in Hz
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.clamp(20.0, self.sample_rate / 2.0);
        self.calculate_coefficients();
    }
    
    /// Sets the resonance (Q) value.
    ///
    /// # Arguments
    ///
    /// * `resonance` - New Q value (typically 0.1 to 20)
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.max(0.001);
        self.calculate_coefficients();
    }
    
    /// Sets the filter gain (for shelf/peaking filters).
    ///
    /// # Arguments
    ///
    /// * `gain` - Gain in dB
    pub fn set_gain(&mut self, gain: f32) {
        self.calculate_coefficients();
    }
    
    /// Sets the sample rate and recalculates coefficients.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - New sample rate in Hz
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.calculate_coefficients();
    }
    
    /// Resets the filter state to zero.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
    
    /// Calculates biquad filter coefficients based on current parameters.
    ///
    /// Uses the standard bilinear transform method for coefficient calculation.
    /// This is called whenever filter parameters change.
    fn calculate_coefficients(&mut self) {
        let omega = 2.0 * PI * self.cutoff / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * self.resonance);
        
        // Normalize coefficients by a0
        let a0 = 1.0 + alpha;
        
        match self.filter_type {
            FilterType::LowPass => {
                self.b0 = (1.0 - cos_omega) / 2.0;
                self.b1 = 1.0 - cos_omega;
                self.b2 = (1.0 - cos_omega) / 2.0;
                self.a1 = -2.0 * cos_omega;
                self.a2 = 1.0 - alpha;
            }
            
            FilterType::HighPass => {
                self.b0 = (1.0 + cos_omega) / 2.0;
                self.b1 = -(1.0 + cos_omega);
                self.b2 = (1.0 + cos_omega) / 2.0;
                self.a1 = -2.0 * cos_omega;
                self.a2 = 1.0 - alpha;
            }
            
            FilterType::BandPass => {
                self.b0 = alpha;
                self.b1 = 0.0;
                self.b2 = -alpha;
                self.a1 = -2.0 * cos_omega;
                self.a2 = 1.0 - alpha;
            }
            
            FilterType::Notch => {
                self.b0 = 1.0;
                self.b1 = -2.0 * cos_omega;
                self.b2 = 1.0;
                self.a1 = -2.0 * cos_omega;
                self.a2 = 1.0 - alpha;
            }
            
            FilterType::AllPass => {
                self.b0 = 1.0 - alpha;
                self.b1 = -2.0 * cos_omega;
                self.b2 = 1.0 + alpha;
                self.a1 = -2.0 * cos_omega;
                self.a2 = 1.0 - alpha;
            }
        }
        
        // Normalize by a0
        self.b0 /= a0;
        self.b1 /= a0;
        self.b2 /= a0;
        self.a1 /= a0;
        self.a2 /= a0;
    }
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper type for filter operations with simplified API.
#[derive(Debug, Clone)]
pub struct Filter {
    /// Internal biquad filter instance
    inner: BiquadFilter,
}

impl Filter {
    /// Creates a new filter with the specified type.
    ///
    /// # Arguments
    ///
    /// * `filter_type` - Type of filter to create
    /// * `cutoff` - Initial cutoff frequency in Hz
    /// * `resonance` - Initial Q value
    /// * `sample_rate` - Audio system sample rate
    ///
    /// # Returns
    ///
    /// A new Filter instance
    pub fn new(filter_type: FilterType, cutoff: f32, resonance: f32, sample_rate: f32) -> Self {
        let config = FilterConfig {
            filter_type,
            cutoff_frequency: cutoff,
            resonance,
            gain: 0.0,
            sample_rate,
        };
        
        Self {
            inner: BiquadFilter::with_config(config),
        }
    }
    
    /// Processes an audio sample.
    pub fn process(&mut self, sample: f32) -> f32 {
        self.inner.process_sample(sample)
    }
    
    /// Sets the filter cutoff frequency.
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.inner.set_cutoff(cutoff);
    }
    
    /// Sets the filter resonance.
    pub fn set_resonance(&mut self, resonance: f32) {
        self.inner.set_resonance(resonance);
    }
    
    /// Sets the filter type.
    pub fn set_type(&mut self, filter_type: FilterType) {
        self.inner.set_filter_type(filter_type);
    }
    
    /// Resets filter state.
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_default() {
        let filter = Filter::new(FilterType::LowPass, 1000.0, 1.0, 44100.0);
        assert_eq!(filter.inner.filter_type, FilterType::LowPass);
        assert_eq!(filter.inner.cutoff, 1000.0);
    }
    
    #[test]
    fn test_filter_process() {
        let mut filter = Filter::new(FilterType::LowPass, 1000.0, 1.0, 44100.0);
        let input = 0.5;
        let output = filter.process(input);
        // Output should be within valid range
        assert!(output.abs() <= input.abs() + 0.01);
    }
    
    #[test]
    fn test_biquad_reset() {
        let mut filter = BiquadFilter::new();
        let _ = filter.process_sample(1.0);
        filter.reset();
        assert_eq!(filter.x1, 0.0);
        assert_eq!(filter.x2, 0.0);
        assert_eq!(filter.y1, 0.0);
        assert_eq!(filter.y2, 0.0);
    }
    
    #[test]
    fn test_filter_type_change() {
        let mut filter = BiquadFilter::new();
        let lp_output = filter.process_sample(1.0);
        
        filter.set_filter_type(FilterType::HighPass);
        let hp_output = filter.process_sample(1.0);
        
        // Outputs should be different for different filter types
        assert_ne!(lp_output, hp_output);
    }
}
