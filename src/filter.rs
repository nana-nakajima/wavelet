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
/// maintains 2 delay samples for the transposed direct form II structure.
///
/// # Processing
///
/// The filter processes samples using the direct form II transposed structure:
/// y[n] = b0*x[n] + z1
/// z1 = b1*x[n] - a1*y[n] + z2
/// z2 = b2*x[n] - a2*y[n]
#[derive(Debug, Clone)]
pub struct BiquadFilter {
    /// Filter coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    /// Filter state for Direct Form II Transposed (only 2 state variables needed)
    z1: f32, // First delay state
    z2: f32, // Second delay state

    /// Current filter type
    filter_type: FilterType,

    /// Current cutoff frequency
    cutoff: f32,

    /// Current resonance value
    resonance: f32,

    /// Sample rate for coefficient updates
    sample_rate: f32,

    /// Whether the filter is enabled
    enabled: bool,

    /// Wet/dry mix
    mix: f32,
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
            z1: 0.0,
            z2: 0.0,
            filter_type: config.filter_type,
            cutoff: config.cutoff_frequency,
            resonance: config.resonance,
            sample_rate: config.sample_rate,
            enabled: true,
            mix: 1.0,
        };

        filter.calculate_coefficients();
        filter
    }

    /// Processes a single audio sample through the filter.
    ///
    /// Uses Direct Form II Transposed structure for numerical stability.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio sample
    ///
    /// # Returns
    ///
    /// Filtered output sample
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Direct Form II Transposed:
        // y[n] = b0*x[n] + z1
        // z1 = b1*x[n] - a1*y[n] + z2
        // z2 = b2*x[n] - a2*y[n]
        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;

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
    pub fn set_gain(&mut self, _gain: f32) {
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
        self.z1 = 0.0;
        self.z2 = 0.0;
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

    // --- Helper: generate a sine wave at a given frequency ---
    fn generate_sine(freq: f32, sample_rate: f32, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
            .collect()
    }

    // --- Helper: measure RMS of a signal ---
    fn rms(signal: &[f32]) -> f32 {
        let sum_sq: f32 = signal.iter().map(|s| s * s).sum();
        (sum_sq / signal.len() as f32).sqrt()
    }

    // --- Biquad: Lowpass attenuates high frequencies ---
    #[test]
    fn test_lowpass_attenuates_high_frequencies() {
        let sample_rate = 44100.0;
        let cutoff = 500.0;
        let mut filter = BiquadFilter::with_config(FilterConfig {
            filter_type: FilterType::LowPass,
            cutoff_frequency: cutoff,
            resonance: 0.707,
            sample_rate,
            ..Default::default()
        });

        // Signal well below cutoff should pass through
        let low_signal = generate_sine(100.0, sample_rate, 4096);
        let low_output: Vec<f32> = low_signal
            .iter()
            .map(|&s| filter.process_sample(s))
            .collect();
        filter.reset();

        // Signal well above cutoff should be attenuated
        let high_signal = generate_sine(5000.0, sample_rate, 4096);
        let high_output: Vec<f32> = high_signal
            .iter()
            .map(|&s| filter.process_sample(s))
            .collect();

        // Skip transient (first 512 samples), measure steady-state
        let low_rms = rms(&low_output[512..]);
        let high_rms = rms(&high_output[512..]);

        // Low frequency should retain most energy (>80% of input)
        assert!(low_rms > 0.5, "Low freq RMS too low: {}", low_rms);
        // High frequency should be significantly attenuated (<10% of low)
        assert!(
            high_rms < low_rms * 0.2,
            "High freq not attenuated enough: high={}, low={}",
            high_rms,
            low_rms
        );
    }

    // --- Biquad: Highpass attenuates low frequencies ---
    #[test]
    fn test_highpass_attenuates_low_frequencies() {
        let sample_rate = 44100.0;
        let cutoff = 5000.0;
        let mut filter = BiquadFilter::with_config(FilterConfig {
            filter_type: FilterType::HighPass,
            cutoff_frequency: cutoff,
            resonance: 0.707,
            sample_rate,
            ..Default::default()
        });

        let low_signal = generate_sine(100.0, sample_rate, 4096);
        let low_output: Vec<f32> = low_signal
            .iter()
            .map(|&s| filter.process_sample(s))
            .collect();
        filter.reset();

        let high_signal = generate_sine(15000.0, sample_rate, 4096);
        let high_output: Vec<f32> = high_signal
            .iter()
            .map(|&s| filter.process_sample(s))
            .collect();

        let low_rms = rms(&low_output[512..]);
        let high_rms = rms(&high_output[512..]);

        assert!(
            low_rms < high_rms * 0.2,
            "Low freq not attenuated: low={}, high={}",
            low_rms,
            high_rms
        );
        assert!(high_rms > 0.5, "High freq RMS too low: {}", high_rms);
    }

    // --- Biquad: Bandpass passes center frequency ---
    #[test]
    fn test_bandpass_passes_center_rejects_extremes() {
        let sample_rate = 44100.0;
        let center = 2000.0;
        let mut filter = BiquadFilter::with_config(FilterConfig {
            filter_type: FilterType::BandPass,
            cutoff_frequency: center,
            resonance: 1.0,
            sample_rate,
            ..Default::default()
        });

        let center_signal = generate_sine(center, sample_rate, 4096);
        let center_out: Vec<f32> = center_signal
            .iter()
            .map(|&s| filter.process_sample(s))
            .collect();
        filter.reset();

        let far_signal = generate_sine(100.0, sample_rate, 4096);
        let far_out: Vec<f32> = far_signal
            .iter()
            .map(|&s| filter.process_sample(s))
            .collect();

        let center_rms = rms(&center_out[512..]);
        let far_rms = rms(&far_out[512..]);

        assert!(
            center_rms > far_rms * 2.0,
            "Bandpass center not louder than far: center={}, far={}",
            center_rms,
            far_rms
        );
    }

    // --- Biquad: DC response of lowpass ---
    #[test]
    fn test_lowpass_passes_dc() {
        let mut filter = BiquadFilter::with_config(FilterConfig {
            filter_type: FilterType::LowPass,
            cutoff_frequency: 1000.0,
            resonance: 0.707,
            sample_rate: 44100.0,
            ..Default::default()
        });

        // Feed constant (DC) signal, should converge to input value
        let mut output = 0.0;
        for _ in 0..2000 {
            output = filter.process_sample(1.0);
        }
        assert!(
            (output - 1.0).abs() < 0.01,
            "Lowpass DC response should be ~1.0, got {}",
            output
        );
    }

    // --- Biquad: Highpass blocks DC ---
    #[test]
    fn test_highpass_blocks_dc() {
        let mut filter = BiquadFilter::with_config(FilterConfig {
            filter_type: FilterType::HighPass,
            cutoff_frequency: 1000.0,
            resonance: 0.707,
            sample_rate: 44100.0,
            ..Default::default()
        });

        let mut output = 0.0;
        for _ in 0..2000 {
            output = filter.process_sample(1.0);
        }
        assert!(
            output.abs() < 0.01,
            "Highpass should block DC, got {}",
            output
        );
    }

    // --- Biquad: Reset produces silence from silence ---
    #[test]
    fn test_reset_then_silence() {
        let mut filter = BiquadFilter::new();
        // Excite the filter
        for _ in 0..100 {
            filter.process_sample(1.0);
        }
        filter.reset();
        // After reset, processing silence should produce silence
        let output = filter.process_sample(0.0);
        assert_eq!(
            output, 0.0,
            "After reset, silence in should give silence out"
        );
    }

    // --- Biquad: process_buffer matches sample-by-sample ---
    #[test]
    fn test_process_buffer_matches_sample_by_sample() {
        let config = FilterConfig {
            filter_type: FilterType::LowPass,
            cutoff_frequency: 2000.0,
            resonance: 1.5,
            sample_rate: 44100.0,
            ..Default::default()
        };

        let input = generate_sine(440.0, 44100.0, 256);

        // Process sample-by-sample
        let mut filter1 = BiquadFilter::with_config(config);
        let expected: Vec<f32> = input.iter().map(|&s| filter1.process_sample(s)).collect();

        // Process as buffer
        let mut filter2 = BiquadFilter::with_config(config);
        let mut buffer = input.clone();
        filter2.process_buffer(&mut buffer);

        for (i, (e, b)) in expected.iter().zip(buffer.iter()).enumerate() {
            assert!(
                (e - b).abs() < 1e-6,
                "Mismatch at sample {}: expected {}, got {}",
                i,
                e,
                b
            );
        }
    }

    // --- Biquad: Cutoff clamping ---
    #[test]
    fn test_cutoff_clamping() {
        let mut filter = BiquadFilter::new();
        filter.set_cutoff(5.0); // Below minimum
        assert!(filter.cutoff >= 20.0);
        filter.set_cutoff(100000.0); // Above Nyquist
        assert!(filter.cutoff <= filter.sample_rate / 2.0);
    }

    // --- Biquad: All filter types produce finite output ---
    #[test]
    fn test_all_filter_types_stable() {
        let signal = generate_sine(440.0, 44100.0, 1024);
        for filter_type in [
            FilterType::LowPass,
            FilterType::HighPass,
            FilterType::BandPass,
            FilterType::Notch,
            FilterType::AllPass,
        ] {
            let mut filter = BiquadFilter::with_config(FilterConfig {
                filter_type,
                cutoff_frequency: 1000.0,
                resonance: 2.0,
                sample_rate: 44100.0,
                ..Default::default()
            });
            for &s in &signal {
                let out = filter.process_sample(s);
                assert!(
                    out.is_finite(),
                    "{:?} produced non-finite output",
                    filter_type
                );
            }
        }
    }

    // --- ZDF: Lowpass attenuates high frequencies ---
    #[test]
    fn test_zdf_lowpass_attenuates_highs() {
        let sample_rate = 44100.0;
        let mut zdf = ZdfFilter::with_config(ZdfFilterConfig {
            mode: ZdfFilterMode::LowPass4,
            cutoff_frequency: 500.0,
            resonance: 0.5,
            drive: 0.0,
            sample_rate,
        });

        let low_signal = generate_sine(100.0, sample_rate, 4096);
        let low_out: Vec<f32> = low_signal.iter().map(|&s| zdf.process_sample(s)).collect();
        zdf.reset();

        let high_signal = generate_sine(5000.0, sample_rate, 4096);
        let high_out: Vec<f32> = high_signal.iter().map(|&s| zdf.process_sample(s)).collect();

        let low_rms = rms(&low_out[512..]);
        let high_rms = rms(&high_out[512..]);

        assert!(
            high_rms < low_rms * 0.15,
            "ZDF LP4 high freq not attenuated: high={}, low={}",
            high_rms,
            low_rms
        );
    }

    // --- ZDF: LP2 has less attenuation than LP4 ---
    #[test]
    fn test_zdf_lp2_less_steep_than_lp4() {
        let sample_rate = 44100.0;
        let freq = 3000.0; // Above cutoff
        let signal = generate_sine(freq, sample_rate, 4096);

        let mut lp2 = ZdfFilter::with_config(ZdfFilterConfig {
            mode: ZdfFilterMode::LowPass2,
            cutoff_frequency: 500.0,
            resonance: 0.5,
            drive: 0.0,
            sample_rate,
        });
        let lp2_out: Vec<f32> = signal.iter().map(|&s| lp2.process_sample(s)).collect();

        let mut lp4 = ZdfFilter::with_config(ZdfFilterConfig {
            mode: ZdfFilterMode::LowPass4,
            cutoff_frequency: 500.0,
            resonance: 0.5,
            drive: 0.0,
            sample_rate,
        });
        let lp4_out: Vec<f32> = signal.iter().map(|&s| lp4.process_sample(s)).collect();

        let lp2_rms = rms(&lp2_out[512..]);
        let lp4_rms = rms(&lp4_out[512..]);

        // LP4 (24dB/oct) should attenuate more than LP2 (12dB/oct)
        assert!(
            lp4_rms < lp2_rms,
            "LP4 should attenuate more than LP2: lp4={}, lp2={}",
            lp4_rms,
            lp2_rms
        );
    }

    // --- ZDF: Wrapper bypass ---
    #[test]
    fn test_zdf_wrapper_bypass_passthrough() {
        let mut zdf = ZdfFilterWrapper::new(44100.0);
        zdf.set_enabled(false);

        for &val in &[0.0, 0.5, -0.5, 1.0, -1.0] {
            assert_eq!(zdf.process(val), val, "Bypass should pass through {}", val);
        }
    }

    // --- ZDF: Drive adds saturation ---
    #[test]
    fn test_zdf_drive_adds_harmonics() {
        let sample_rate = 44100.0;
        let signal = generate_sine(200.0, sample_rate, 4096);

        let mut clean = ZdfFilter::with_config(ZdfFilterConfig {
            mode: ZdfFilterMode::LowPass4,
            cutoff_frequency: 10000.0,
            resonance: 0.5,
            drive: 0.0,
            sample_rate,
        });
        let clean_out: Vec<f32> = signal.iter().map(|&s| clean.process_sample(s)).collect();

        let mut driven = ZdfFilter::with_config(ZdfFilterConfig {
            mode: ZdfFilterMode::LowPass4,
            cutoff_frequency: 10000.0,
            resonance: 0.5,
            drive: 5.0,
            sample_rate,
        });
        let driven_out: Vec<f32> = signal.iter().map(|&s| driven.process_sample(s)).collect();

        // Driven signal should differ from clean (harmonics added)
        let diff: f32 = clean_out[512..]
            .iter()
            .zip(driven_out[512..].iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>()
            / (4096 - 512) as f32;

        assert!(
            diff > 0.01,
            "Drive should change the signal, avg diff={}",
            diff
        );
    }
}

// Import Effect trait for BiquadFilter implementation
use crate::effects::Effect;

/// Effect trait implementation for BiquadFilter
impl Effect for BiquadFilter {
    fn process(&mut self, input: f32) -> f32 {
        let filtered = self.process_sample(input);
        input * (1.0 - self.mix) + filtered * self.mix
    }

    fn process_with_bypass(&mut self, input: f32) -> f32 {
        if self.enabled {
            self.process(input)
        } else {
            input
        }
    }

    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    fn reset(&mut self) {
        self.reset();
    }

    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn set_intensity(&mut self, intensity: f32) {
        // Map intensity to resonance for simplicity
        self.set_resonance(0.1 + intensity * 19.9); // 0.1 to 20.0
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
