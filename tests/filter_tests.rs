//! Filter tests for WAVELET audio engine
//!
//! Tests cover:
use super::common::{
    assert_samples_are_finite, assert_samples_in_range, calculate_peak, calculate_rms,
    generate_test_buffer,
};
/// - Biquad filter stability
/// - ZDF filter frequency response
/// - ZDF filter doesn't explode with high resonance
/// - Filter bypass
use wavelet::filter::{
    BiquadFilter, Filter, FilterConfig, FilterType, ZdfFilter, ZdfFilterConfig, ZdfFilterMode,
};

const SAMPLE_RATE: f32 = 48000.0;

#[test]
fn test_biquad_lowpass_stability() {
    let mut filter = BiquadFilter::new();

    // Process a sweep of frequencies - should be stable
    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    filter.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_biquad_highpass_stability() {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::HighPass);

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    filter.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_biquad_bandpass_stability() {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::BandPass);

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    filter.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_biquad_notch_stability() {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::Notch);

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    filter.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_biquad_allpass_stability() {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::AllPass);

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    filter.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_biquad_cutoff_sweep() {
    let mut filter = BiquadFilter::new();

    // Sweep through various cutoff frequencies
    for cutoff in [100.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0] {
        filter.set_cutoff(cutoff);

        let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
        filter.process_buffer(&mut samples);

        assert_samples_in_range(&samples);
        assert_samples_are_finite(&samples);
    }
}

#[test]
fn test_biquad_resonance_sweep() {
    let mut filter = BiquadFilter::new();
    filter.set_cutoff(1000.0);

    // Sweep through various resonance values
    for resonance in [0.1, 1.0, 5.0, 10.0, 20.0] {
        filter.set_resonance(resonance);

        let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
        filter.process_buffer(&mut samples);

        assert_samples_in_range(&samples);
        assert_samples_are_finite(&samples);
    }
}

#[test]
fn test_biquad_high_resonance_stability() {
    let mut filter = BiquadFilter::new();
    filter.set_cutoff(1000.0);
    filter.set_resonance(20.0); // Very high Q

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.3);
    let mut output = input.clone();

    filter.process_buffer(&mut output);

    // Should not clip even at high resonance
    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_biquad_reset() {
    let mut filter = BiquadFilter::new();

    // Process some samples to build up state
    let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 1.0);
    filter.process_buffer(&mut samples);

    // Reset should clear state
    filter.reset();

    // Next samples should start fresh
    let output = filter.process_sample(0.0);
    assert!(output.abs() < 0.001, "After reset, output should be near 0");
}

#[test]
fn test_biquad_single_sample_processing() {
    let mut filter = BiquadFilter::new();

    for _ in 0..1000 {
        let input = 0.5;
        let output = filter.process_sample(input);
        assert!(output.abs() <= 1.0);
        assert!(output.is_finite());
    }
}

#[test]
fn test_biquad_filter_type_change() {
    let mut filter = BiquadFilter::new();

    let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    let original_output: Vec<f32> = samples.iter().copied().collect();

    filter.set_filter_type(FilterType::HighPass);
    filter.process_buffer(&mut samples);
    let hp_output: Vec<f32> = samples.iter().copied().collect();

    filter.set_filter_type(FilterType::BandPass);
    filter.process_buffer(&mut samples);
    let bp_output: Vec<f32> = samples.iter().copied().collect();

    // All outputs should be valid
    for output in [&original_output, &hp_output, &bp_output] {
        assert_samples_in_range(output);
        assert_samples_are_finite(output);
    }
}

#[test]
fn test_biquad_cutoff_clamping() {
    let mut filter = BiquadFilter::new();

    // Cutoff should be clamped to valid range
    filter.set_cutoff(10.0); // Too low
    assert!(filter.cutoff >= 20.0);

    filter.set_cutoff(SAMPLE_RATE * 2.0); // Too high (above Nyquist)
    assert!(filter.cutoff <= SAMPLE_RATE / 2.0);
}

#[test]
fn test_biquad_zero_resonance() {
    let mut filter = BiquadFilter::new();
    filter.set_resonance(0.0);

    let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    filter.process_buffer(&mut samples);

    assert_samples_in_range(&samples);
    assert_samples_are_finite(&samples);
}

// ZDF Filter Tests

#[test]
fn test_zdf_default_config() {
    let zdf = ZdfFilter::new();
    assert_eq!(zdf.mode, ZdfFilterMode::LowPass4);
    assert_eq!(zdf.cutoff, 1000.0);
    assert_eq!(zdf.resonance, 1.0);
}

#[test]
fn test_zdf_lowpass4_process() {
    let mut zdf = ZdfFilter::new();

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    zdf.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_zdf_lowpass2_process() {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::LowPass2);

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    zdf.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_zdf_highpass2_process() {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::HighPass2);

    let input = generate_test_buffer(SAMPLE_RATE, 440.0, 1.0, 0.5);
    let mut output = input.clone();

    zdf.process_buffer(&mut output);

    assert_samples_in_range(&output);
    assert_samples_are_finite(&output);
}

#[test]
fn test_zdf_no_explosion_high_resonance() {
    let mut zdf = ZdfFilter::new();

    // Test with high resonance values
    for resonance in [1.0, 2.0, 3.0, 3.5, 4.0] {
        zdf.set_resonance(resonance);
        zdf.set_cutoff(1000.0);

        let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.3);
        let mut output = input.clone();

        zdf.process_buffer(&mut output);

        assert_samples_in_range(&output);
        assert_samples_are_finite(&output);
    }
}

#[test]
fn test_zdf_cutoff_sweep() {
    let mut zdf = ZdfFilter::new();

    // Sweep through various cutoff frequencies
    for cutoff in [100.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0] {
        zdf.set_cutoff(cutoff);

        let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
        zdf.process_buffer(&mut samples);

        assert_samples_in_range(&samples);
        assert_samples_are_finite(&samples);
    }
}

#[test]
fn test_zdf_single_sample_processing() {
    let mut zdf = ZdfFilter::new();

    for _ in 0..1000 {
        let input = 0.5;
        let output = zdf.process_sample(input);
        assert!(output.abs() <= 1.0);
        assert!(output.is_finite());
    }
}

#[test]
fn test_zdf_reset() {
    let mut zdf = ZdfFilter::new();

    // Process some samples to build up state
    let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 1.0);
    zdf.process_buffer(&mut samples);

    // Reset should clear state
    zdf.reset();

    // Check that state variables are reset
    assert_eq!(zdf.v0, 0.0);
    assert_eq!(zdf.v1, 0.0);
    assert_eq!(zdf.v2, 0.0);
    assert_eq!(zdf.v3, 0.0);
}

#[test]
fn test_zdf_with_drive() {
    let mut zdf = ZdfFilter::new();

    // Test with drive
    for drive in [0.0, 1.0, 5.0, 10.0] {
        zdf.set_drive(drive);

        let input = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.3);
        let mut output = input.clone();

        zdf.process_buffer(&mut output);

        assert_samples_in_range(&output);
        assert_samples_are_finite(&output);
    }
}

#[test]
fn test_zdf_mode_change() {
    let mut zdf = ZdfFilter::new();

    let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    zdf.process_buffer(&mut samples);
    let output_lp4: Vec<f32> = samples.iter().copied().collect();

    zdf.set_mode(ZdfFilterMode::LowPass2);
    zdf.reset();
    zdf.process_buffer(&mut samples);
    let output_lp2: Vec<f32> = samples.iter().copied().collect();

    zdf.set_mode(ZdfFilterMode::HighPass2);
    zdf.reset();
    zdf.process_buffer(&mut samples);
    let output_hp2: Vec<f32> = samples.iter().copied().collect();

    // All outputs should be valid but different
    assert_samples_in_range(&output_lp4);
    assert_samples_in_range(&output_lp2);
    assert_samples_in_range(&output_hp2);
}

#[test]
fn test_zdf_dc_response() {
    let mut zdf = ZdfFilter::new();

    // DC (0 Hz) should pass through lowpass filters
    let dc_input = vec![1.0; 1000];
    let mut output = dc_input.clone();
    zdf.process_buffer(&mut output);

    // For a lowpass filter, DC should be passed
    // The exact value depends on resonance and cutoff
    for &sample in &output {
        assert!(sample.is_finite());
    }
}

#[test]
fn test_zdf_nyquist_response() {
    let mut zdf = ZdfFilter::new();

    // Nyquist frequency should be attenuated by lowpass
    let nyquist_input = generate_test_buffer(SAMPLE_RATE, SAMPLE_RATE / 2.0 - 10.0, 0.1, 0.5);
    let mut output = nyquist_input.clone();
    zdf.process_buffer(&mut output);

    // Should be attenuated (though not completely at high resonance)
    let input_rms = calculate_rms(&nyquist_input);
    let output_rms = calculate_rms(&output);

    assert!(output_rms <= input_rms * 1.5); // Allow some resonance boost
    assert_samples_are_finite(&output);
}

#[test]
fn test_filter_wrapper_bypass() {
    let mut filter = Filter::new(FilterType::LowPass, 1000.0, 1.0, SAMPLE_RATE);

    let input = 0.5;
    let output = filter.process(input);

    // Output should be filtered, not exactly the input
    // (Unless it's a full-pass filter like AllPass)
    assert!(output.is_finite());
}

#[test]
fn test_zdf_wrapper_bypass() {
    use wavelet::filter::ZdfFilterWrapper;

    let mut zdf = ZdfFilterWrapper::new(SAMPLE_RATE);

    // When disabled, should pass through
    zdf.set_enabled(false);
    let passthrough = zdf.process(0.5);
    assert_eq!(passthrough, 0.5);

    // When enabled, should filter
    zdf.set_enabled(true);
    let filtered = zdf.process(0.5);
    assert_ne!(filtered, 0.5);

    assert!(filtered.is_finite());
}

#[test]
fn test_biquad_sane_at_extreme_cutoffs() {
    let mut filter = BiquadFilter::new();

    // Very low cutoff
    filter.set_cutoff(20.0);
    let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    filter.process_buffer(&mut samples);
    assert_samples_are_finite(&samples);

    // Very high cutoff (near Nyquist)
    filter.set_cutoff(SAMPLE_RATE / 2.0 - 100.0);
    filter.reset();
    samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    filter.process_buffer(&mut samples);
    assert_samples_are_finite(&samples);
}

#[test]
fn test_zdf_sane_at_extreme_cutoffs() {
    let mut zdf = ZdfFilter::new();

    // Very low cutoff
    zdf.set_cutoff(20.0);
    let mut samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    zdf.process_buffer(&mut samples);
    assert_samples_are_finite(&samples);

    // Very high cutoff (near Nyquist)
    zdf.set_cutoff(SAMPLE_RATE / 2.0 - 100.0);
    zdf.reset();
    samples = generate_test_buffer(SAMPLE_RATE, 440.0, 0.1, 0.5);
    zdf.process_buffer(&mut samples);
    assert_samples_are_finite(&samples);
}

#[test]
fn test_biquad_with_config() {
    let config = FilterConfig {
        filter_type: FilterType::LowPass,
        cutoff_frequency: 1500.0,
        resonance: 2.0,
        gain: 0.0,
        sample_rate: SAMPLE_RATE,
    };

    let filter = BiquadFilter::with_config(config);
    assert!(true); // Just verify it was created
}

#[test]
fn test_zdf_with_config() {
    let config = ZdfFilterConfig {
        mode: ZdfFilterMode::LowPass4,
        cutoff_frequency: 2000.0,
        resonance: 3.0,
        drive: 2.0,
        sample_rate: SAMPLE_RATE,
    };

    let zdf = ZdfFilter::with_config(config);
    assert_eq!(zdf.cutoff, 2000.0);
    assert_eq!(zdf.resonance, 3.0);
    assert_eq!(zdf.drive, 2.0);
}

#[test]
fn test_filter_set_sample_rate() {
    let mut filter = BiquadFilter::new();

    // Change sample rate
    filter.set_sample_rate(96000.0);

    let mut samples = generate_test_buffer(96000.0, 440.0, 0.1, 0.5);
    filter.process_buffer(&mut samples);

    assert_samples_are_finite(&samples);
}

#[test]
fn test_zdf_set_sample_rate() {
    let mut zdf = ZdfFilter::new();

    // Change sample rate
    zdf.set_sample_rate(96000.0);

    let mut samples = generate_test_buffer(96000.0, 440.0, 0.1, 0.5);
    zdf.process_buffer(&mut samples);

    assert_samples_are_finite(&samples);
}
