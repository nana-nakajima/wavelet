use super::*;

#[derive(Debug, Clone)]
pub struct Degrader {
    pub bit_depth: f32,
    pub overample: f32,
    pub sample_rate_reduction: f32,
    pub dropout: f32,
    pub rate: f32,
    pub depth: f32,
    pub freeze: f32,
    pub freeze_time: f32,
}

impl Default for Degrader {
    fn default() -> Self {
        Degrader {
            bit_depth: 16.0,
            overample: 0.0,
            sample_rate_reduction: 0.0,
            dropout: 0.0,
            rate: 64.0,
            depth: 64.0,
            freeze: 0.0,
            freeze_time: 0.0,
        }
    }
}

impl Degrader {
    pub fn new() -> Self {
        Degrader::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let bits = (self.bit_depth / 1.0) as u32;
        let sr_divisor = (self.sample_rate_reduction / 1.0) as u32 + 1;
        let step = 2.0_f32.powi(bits as i32);

        for (i, out) in output.iter_mut().enumerate() {
            let input_sample = input[i];

            if self.freeze > 0.5 {
                continue;
            }

            let quantized = (input_sample * step).floor() / step;
            let noise = if self.overample > 0.0 {
                (rand::random::<f32>() - 0.5) * (1.0 / step) * (self.overample / 127.0)
            } else {
                0.0
            };

            let sr_noise = if self.sample_rate_reduction > 0.0 && i as u32 % sr_divisor == 0 {
                rand::random::<f32>() * 2.0 - 1.0
            } else {
                0.0
            };

            *out = (quantized + noise + sr_noise * (self.rate / 127.0)) * (self.depth / 127.0);
        }
    }
}
