//! Synth benchmarks for WAVELET audio engine
//!
//! Measures full synthesizer performance including polyphony and effects.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavelet::synth::Synth;
use wavelet::effects::EffectType;

const SAMPLE_RATE: f32 = 48000.0;

fn bench_synth_mono_single_voice(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    
    c.bench_function("synth_mono_single_voice", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_stereo_single_voice(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    
    c.bench_function("synth_stereo_single_voice", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_stereo());
            }
        })
    });
}

fn bench_synth_polyphony_4_voices(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 80);
    synth.note_on(64, 80);
    synth.note_on(67, 80);
    synth.note_on(71, 80);
    
    c.bench_function("synth_polyphony_4_voices", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_polyphony_8_voices(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    for note in [60, 62, 64, 65, 67, 69, 71, 72] {
        synth.note_on(note, 60);
    }
    
    c.bench_function("synth_polyphony_8_voices", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_polyphony_max_voices(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    for note in 0..16 {
        synth.note_on(note, 50);
    }
    
    c.bench_function("synth_polyphony_max_voices", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_block_1000_samples(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    
    c.bench_function("synth_block_1000_samples", |b| {
        b.iter(|| {
            black_box(synth.process_block_mono(1000));
        })
    });
}

fn bench_synth_with_zdf(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    synth.set_zdf_enabled(true);
    synth.set_zdf_cutoff(1500.0);
    synth.set_zdf_resonance(2.0);
    
    c.bench_function("synth_with_zdf", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_without_zdf(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    synth.set_zdf_enabled(false);
    
    c.bench_function("synth_without_zdf", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_with_saturation(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    synth.set_saturation_drive(2.0);
    synth.set_saturation_mix(0.5);
    
    c.bench_function("synth_with_saturation", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_with_delay(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    synth.set_effect_type(EffectType::Delay);
    synth.set_effect_mix(0.3);
    
    c.bench_function("synth_with_delay", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_with_reverb(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    synth.set_effect_type(EffectType::Reverb);
    synth.set_effect_mix(0.2);
    
    c.bench_function("synth_with_reverb", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_note_on_off(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    
    c.bench_function("synth_note_on_off", |b| {
        b.iter(|| {
            synth.note_on(black_box(60), black_box(100));
            for _ in 0..10 {
                black_box(synth.process_mono());
            }
            synth.note_off_specific(black_box(60));
        })
    });
}

fn bench_synth_parameter_changes(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    
    c.bench_function("synth_parameter_changes", |b| {
        b.iter(|| {
            for i in 0..100 {
                synth.set_filter_cutoff(500.0 + i as f32 % 2000.0);
                synth.set_filter_resonance(1.0 + (i % 5) as f32);
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_reset(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    synth.note_on(60, 100);
    
    c.bench_function("synth_reset", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                synth.process_mono();
            }
            synth.reset();
        })
    });
}

fn bench_synth_silence(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    // No notes
    
    c.bench_function("synth_silence", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

fn bench_synth_voice_stealing(c: &mut Criterion) {
    let mut synth = Synth::new(SAMPLE_RATE);
    
    c.bench_function("synth_voice_stealing", |b| {
        b.iter(|| {
            // Fill all voices
            for note in 0..16 {
                synth.note_on(note, 80);
            }
            // Steal voice
            synth.note_on(100, 80);
            // Process
            for _ in 0..100 {
                black_box(synth.process_mono());
            }
        })
    });
}

criterion_group!(
    synth_benches,
    bench_synth_mono_single_voice,
    bench_synth_stereo_single_voice,
    bench_synth_polyphony_4_voices,
    bench_synth_polyphony_8_voices,
    bench_synth_polyphony_max_voices,
    bench_synth_block_1000_samples,
    bench_synth_with_zdf,
    bench_synth_without_zdf,
    bench_synth_with_saturation,
    bench_synth_with_delay,
    bench_synth_with_reverb,
    bench_synth_note_on_off,
    bench_synth_parameter_changes,
    bench_synth_reset,
    bench_synth_silence,
    bench_synth_voice_stealing,
);

criterion_main!(synth_benches);
