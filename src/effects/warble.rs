use super::*;

#[derive(Debug, Clone)]
pub struct Warble {
    pub speed: f32,
    pub depth: f32,
    pub base: f32,
    pub width: f32,
    pub noise_level: f32,
    pub noise_hpf: f32,
    pub stereo: f32,
    pub mix: f32,
}

impl Default for Warble {
    fn default() -> Self {
        Warble {
            speed: 16.0,
            depth: 64.0,
            base: 64.0,
            width: 64.0,
            noise_level: 0.0,
            noise_hpf: 20.0,
            stereo: 64.0,
            mix: 64.0,
        }
    }
}

impl Warble {
    pub fn new() -> Self {
        Warble::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let base_freq = 20.0 + (self.base / 127.0) * 1980.0;
        let var_width = (self.width / 127.0) * 1000.0;
        let var_speed = self.speed / 127.0 * 20.0;
        let var_depth = self.depth / 127.0;
        let mix = self.mix / 127.0;
        let stereo_width = (self.stereo / 127.0) * 0.5;

        let delay_samples = (sample_rate as f32 / base_freq) as usize;
        let var_samples = (sample_rate as f32 / var_speed) as usize;

        let mut delay_buffer = vec![0.0; delay_samples * 2];
        let mut var_ptr = 0;
        let mut var_buffer = vec![0.0; var_samples.max(1)];
        let mut write_ptr = 0;
        let mut phase = 0.0f32;

        let hp_coeff = 2.0 * std::f32::consts::PI * self.noise_hpf / sample_rate as f32;
        let mut hp_state = 0.0f32;

        for (i, out) in output.iter_mut().enumerate() {
            let lfo = (phase * std::f32::consts::TAU).sin();
            let var_amount = lfo * var_width * var_depth;

            var_buffer[var_ptr] = var_amount;
            let current_var = var_buffer[(var_ptr + 1) % var_buffer.len()];

            let read_idx = (write_ptr as i32 - delay_samples as i32 + current_var as i32).abs()
                as usize
                % delay_buffer.len();
            let wet = delay_buffer[read_idx];

            let noise = rand::random::<f32>() * 2.0 - 1.0;
            hp_state = hp_state + hp_coeff * (noise - hp_state);
            let filtered_noise = noise - hp_state;

            delay_buffer[write_ptr] = input[i] + filtered_noise * (self.noise_level / 127.0);

            *out = input[i] * (1.0 - mix) + wet * mix;

            var_ptr = (var_ptr + 1) % var_buffer.len();
            write_ptr = (write_ptr + 1) % delay_buffer.len();
            phase += var_speed / sample_rate as f32;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }
    }
}
