use super::*;

#[derive(Debug, Clone)]
pub struct Dirtshaper {
    pub drive: f32,
    pub rectif: f32,
    pub highpass: f32,
    pub lowpass: f32,
    pub noise: f32,
    pub noise_freq: f32,
    pub noise_reso: f32,
    pub mix: f32,
}

impl Default for Dirtshaper {
    fn default() -> Self {
        Dirtshaper {
            drive: 0.0,
            rectif: 0.0,
            highpass: 20.0,
            lowpass: 20000.0,
            noise: 0.0,
            noise_freq: 1000.0,
            noise_reso: 0.0,
            mix: 64.0,
        }
    }
}

impl Dirtshaper {
    pub fn new() -> Self {
        Dirtshaper::default()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        let drive = 1.0 + (self.drive / 127.0) * 10.0;
        let noise_amount = self.noise / 127.0;
        let mix = self.mix / 127.0;

        let hp_coeff = if self.highpass > 20.0 {
            Some(2.0 * std::f32::consts::PI * self.highpass / sample_rate as f32)
        } else {
            None
        };

        let lp_coeff = if self.lowpass < 20000.0 {
            Some(2.0 * std::f32::consts::PI * self.lowpass / sample_rate as f32)
        } else {
            None
        };

        let mut prev_sample = 0.0f32;

        for (i, out) in output.iter_mut().enumerate() {
            let mut sample = input[i] * drive;

            match self.rectif {
                r if r < 42.0 => {}
                r if r < 84.0 => {
                    sample = sample.abs();
                }
                _ => {
                    sample = sample.abs() * 2.0 - 1.0;
                }
            }

            let noise = rand::random::<f32>() * 2.0 - 1.0;
            sample += noise * noise_amount;

            if let Some(coeff) = hp_coeff {
                sample = sample - sample * coeff;
            }

            if let Some(coeff) = lp_coeff {
                sample = sample * coeff + prev_sample * (1.0 - coeff);
            }

            prev_sample = sample;

            *out = input[i] * (1.0 - mix) + sample * mix;
        }
    }
}
