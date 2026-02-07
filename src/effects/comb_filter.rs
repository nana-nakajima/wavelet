use super::*;

#[derive(Debug, Clone)]
pub struct CombFilter {
    pub speed: f32,
    pub depth: f32,
    pub spread: f32,
    pub detune: f32,
    pub frequency: f32,
    pub feedback: f32,
    pub lowpass: f32,
    pub mix: f32,
    delay_lines: [Vec<f32>; 2],
    write_index: usize,
}

impl Default for CombFilter {
    fn default() -> Self {
        CombFilter {
            speed: 16.0,
            depth: 64.0,
            spread: 0.0,
            detune: 0.0,
            frequency: 200.0,
            feedback: 0.0,
            lowpass: 20000.0,
            mix: 64.0,
            delay_lines: [Vec::new(), Vec::new()],
            write_index: 0,
        }
    }
}

impl CombFilter {
    pub fn new() -> Self {
        CombFilter::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let delay_samples = (1.0 / self.frequency * sample_rate as f32) as usize;
        let feedback_amount = self.feedback / 127.0;
        let spread_amount = self.spread / 127.0;

        if self.delay_lines[0].len() < delay_samples + 1024 {
            for line in &mut self.delay_lines {
                line.resize(delay_samples + 1024, 0.0);
            }
        }

        for (i, out) in output.iter_mut().enumerate() {
            let left_input = input[i];
            let right_input = input[i];

            let delay_read = (self.write_index + delay_samples) % self.delay_lines[0].len();
            let spread_read = (self.write_index
                + (delay_samples as f32 * (1.0 + spread_amount)) as usize)
                % self.delay_lines[0].len();

            let left_feedback = self.delay_lines[0][delay_read] * feedback_amount;
            let right_feedback =
                self.delay_lines[1][spread_read] * feedback_amount * (1.0 + self.detune / 127.0);

            self.delay_lines[0][self.write_index] = left_input + left_feedback;
            self.delay_lines[1][self.write_index] = right_input + right_feedback;

            let wet_left = self.delay_lines[0][delay_read] * (self.depth / 127.0);
            let wet_right = self.delay_lines[1][spread_read] * (self.depth / 127.0);

            *out = (left_input + right_input) * 0.5 * (1.0 - self.depth / 127.0)
                + (wet_left + wet_right) * 0.5;

            self.write_index = (self.write_index + 1) % self.delay_lines[0].len();
        }

        let mix = self.mix / 127.0;
        for (i, out) in output.iter_mut().enumerate() {
            *out = input[i] * (1.0 - mix) + *out * mix;
        }
    }
}
