use super::*;

#[derive(Debug, Clone)]
pub struct SupervoidReverb {
    pub pre_delay: f32,
    pub decay: f32,
    pub frequency: f32,
    pub gain: f32,
    pub highpass: f32,
    pub lowpass: f32,
    pub mix: f32,
}

impl Default for SupervoidReverb {
    fn default() -> Self {
        SupervoidReverb {
            pre_delay: 0.0,
            decay: 64.0,
            frequency: 64.0,
            gain: 64.0,
            highpass: 20.0,
            lowpass: 20000.0,
            mix: 64.0,
        }
    }
}

impl SupervoidReverb {
    pub fn new() -> Self {
        SupervoidReverb::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let decay_time = 0.1 + (self.decay / 127.0) * 4.0;
        let damping_freq = 100.0 + (self.frequency / 127.0) * 10000.0;
        let mix = self.mix / 127.0;

        let delay_size = (self.pre_delay / 1000.0 * sample_rate as f32) as usize;
        let mut pre_delay = vec![0.0; delay_size.max(1024)];
        let mut write_ptr = 0;

        let reverb_size = (sample_rate as f32 * decay_time * 4.0) as usize;
        let mut reverb_buffer = vec![0.0; reverb_size.max(10240)];
        let mut reverb_ptr = 0;

        let damp_coeff = 2.0 * std::f32::consts::PI * damping_freq / sample_rate as f32;
        let mut damp_state = 0.0f32;

        for (i, out) in output.iter_mut().enumerate() {
            let input_sample = input[i];

            pre_delay[write_ptr] = input_sample;
            let delayed = pre_delay[(write_ptr + delay_size.saturating_sub(1)) % pre_delay.len()];

            let feedback = reverb_buffer[reverb_ptr];
            damp_state = damp_state + damp_coeff * (feedback - damp_state);
            let damped = feedback - damp_state;

            let new_reverb = delayed + damped * 0.7;
            reverb_buffer[reverb_ptr] = new_reverb;

            *out = input_sample * (1.0 - mix) + damped * mix;

            write_ptr = (write_ptr + 1) % pre_delay.len();
            reverb_ptr = (reverb_ptr + 1) % reverb_buffer.len();
        }
    }
}
