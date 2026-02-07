use super::*;

#[derive(Debug, Clone)]
pub struct RumsklangReverb {
    pub pre: f32,
    pub early: f32,
    pub damp: f32,
    pub size: f32,
    pub low_cut: f32,
    pub high_cut: f32,
}

impl Default for RumsklangReverb {
    fn default() -> Self {
        RumsklangReverb {
            pre: 0.0,
            early: 64.0,
            damp: 64.0,
            size: 64.0,
            low_cut: 20.0,
            high_cut: 20000.0,
        }
    }
}

impl RumsklangReverb {
    pub fn new() -> Self {
        RumsklangReverb::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let early_level = self.early / 127.0;
        let damping = self.damp / 127.0;
        let room_size = 0.5 + (self.size / 127.0) * 1.5;

        let lp_coeff = 2.0 * std::f32::consts::PI * self.high_cut / sample_rate as f32;
        let hp_coeff = 2.0 * std::f32::consts::PI * self.low_cut / sample_rate as f32;
        let mut lp_state = 0.0f32;
        let mut hp_state = 0.0f32;

        let comb_sizes = [
            (sample_rate as f32 * 0.05 * room_size) as usize,
            (sample_rate as f32 * 0.04 * room_size) as usize,
            (sample_rate as f32 * 0.03 * room_size) as usize,
            (sample_rate as f32 * 0.02 * room_size) as usize,
        ];

        let mut comb_buffers: Vec<Vec<f32>> =
            comb_sizes.iter().map(|&s| vec![0.0; s.max(1000)]).collect();
        let mut comb_ptrs = vec![0usize; comb_sizes.len()];

        let ap_size = (sample_rate as f32 * 0.01 * room_size) as usize;
        let mut ap_buffer = vec![0.0; ap_size.max(1000)];
        let mut ap_ptr = 0;

        for (i, out) in output.iter_mut().enumerate() {
            let input_sample = input[i];

            let hp_filtered = input_sample - hp_state;
            hp_state = hp_state + hp_coeff * (input_sample - hp_state);

            let mut comb_sum = 0.0f32;
            for (j, buffer) in comb_buffers.iter_mut().enumerate() {
                let read_ptr = (comb_ptrs[j] + buffer.len() - comb_sizes[j]) % buffer.len();
                let feedback = buffer[read_ptr] * (0.9 - damping * 0.3);
                buffer[comb_ptrs[j]] = hp_filtered + feedback;
                comb_sum += buffer[read_ptr];
                comb_ptrs[j] = (comb_ptrs[j] + 1) % buffer.len();
            }

            let ap_input = comb_sum / comb_sizes.len() as f32;
            let ap_read = ap_buffer[(ap_ptr + ap_size / 2) % ap_buffer.len()];
            let ap_output = ap_read + ap_input * 0.5;
            ap_buffer[ap_ptr] = ap_output;
            ap_ptr = (ap_ptr + 1) % ap_buffer.len();

            let lp_filtered = ap_output - lp_state;
            lp_state = lp_state + lp_coeff * (ap_output - lp_state);

            *out = input_sample * (1.0 - early_level) + lp_filtered * early_level;
        }
    }
}
