use super::*;

#[derive(Debug, Clone)]
pub struct PanoramicChorus {
    pub depth: f32,
    pub speed: f32,
    pub highpass: f32,
    pub width: f32,
    pub mix: f32,
}

impl Default for PanoramicChorus {
    fn default() -> Self {
        PanoramicChorus {
            depth: 64.0,
            speed: 16.0,
            highpass: 20.0,
            width: 64.0,
            mix: 64.0,
        }
    }
}

impl PanoramicChorus {
    pub fn new() -> Self {
        PanoramicChorus::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let delay_modulation = (self.depth / 127.0) * 5.0;
        let lfo_speed = self.speed / 127.0 * 5.0;
        let stereo_width = (self.width / 127.0) * 0.5 + 0.5;
        let hp_freq = self.highpass;
        let mix = self.mix / 127.0;

        let delay_samples = (sample_rate as f32 * 0.02) as usize;
        let mut left_delay = vec![0.0; delay_samples * 2];
        let mut right_delay = vec![0.0; delay_samples * 2];
        let mut left_write = 0;
        let mut right_write = 0;
        let mut lfo_phase = 0.0;
        let hp_coeff = 2.0 * std::f32::consts::PI * hp_freq / sample_rate as f32;
        let mut hp_state = (0.0, 0.0);

        for (i, out) in output.iter_mut().enumerate() {
            let left_in = input[i];
            let right_in = input[i];

            let lfo = (lfo_phase * 2.0 * std::f32::consts::PI).sin();
            let modulated_delay = (delay_samples as f32
                + lfo * delay_modulation * sample_rate as f32 / 1000.0)
                as usize;
            let left_read = (left_write + modulated_delay) % left_delay.len();
            let right_read = (right_write + modulated_delay) % right_delay.len();

            let left_wet = left_delay[left_read];
            let right_wet = right_delay[right_read];

            left_delay[left_write] = left_in;
            right_delay[right_write] = right_in;

            hp_state.0 = hp_state.0 + hp_coeff * (left_in - hp_state.0);
            hp_state.1 = hp_state.1 + hp_coeff * (right_in - hp_state.1);
            let left_hp = left_in - hp_state.0;
            let right_hp = right_in - hp_state.1;

            *out = (left_in + right_in) * 0.5 * (1.0 - mix)
                + (left_wet + right_hp * stereo_width) * mix * 0.25
                + (right_wet + left_hp * stereo_width) * mix * 0.25;

            lfo_phase += lfo_speed / sample_rate as f32;
            if lfo_phase >= 1.0 {
                lfo_phase -= 1.0;
            }
            left_write = (left_write + 1) % left_delay.len();
            right_write = (right_write + 1) % right_delay.len();
        }
    }
}
