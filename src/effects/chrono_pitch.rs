use super::*;

#[derive(Debug, Clone)]
pub struct ChronoPitch {
    pub tune: f32,
    pub window: f32,
    pub feedback: f32,
    pub depth: f32,
    pub highpass: f32,
    pub lowpass: f32,
    pub speed: f32,
    pub mix: f32,
    pitch_grain: Vec<f32>,
    grain_index: usize,
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
}

impl Default for ChronoPitch {
    fn default() -> Self {
        ChronoPitch {
            tune: 64.0,
            window: 50.0,
            feedback: 0.0,
            depth: 64.0,
            highpass: 20.0,
            lowpass: 20000.0,
            speed: 16.0,
            mix: 64.0,
            pitch_grain: Vec::new(),
            grain_index: 0,
            input_buffer: Vec::new(),
            output_buffer: Vec::new(),
        }
    }
}

impl ChronoPitch {
    pub fn new() -> Self {
        ChronoPitch::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let window_samples = (self.window / 1000.0 * sample_rate as f32) as usize;
        let pitch_factor = 2.0_f32.powf((self.tune - 64.0) / 12.0);

        if self.pitch_grain.len() < window_samples {
            self.pitch_grain.resize(window_samples, 0.0);
        }

        for (i, out) in output.iter_mut().enumerate() {
            let idx = i % window_samples;
            let pitch_idx = (self.grain_index as f32 * pitch_factor) as usize;

            if self.grain_index < window_samples {
                self.pitch_grain[idx] = input[i];
            }

            if pitch_idx < self.pitch_grain.len() {
                *out = self.pitch_grain[pitch_idx];
            } else {
                *out = 0.0;
            }

            self.grain_index = (self.grain_index + 1) % window_samples;
        }

        let wet = self.mix / 127.0;
        let dry = 1.0 - wet;

        for (i, out) in output.iter_mut().enumerate() {
            *out = input[i] * dry + *out * wet;
        }
    }
}
