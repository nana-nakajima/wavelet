use super::*;

#[derive(Debug, Clone)]
pub struct Phase98 {
    pub speed: f32,
    pub depth: f32,
    pub shape: f32,
    pub lag: f32,
    pub frequency: f32,
    pub feedback: f32,
    pub stages: u8,
    pub mix: f32,
}

impl Default for Phase98 {
    fn default() -> Self {
        Phase98 {
            speed: 16.0,
            depth: 64.0,
            shape: 0.0,
            lag: 0.0,
            frequency: 500.0,
            feedback: 0.0,
            stages: 4,
            mix: 64.0,
        }
    }
}

impl Phase98 {
    pub fn new() -> Self {
        Phase98::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let num_stages = if self.stages == 6 { 6 } else { 4 };
        let wet_amount = self.depth / 127.0;
        let lfo_speed = self.speed / 127.0 * 10.0;
        let feedback_amount = self.feedback / 127.0 * 0.9;
        let center_freq = 20.0 + (self.frequency / 127.0) * 1980.0;
        let mix = self.mix / 127.0;

        let mut allpass_coeffs = vec![0.0; num_stages];
        let mut ap_states = vec![vec![0.0; 2]; num_stages];
        let mut lfo_phase = 0.0;

        for (i, out) in output.iter_mut().enumerate() {
            let lfo = (lfo_phase * std::f32::consts::PI * 2.0).sin();
            let freq_offset = center_freq * (1.0 + lfo * wet_amount * 2.0);
            let coeff = (1.0 - freq_offset * std::f32::consts::PI / sample_rate as f32)
                / (1.0 + freq_offset * std::f32::consts::PI / sample_rate as f32);

            let mut sample = input[i];

            for stage in 0..num_stages {
                allpass_coeffs[stage] = coeff;
            }

            for stage in 0..num_stages {
                let coef = allpass_coeffs[stage];
                let input_sample = sample;
                let ap_output = coef * input_sample + ap_states[stage][0];
                ap_states[stage][0] = input_sample - coef * ap_output;
                sample = ap_output;
            }

            sample = sample + input[i] * feedback_amount;

            *out = input[i] * (1.0 - mix) + sample * mix;

            lfo_phase += lfo_speed / sample_rate as f32;
            if lfo_phase >= 1.0 {
                lfo_phase -= 1.0;
            }
        }
    }
}
