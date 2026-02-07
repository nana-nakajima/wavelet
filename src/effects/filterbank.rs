use super::*;

#[derive(Debug, Clone)]
pub struct Filterbank {
    pub gains: [f32; 8],
}

impl Default for Filterbank {
    fn default() -> Self {
        Filterbank { gains: [64.0; 8] }
    }
}

impl Filterbank {
    pub fn new() -> Self {
        Filterbank::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let frequencies = [90.0, 180.0, 360.0, 720.0, 1440.0, 2880.0, 5760.0, 11520.0];
        let mut filters = vec![BiquadFilter::low_pass(frequencies[0], 1.0, sample_rate); 8];
        let mut highpass = BiquadFilter::high_pass(20.0, 1.0, sample_rate);

        for (i, (freq, gain)) in frequencies.iter().zip(self.gains.iter()).enumerate() {
            let q = 1.0 + (i as f32 * 0.5);
            let gain_db = (gain / 64.0 - 1.0) * 12.0;
            filters[i] = BiquadFilter::peak(freq.clone(), q, gain_db, sample_rate);
        }

        let mut band_outputs = vec![vec![0.0; input.len()]; 8];
        for (i, filter) in filters.iter_mut().enumerate() {
            for (j, out) in band_outputs[i].iter_mut().enumerate() {
                *out = filter.process_sample(input[j]);
            }
        }

        for (i, out) in output.iter_mut().enumerate() {
            let mut sum = 0.0;
            for band in &band_outputs {
                sum += band[i] / 8.0;
            }
            *out = sum;
        }
    }
}

#[derive(Debug, Clone)]
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    pub fn low_pass(freq: f32, q: f32, sample_rate: u32) -> Self {
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate as f32;
        let alpha = omega.sin() / (2.0 * q);
        let cos = omega.cos();

        let b0 = (1.0 - cos) / 2.0;
        let b1 = 1.0 - cos;
        let b2 = (1.0 - cos) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;

        BiquadFilter {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn high_pass(freq: f32, q: f32, sample_rate: u32) -> Self {
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate as f32;
        let alpha = omega.sin() / (2.0 * q);
        let cos = omega.cos();

        let b0 = (1.0 + cos) / 2.0;
        let b1 = -(1.0 + cos);
        let b2 = (1.0 + cos) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;

        BiquadFilter {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn peak(freq: f32, q: f32, gain_db: f32, sample_rate: u32) -> Self {
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate as f32;
        let alpha = omega.sin() / (2.0 * q);
        let cos = omega.cos();
        let a = 10.0_f32.powf(gain_db / 40.0);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha / a;

        BiquadFilter {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}
