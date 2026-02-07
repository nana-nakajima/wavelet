use super::*;

#[derive(Debug, Clone)]
pub struct InfiniteFlanger {
    pub speed: f32,
    pub depth: f32,
    pub tune: f32,
    pub feedback: f32,
    pub lowpass: f32,
}

impl Default for InfiniteFlanger {
    fn default() -> Self {
        InfiniteFlanger {
            speed: 16.0,
            depth: 64.0,
            tune: 64.0,
            feedback: 0.0,
            lowpass: 20000.0,
        }
    }
}

impl InfiniteFlanger {
    pub fn new() -> Self {
        InfiniteFlanger::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let delay_ms = 0.5 + (self.depth / 127.0) * 10.0;
        let delay_samples = (delay_ms / 1000.0 * sample_rate as f32) as usize;
        let feedback_amount = self.feedback / 127.0 * 0.95;
        let lfo_speed = self.speed / 127.0 * 10.0;
        let lfo_depth = delay_samples as f32 / 2.0;
        let phase_offset = self.tune / 127.0 * std::f32::consts::PI;

        let mut delay_buffer = vec![0.0; delay_samples * 2];
        let mut write_index = 0;
        let mut lfo_phase = 0.0;

        for (i, out) in output.iter_mut().enumerate() {
            let lfo = (lfo_phase + phase_offset).sin();
            let modulated_delay = (delay_samples as f32 + lfo * lfo_depth) as usize;
            let read_index = (write_index + modulated_delay) % delay_buffer.len();

            let wet = delay_buffer[read_index];
            delay_buffer[write_index] = input[i] + wet * feedback_amount;
            *out = input[i] * 0.5 + wet * 0.5;

            lfo_phase += 2.0 * std::f32::consts::PI * lfo_speed / sample_rate as f32;
            if lfo_phase > 2.0 * std::f32::consts::PI {
                lfo_phase -= 2.0 * std::f32::consts::PI;
            }
            write_index = (write_index + 1) % delay_buffer.len();
        }
    }
}
