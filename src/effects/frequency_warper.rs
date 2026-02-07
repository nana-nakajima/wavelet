use super::*;

#[derive(Debug, Clone)]
pub struct FrequencyWarper {
    pub speed: f32,
    pub depth: f32,
    pub spread: f32,
    pub lag: f32,
    pub shift: f32,
    pub spread_band: f32,
    pub subtle_band: f32,
    pub mix: f32,
}

impl Default for FrequencyWarper {
    fn default() -> Self {
        FrequencyWarper {
            speed: 16.0,
            depth: 64.0,
            spread: 0.0,
            lag: 0.0,
            shift: 64.0,
            spread_band: 64.0,
            subtle_band: 64.0,
            mix: 64.0,
        }
    }
}

impl FrequencyWarper {
    pub fn new() -> Self {
        FrequencyWarper::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let lfo_speed = self.speed / 127.0 * 10.0;
        let depth = self.depth / 127.0;
        let shift_semitones = (self.shift - 64.0) / 12.0;
        let mix = self.mix / 127.0;

        let mut phase = 0.0f32;
        let mut lag_state = 0.0f32;

        for (i, out) in output.iter_mut().enumerate() {
            let lfo = (phase * std::f32::consts::TAU).sin();
            let modulated_shift = shift_semitones + lfo * depth;

            let shift_factor = 2.0_f32.powf(modulated_shift / 12.0);

            let target = input[i] * shift_factor;
            lag_state = lag_state + (self.lag / 127.0) * (target - lag_state);

            *out = input[i] * (1.0 - mix) + lag_state * mix;

            phase += lfo_speed / sample_rate as f32;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }
    }
}
