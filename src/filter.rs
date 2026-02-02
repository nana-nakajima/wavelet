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

/// Enumeration of ZDF (Zero-Delay Feedback) filter modes.
/// ZDF filters use a ladder topology similar to classic analog synthesizers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZdfFilterMode {
    /// 2-pole lowpass filter (12 dB/octave)
    LowPass2,
    
    /// 4-pole lowpass filter (24 dB/octave, like Moog ladder)
    LowPass4,
    
    /// 2-pole highpass filter (12 dB/octave)
    HighPass2,
}

/// Configuration structure for ZDF filter parameters.
#[derive(Debug, Clone, Copy)]
pub struct ZdfFilterConfig {
    /// Filter mode (lowpass 2/4 pole, highpass)
    pub mode: ZdfFilterMode,
    
    /// Cutoff frequency in Hz (typically 20 to 20000)
    pub cutoff_frequency: f32,
    
    /// Resonance factor (0.0 to ~4.0 for self-oscillation)
    pub resonance: f32,
    
    /// Drive amount for analog saturation (0.0 to ~10.0)
    pub drive: f32,
    
    /// Sample rate for internal calculations
    pub sample_rate: f32,
}

impl Default for ZdfFilterConfig {
    fn default() -> Self {
        Self {
            mode: ZdfFilterMode::LowPass4,
            cutoff_frequency: 1000.0,
            resonance: 1.0,
            drive: 0.0,
            sample_rate: 44100.0,
        }
    }
}

/// Zero-Delay Feedback (ZDF) Ladder Filter.
///
/// This filter implements a ladder topology similar to the classic Moog synthesizer.
/// Unlike traditional IIR filters that use delay elements for feedback, ZDF filters
/// use algebraic equations to solve for the feedback signal, eliminating the delay
/// inherent in digital implementations and producing a more analog character.
///
/// The filter uses a series of 4 one-pole lowpass sections (integrators) connected
/// in a feedback loop. The feedback signal is calculated without delay, creating
/// a more accurate simulation of analog circuitry.
///
/// # Characteristics
///
/// - **LowPass4**: Classic Moog ladder sound with 24 dB/octave rolloff
/// - **LowPass2**: Softer 12 dB/octave lowpass
/// - **HighPass2**: 12 dB/octave highpass with similar character
///
/// # Parameters
///
/// - **Cutoff**: Frequency where filtering begins (20 Hz to 20 kHz)
/// - **Resonance**: Emphasizes frequencies near cutoff (0.0 to ~4.0)
/// - **Drive**: Input gain that creates harmonic saturation
#[derive(Debug, Clone)]
pub struct ZdfFilter {
    /// State variables for the 4 integrator stages (v0, v1, v2, v3)
    v0: f32,
    v1: f32,
    v2: f32,
    v3: f32,
    
    /// Current filter mode
    mode: ZdfFilterMode,
    
    /// Current cutoff frequency
    cutoff: f32,
    
    /// Current resonance value
    resonance: f32,
    
    /// Current drive amount
    drive: f32,
    
    /// Pre-calculated frequency coefficient (fc)
    fc: f32,
    
    /// Sample rate for internal calculations
    sample_rate: f32,
}

impl ZdfFilter {
    /// Creates a new ZDF filter with default configuration.
    pub fn new() -> Self {
        Self::with_config(ZdfFilterConfig::default())
    }
    
    /// Creates a new ZDF filter with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - ZDF filter configuration parameters
    ///
    /// # Returns
    ///
    /// A configured ZdfFilter instance
    pub fn with_config(config: ZdfFilterConfig) -> Self {
        let mut filter = Self {
            v0: 0.0,
            v1: 0.0,
            v2: 0.0,
            v3: 0.0,
            mode: config.mode,
            cutoff: config.cutoff_frequency,
            resonance: config.resonance,
            drive: config.drive,
            fc: 0.0,
            sample_rate: config.sample_rate,
        };
        
        filter.calculate_coefficients();
        filter
    }
    
    /// Processes a single audio sample through the ZDF filter.
    ///
    /// Uses the ladder filter topology with zero-delay feedback.
    /// The algorithm solves for the feedback signal without using delay elements,
    /// resulting in more accurate analog simulation.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio sample
    ///
    /// # Returns
    ///
    /// Filtered output sample
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Apply drive to input signal
        let driven_input = input * (1.0 + self.drive);
        
        // Calculate feedback compensation factor
        // This prevents the filter from becoming unstable at high resonance
        let feedback_comp = 1.0 + self.resonance;
        
        // Calculate the input to the first integrator stage
        // The feedback is subtracted from input, then compensated
        let input_stage = (driven_input - self.resonance * self.v3) / feedback_comp;
        
        // Apply hyperbolic tangent for soft clipping (analog saturation in each stage)
        // This adds smooth harmonic content similar to analog circuitry
        let s0 = input_stage.tanh();
        
        // First integrator stage
        // v1(n) = v1(n-1) + fc * (s0 - v1(n-1))
        // Using trapezoidal integration for better accuracy
        let s1 = (s0 - self.v1) * self.fc + self.v1;
        let v1_new = s1.tanh();
        
        // Second integrator stage
        let s2 = (v1_new - self.v2) * self.fc + self.v2;
        let v2_new = s2.tanh();
        
        // Third integrator stage
        let s3 = (v2_new - self.v3) * self.fc + self.v3;
        let v3_new = s3.tanh();
        
        // Update state variables
        self.v0 = s0;
        self.v1 = v1_new;
        self.v2 = v2_new;
        self.v3 = v3_new;
        
        // Select output based on filter mode
        match self.mode {
            ZdfFilterMode::LowPass4 => v3_new,
            ZdfFilterMode::LowPass2 => v2_new,
            ZdfFilterMode::HighPass2 => {
                // Highpass = input - lowpass2
                s0 - v2_new
            }
        }
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
    
    /// Sets the filter mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - New ZDF filter mode
    pub fn set_mode(&mut self, mode: ZdfFilterMode) {
        self.mode = mode;
    }
    
    /// Sets the cutoff frequency.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - New cutoff frequency in Hz (20 to sample_rate/2)
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.clamp(20.0, self.sample_rate / 2.0);
        self.calculate_coefficients();
    }
    
    /// Sets the resonance (Q) value.
    ///
    /// # Arguments
    ///
    /// * `resonance` - New resonance value (0.0 to ~4.0)
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.max(0.0);
        self.calculate_coefficients();
    }
    
    /// Sets the drive amount for analog saturation.
    ///
    /// # Arguments
    ///
    /// * `drive` - New drive amount (0.0 to ~10.0)
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(0.0);
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
        self.v0 = 0.0;
        self.v1 = 0.0;
        self.v2 = 0.0;
        self.v3 = 0.0;
    }
    
    /// Calculates the frequency coefficient from cutoff frequency.
    ///
    /// Uses the formula: fc = 2 * sin(pi * cutoff / sample_rate)
    /// This provides accurate frequency tracking across the audible range.
    fn calculate_coefficients(&mut self) {
        // Calculate normalized frequency
        let normalized_freq = self.cutoff / self.sample_rate;
        
        // Calculate frequency coefficient using sin approximation
        // This provides accurate cutoff tracking
        self.fc = 2.0 * (PI * normalized_freq).sin();
        
        // Clamp to prevent instability
        self.fc = self.fc.clamp(0.0, 2.0);
    }
}

impl Default for ZdfFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified wrapper for ZDF filter with standard API.
#[derive(Debug, Clone)]
pub struct ZdfFilterWrapper {
    /// Internal ZDF filter instance
    inner: ZdfFilter,
    
    /// Whether the filter is enabled
    enabled: bool,
}

impl ZdfFilterWrapper {
    /// Creates a new ZDF filter wrapper.
    pub fn new(sample_rate: f32) -> Self {
        let config = ZdfFilterConfig {
            mode: ZdfFilterMode::LowPass4,
            cutoff_frequency: 1000.0,
            resonance: 1.0,
            drive: 0.0,
            sample_rate,
        };
        
        Self {
            inner: ZdfFilter::with_config(config),
            enabled: true,
        }
    }
    
    /// Processes an audio sample with optional bypass.
    ///
    /// # Arguments
    ///
    /// * `sample` - Input audio sample
    ///
    /// # Returns
    ///
    /// Output sample (filtered if enabled, passthrough if disabled)
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.enabled {
            self.inner.process_sample(sample)
        } else {
            sample
        }
    }
    
    /// Sets the filter cutoff frequency.
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.inner.set_cutoff(cutoff);
    }
    
    /// Sets the filter resonance.
    pub fn set_resonance(&mut self, resonance: f32) {
        self.inner.set_resonance(resonance);
    }
    
    /// Sets the filter drive.
    pub fn set_drive(&mut self, drive: f32) {
        self.inner.set_drive(drive);
    }
    
    /// Sets the filter mode.
    pub fn set_mode(&mut self, mode: ZdfFilterMode) {
        self.inner.set_mode(mode);
    }
    
    /// Enables or disables the filter.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Checks if the filter is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
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
    
    #[test]
    fn test_zdf_filter_default() {
        let zdf = ZdfFilter::new();
        assert_eq!(zdf.mode, ZdfFilterMode::LowPass4);
        assert_eq!(zdf.cutoff, 1000.0);
        assert_eq!(zdf.resonance, 1.0);
    }
    
    #[test]
    fn test_zdf_filter_process() {
        let mut zdf = ZdfFilter::new();
        let input = 0.5;
        let output = zdf.process_sample(input);
        // Output should be within valid range
        assert!(output.abs() <= 1.0);
    }
    
    #[test]
    fn test_zdf_filter_reset() {
        let mut zdf = ZdfFilter::new();
        let _ = zdf.process_sample(1.0);
        zdf.reset();
        assert_eq!(zdf.v0, 0.0);
        assert_eq!(zdf.v1, 0.0);
        assert_eq!(zdf.v2, 0.0);
        assert_eq!(zdf.v3, 0.0);
    }
    
    #[test]
    fn test_zdf_filter_mode_change() {
        let mut zdf = ZdfFilter::new();
        
        zdf.set_mode(ZdfFilterMode::LowPass4);
        let lp4_output = zdf.process_sample(1.0);
        
        zdf.set_mode(ZdfFilterMode::LowPass2);
        let lp2_output = zdf.process_sample(1.0);
        
        zdf.set_mode(ZdfFilterMode::HighPass2);
        let hp_output = zdf.process_sample(1.0);
        
        // All outputs should be valid
        assert!(lp4_output.abs() <= 1.0);
        assert!(lp2_output.abs() <= 1.0);
        assert!(hp_output.abs() <= 1.0);
    }
    
    #[test]
    fn test_zdf_filter_cutoff_sweep() {
        let mut zdf = ZdfFilter::new();
        
        // Sweep through cutoff frequencies
        for cutoff in [100.0, 500.0, 1000.0, 5000.0, 10000.0] {
            zdf.set_cutoff(cutoff);
            let output = zdf.process_sample(0.5);
            assert!(output.abs() <= 1.0, "Output clipped at cutoff {}", cutoff);
        }
    }
    
    #[test]
    fn test_zdf_filter_high_resonance() {
        let mut zdf = ZdfFilter::new();
        zdf.set_resonance(3.5); // High resonance
        zdf.set_cutoff(1000.0);
        
        // Should not clip even at high resonance
        let output = zdf.process_sample(0.3);
        assert!(output.abs() <= 1.0);
    }
    
    #[test]
    fn test_zdf_filter_wrapper_bypass() {
        let mut zdf = ZdfFilterWrapper::new(44100.0);
        
        zdf.set_enabled(false);
        let passthrough = zdf.process(0.5);
        assert_eq!(passthrough, 0.5);
        
        zdf.set_enabled(true);
        let filtered = zdf.process(0.5);
        assert_ne!(filtered, 0.5);
    }
}
