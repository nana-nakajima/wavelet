use super::*;

#[derive(Debug, Clone)]
pub struct DaisyDelay {
    pub drive: f32,
    pub time: f32,
    pub feedback: f32,
    pub width: f32,
    pub modulation: f32,
    pub skew: f32,
    pub filter: f32,
}

impl Default for DaisyDelay {
    fn default() -> Self {
        DaisyDelay {
            drive: 0.0,
            time: 64.0,
            feedback: 0.0,
            width: 64.0,
            modulation: 0.0,
            skew: 64.0,
            filter: 20000.0,
        }
    }
}

impl DaisyDelay {
    pub fn new() -> Self {
        DaisyDelay::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let delay_ms = (self.time / 127.0 * 2000.0 + 1.0) as usize;
        let delay_samples = (delay_ms as f32 / 1000.0 * sample_rate as f32) as usize;
        let feedback_amount = self.feedback / 127.0 * 0.9;
        let drive = 1.0 + (self.drive / 127.0) * 10.0;
        let mod_amount = self.modulation / 127.0 * 10.0;
        let skew = (self.skew - 64.0) / 64.0;
        let filter_freq = self.filter;

        let mut delay_buffer = vec![0.0; delay_samples * 2];
        let mut write_ptr = 0;
        let mut mod_phase = 0.0f32;

        let lp_coeff = 2.0 * std::f32::consts::PI * filter_freq / sample_rate as f32;
        let mut lp_state = 0.0f32;

        for (i, out) in output.iter_mut().enumerate() {
            let lfo = (mod_phase * std::f32::consts::TAU).sin();
            let modulated_delay =
                (delay_samples as f32 + lfo * mod_amount * sample_rate as f32 / 1000.0) as usize;
            let skew_offset = (skew * modulated_delay as f32) as usize;

            let read_ptr = (write_ptr + modulated_delay + skew_offset) % delay_buffer.len();
            let wet = delay_buffer[read_ptr];

            let saturated = (wet * drive).tanh();
            lp_state = lp_state + lp_coeff * (saturated - lp_state);
            let filtered = lp_state;

            let feedback = filtered * feedback_amount;
            delay_buffer[write_ptr] = input[i] + feedback;
            delay_buffer[write_ptr + 1] = input[i] + feedback;

            *out = input[i] * 0.5 + wet * 0.5;

            mod_phase += 0.5 / sample_rate as f32;
            if mod_phase >= 1.0 {
                mod_phase -= 1.0;
            }
            write_ptr = (write_ptr + 2) % delay_buffer.len();
        }
    }
}
