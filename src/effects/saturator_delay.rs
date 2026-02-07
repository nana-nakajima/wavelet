use super::*;

#[derive(Debug, Clone)]
pub struct SaturatorDelay {
    pub time: f32,
    pub pingpong: f32,
    pub width: f32,
    pub feedback: f32,
    pub highpass: f32,
    pub lowpass: f32,
    pub mix: f32,
}

impl Default for SaturatorDelay {
    fn default() -> Self {
        SaturatorDelay {
            time: 64.0,
            pingpong: 0.0,
            width: 64.0,
            feedback: 0.0,
            highpass: 20.0,
            lowpass: 20000.0,
            mix: 64.0,
        }
    }
}

impl SaturatorDelay {
    pub fn new() -> Self {
        SaturatorDelay::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let beat_duration = 60.0 / 120.0;
        let time_ms = (self.time / 127.0 * 2000.0 + 10.0) as usize;
        let delay_samples = (time_ms as f32 / 1000.0 * sample_rate as f32) as usize;
        let feedback_amount = self.feedback / 127.0 * 0.95;
        let pingpong = self.pingpong / 127.0;
        let stereo_width = (self.width / 127.0) * 0.5;
        let mix = self.mix / 127.0;
        let hp_freq = self.highpass;
        let lp_freq = self.lowpass;

        let mut delay_buffer = vec![0.0; delay_samples * 2];
        let mut write_index = 0;
        let hp_coeff = 2.0 * std::f32::consts::PI * hp_freq / sample_rate as f32;
        let lp_coeff = 2.0 * std::f32::consts::PI * lp_freq / sample_rate as f32;
        let mut hp_state = (0.0, 0.0);
        let mut lp_state = (0.0, 0.0);

        for (i, out) in output.iter_mut().enumerate() {
            let left_in = input[i];
            let right_in = input[i];

            let read_index = (write_index + delay_samples) % delay_buffer.len();

            let wet_left = delay_buffer[read_index];
            let wet_right = delay_buffer[read_index + 1];

            hp_state.0 = hp_state.0 + hp_coeff * (wet_left - hp_state.0);
            hp_state.1 = hp_state.1 + hp_coeff * (wet_right - hp_state.1);
            let hp_wet_left = wet_left - hp_state.0;
            let hp_wet_right = wet_right - hp_state.1;

            lp_state.0 = lp_state.0 + lp_coeff * (hp_wet_left - lp_state.0);
            lp_state.1 = lp_state.1 + lp_coeff * (hp_wet_right - lp_state.1);
            let lp_wet_left = lp_state.0;
            let lp_wet_right = lp_state.1;

            let feedback_left = lp_wet_left * feedback_amount;
            let feedback_right = lp_wet_right * feedback_amount;

            delay_buffer[write_index] = left_in
                + if pingpong > 0.5 {
                    feedback_right
                } else {
                    feedback_left
                };
            delay_buffer[write_index + 1] = right_in
                + if pingpong > 0.5 {
                    feedback_left
                } else {
                    feedback_right
                };

            *out = (left_in + right_in) * 0.5 * (1.0 - mix)
                + (lp_wet_left * (1.0 - stereo_width) + wet_right * stereo_width) * mix * 0.5
                + (lp_wet_right * (1.0 - stereo_width) + wet_left * stereo_width) * mix * 0.5;

            write_index = (write_index + 2) % delay_buffer.len();
        }
    }
}
