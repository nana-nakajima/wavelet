//! Filter benchmarks for WAVELET audio engine
//!
//! Measures filter performance for various filter types and configurations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavelet::filter::{Filter, FilterType, BiquadFilter, ZdfFilter, ZdfFilterMode, ZdfFilterConfig};

const SAMPLE_RATE: f32 = 48000.0;

fn generate_test_input(samples: usize) -> Vec<f32> {
    // Mix of frequencies to test filter response
    let mut input = Vec::with_capacity(samples);
    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let sample = (2.0 * PI * 440.0 * t).sin() * 0.3
            + (2.0 * PI * 880.0 * t).sin() * 0.2
            + (2.0 * PI * 1760.0 * t).sin() * 0.1;
        input.push(sample);
    }
    input
}

use std::f32::consts::PI;

fn bench_biquad_lowpass(c: &mut Criterion) {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::LowPass);
    filter.set_cutoff(1000.0);
    filter.set_resonance(1.0);
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_biquad_lowpass_1khz", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            filter.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_biquad_highpass(c: &mut Criterion) {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::HighPass);
    filter.set_cutoff(500.0);
    filter.set_resonance(1.0);
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_biquad_highpass_500hz", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            filter.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_biquad_bandpass(c: &mut Criterion) {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::BandPass);
    filter.set_cutoff(1000.0);
    filter.set_resonance(2.0);
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_biquad_bandpass", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            filter.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_biquad_single_sample(c: &mut Criterion) {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::LowPass);
    filter.set_cutoff(1000.0);
    filter.set_resonance(1.0);
    
    c.bench_function("filter_biquad_single_sample", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(filter.process_sample(0.5));
            }
        })
    });
}

fn bench_biquad_high_resonance(c: &mut Criterion) {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::LowPass);
    filter.set_cutoff(1000.0);
    filter.set_resonance(20.0); // High Q
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_biquad_high_resonance", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            filter.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_biquad_cutoff_sweep(c: &mut Criterion) {
    let mut filter = BiquadFilter::new();
    filter.set_filter_type(FilterType::LowPass);
    filter.set_cutoff(1000.0);
    filter.set_resonance(1.0);
    
    c.bench_function("filter_biquad_cutoff_sweep", |b| {
        b.iter(|| {
            for cutoff in [100.0, 500.0, 1000.0, 2000.0, 5000.0] {
                filter.set_cutoff(black_box(cutoff));
                for _ in 0..100 {
                    black_box(filter.process_sample(0.5));
                }
            }
        })
    });
}

fn bench_zdf_lowpass4(c: &mut Criterion) {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::LowPass4);
    zdf.set_cutoff(1000.0);
    zdf.set_resonance(1.0);
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_zdf_lowpass4", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            zdf.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_zdf_lowpass2(c: &mut Criterion) {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::LowPass2);
    zdf.set_cutoff(1000.0);
    zdf.set_resonance(1.0);
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_zdf_lowpass2", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            zdf.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_zdf_highpass2(c: &mut Criterion) {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::HighPass2);
    zdf.set_cutoff(500.0);
    zdf.set_resonance(1.0);
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_zdf_highpass2", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            zdf.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_zdf_single_sample(c: &mut Criterion) {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::LowPass4);
    zdf.set_cutoff(1000.0);
    zdf.set_resonance(1.0);
    
    c.bench_function("filter_zdf_single_sample", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(zdf.process_sample(0.5));
            }
        })
    });
}

fn bench_zdf_with_drive(c: &mut Criterion) {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::LowPass4);
    zdf.set_cutoff(1000.0);
    zdf.set_resonance(2.0);
    zdf.set_drive(5.0);
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_zdf_with_drive", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            zdf.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_zdf_high_resonance(c: &mut Criterion) {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::LowPass4);
    zdf.set_cutoff(1000.0);
    zdf.set_resonance(3.5); // High resonance
    
    let input = generate_test_input(1000);
    let mut output = input.clone();
    
    c.bench_function("filter_zdf_high_resonance", |b| {
        b.iter(|| {
            output.copy_from_slice(black_box(&input));
            zdf.process_buffer(&mut output);
            black_box(&output);
        })
    });
}

fn bench_zdf_reset(c: &mut Criterion) {
    let mut zdf = ZdfFilter::new();
    zdf.set_mode(ZdfFilterMode::LowPass4);
    zdf.set_cutoff(1000.0);
    
    c.bench_function("filter_zdf_reset", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(zdf.process_sample(0.5));
            }
            zdf.reset();
        })
    });
}

fn bench_filter_wrapper(c: &mut Criterion) {
    let mut filter = Filter::new(FilterType::LowPass, 1000.0, 1.0, SAMPLE_RATE);
    
    c.bench_function("filter_wrapper", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(filter.process(0.5));
            }
        })
    });
}

criterion_group!(
    filter_benches,
    bench_biquad_lowpass,
    bench_biquad_highpass,
    bench_biquad_bandpass,
    bench_biquad_single_sample,
    bench_biquad_high_resonance,
    bench_biquad_cutoff_sweep,
    bench_zdf_lowpass4,
    bench_zdf_lowpass2,
    bench_zdf_highpass2,
    bench_zdf_single_sample,
    bench_zdf_with_drive,
    bench_zdf_high_resonance,
    bench_zdf_reset,
    bench_filter_wrapper,
);

criterion_main!(filter_benches);
