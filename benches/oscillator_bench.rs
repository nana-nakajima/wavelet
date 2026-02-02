//! Oscillator benchmarks for WAVELET audio engine
//!
//! Measures oscillator performance for various waveforms and configurations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavelet::oscillator::{Oscillator, OscillatorConfig, OversampleFactor, Waveform};

const SAMPLE_RATE: f32 = 48000.0;

fn bench_sine_wave(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Sine,
        frequency: 440.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_sine_440hz", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_square_wave(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Square,
        frequency: 440.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_square_440hz", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_sawtooth_wave(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Sawtooth,
        frequency: 440.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_sawtooth_440hz", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_triangle_wave(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Triangle,
        frequency: 440.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_triangle_440hz", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_noise_wave(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Noise,
        frequency: 440.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_noise", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_high_frequency(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Sawtooth,
        frequency: 8000.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_sawtooth_8khz", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_oversampling_x2(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Sawtooth,
        frequency: 2000.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        oversample_factor: OversampleFactor::X2,
        ..Default::default()
    });

    c.bench_function("oscillator_oversample_x2", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_oversampling_x4(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Sawtooth,
        frequency: 2000.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        oversample_factor: OversampleFactor::X4,
        ..Default::default()
    });

    c.bench_function("oscillator_oversample_x4", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(osc.next_sample());
            }
        })
    });
}

fn bench_block_processing(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Sine,
        frequency: 440.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_block_1000_samples", |b| {
        b.iter(|| {
            black_box(osc.next_samples(1000));
        })
    });
}

fn bench_frequency_change(c: &mut Criterion) {
    let mut osc = Oscillator::new(OscillatorConfig {
        waveform: Waveform::Sine,
        frequency: 440.0,
        amplitude: 0.8,
        sample_rate: SAMPLE_RATE,
        ..Default::default()
    });

    c.bench_function("oscillator_frequency_change", |b| {
        b.iter(|| {
            osc.set_frequency(black_box(220.0 + _ as f32 % 1000.0));
            for _ in 0..10 {
                black_box(osc.next_sample());
            }
        })
    });
}

criterion_group!(
    oscillator_benches,
    bench_sine_wave,
    bench_square_wave,
    bench_sawtooth_wave,
    bench_triangle_wave,
    bench_noise_wave,
    bench_high_frequency,
    bench_oversampling_x2,
    bench_oversampling_x4,
    bench_block_processing,
    bench_frequency_change,
);

criterion_main!(oscillator_benches);
