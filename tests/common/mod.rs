// Test utilities and helper functions for WAVELET audio engine tests

use std::f32::consts::PI;

/// Generates a sine wave test signal.
///
/// # Arguments
/// * `sample` - Current sample index
/// * `frequency` - Frequency in Hz
/// * `sample_rate` - Sample rate in Hz
/// * `amplitude` - Signal amplitude
///
/// # Returns
/// Sample value in range [-amplitude, amplitude]
pub fn generate_sine_wave(sample: usize, frequency: f32, sample_rate: f32, amplitude: f32) -> f32 {
    let phase = (sample as f32 * frequency / sample_rate) % 1.0;
    (phase * 2.0 * PI).sin() * amplitude
}

/// Generates an impulse test signal.
///
/// # Arguments
/// * `sample` - Current sample index
/// * `impulse_position` - Position of the impulse (sample index)
///
/// # Returns
/// 1.0 at impulse position, 0.0 elsewhere
pub fn generate_impulse(sample: usize, impulse_position: usize) -> f32 {
    if sample == impulse_position {
        1.0
    } else {
        0.0
    }
}

/// Generates white noise test signal.
///
/// # Arguments
/// * `rng` - Random number generator reference
/// * `amplitude` - Signal amplitude
///
/// # Returns
/// Random value in range [-amplitude, amplitude]
pub fn generate_white_noise<R: rand::Rng>(rng: &mut R, amplitude: f32) -> f32 {
    (rng.gen::<f32>() * 2.0 - 1.0) * amplitude
}

/// Calculates the Root Mean Square (RMS) of a signal.
///
/// # Arguments
/// * `samples` - Slice of sample values
///
/// # Returns
/// RMS value
pub fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|&s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

/// Calculates the peak value of a signal.
///
/// # Arguments
/// * `samples` - Slice of sample values
///
/// # Returns
/// Maximum absolute value
pub fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&s| s.abs()).fold(0.0, f32::max)
}

/// Calculates the peak-to-peak value of a signal.
///
/// # Arguments
/// * `samples` - Slice of sample values
///
/// # Returns
/// Maximum value minus minimum value
pub fn calculate_peak_to_peak(samples: &[f32]) -> f32 {
    let max = samples.iter().fold(f32::MIN, |m, &s| s.max(m));
    let min = samples.iter().fold(f32::MAX, |m, &s| s.min(m));
    max - min
}

/// Checks if all samples are within the valid audio range [-1, 1].
pub fn assert_samples_in_range(samples: &[f32]) {
    for (i, &sample) in samples.iter().enumerate() {
        assert!(
            sample >= -1.0 && sample <= 1.0,
            "Sample {} out of range: {} (expected [-1, 1])",
            i,
            sample
        );
    }
}

/// Checks if all samples are finite (not NaN or infinity).
pub fn assert_samples_are_finite(samples: &[f32]) {
    for (i, &sample) in samples.iter().enumerate() {
        assert!(
            sample.is_finite(),
            "Sample {} is not finite: {}",
            i,
            sample
        );
    }
}

/// Asserts that two floating point values are approximately equal.
///
/// # Arguments
/// * `actual` - Actual value
/// * `expected` - Expected value
/// * `epsilon` - Maximum allowed difference
pub fn assert_approx_eq(actual: f32, expected: f32, epsilon: f32) {
    assert!(
        (actual - expected).abs() <= epsilon,
        "Values not approximately equal: actual = {}, expected = {}, epsilon = {}",
        actual,
        expected,
        epsilon
    );
}

/// Asserts that two slices of samples are approximately equal.
///
/// # Arguments
/// * `actual` - Actual sample slice
/// * `expected` - Expected sample slice
/// * `epsilon` - Maximum allowed difference per sample
pub fn assert_samples_approx_eq(actual: &[f32], expected: &[f32], epsilon: f32) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "Sample slice lengths don't match"
    );
    
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            (a - e).abs() <= epsilon,
            "Samples differ at index {}: actual = {}, expected = {}",
            i,
            a,
            e
        );
    }
}

/// Measures the dynamic range of a signal in dB.
///
/// # Arguments
/// * `samples` - Slice of sample values
///
/// # Returns
/// Dynamic range in dB (0 dB for full-scale signal)
pub fn measure_dynamic_range_db(samples: &[f32]) -> f32 {
    let peak = calculate_peak(samples);
    if peak == 0.0 {
        return 0.0;
    }
    20.0 * peak.log10()
}

/// Checks if the signal has a reasonable crest factor.
///
/// # Arguments
/// * `samples` - Slice of sample values
///
/// # Returns
/// Crest factor (peak / RMS)
pub fn calculate_crest_factor(samples: &[f32]) -> f32 {
    let rms = calculate_rms(samples);
    let peak = calculate_peak(samples);
    
    if rms == 0.0 {
        0.0
    } else {
        peak / rms
    }
}

/// Generates a test buffer filled with a sine wave.
///
/// # Arguments
/// * `sample_rate` - Sample rate in Hz
/// * `frequency` - Frequency in Hz
/// * `duration_s` - Duration in seconds
/// * `amplitude` - Signal amplitude
///
/// # Returns
/// Vector of samples
pub fn generate_test_buffer(
    sample_rate: f32,
    frequency: f32,
    duration_s: f32,
    amplitude: f32,
) -> Vec<f32> {
    let num_samples = (sample_rate * duration_s) as usize;
    (0..num_samples)
        .map(|i| generate_sine_wave(i, frequency, sample_rate, amplitude))
        .collect()
}

/// Approximate number of samples for a given time duration.
pub fn samples_for_duration(sample_rate: f32, duration_s: f32) -> usize {
    (sample_rate * duration_s) as usize
}

/// Approximate duration for a number of samples at given sample rate.
pub fn duration_for_samples(sample_rate: f32, num_samples: usize) -> f32 {
    num_samples as f32 / sample_rate
}

/// Normalizes a signal to have peak amplitude of 1.0.
pub fn normalize_signal(samples: &[f32]) -> Vec<f32> {
    let peak = calculate_peak(samples);
    if peak == 0.0 {
        return samples.to_vec();
    }
    samples.iter().map(|&s| s / peak).collect()
}

/// Calculates the mean of a signal.
pub fn calculate_mean(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().sum::<f32>() / samples.len() as f32
}

/// Calculates the variance of a signal.
pub fn calculate_variance(samples: &[f32]) -> f32 {
    let mean = calculate_mean(samples);
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|&s| (s - mean) * (s - mean)).sum();
    sum / samples.len() as f32
}

/// Calculates the standard deviation of a signal.
pub fn calculate_std_dev(samples: &[f32]) -> f32 {
    calculate_variance(samples).sqrt()
}
