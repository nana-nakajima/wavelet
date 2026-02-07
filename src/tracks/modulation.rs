use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModDestination {
    None,
    Tune,
    PlayMode,
    SampleSlot,
    Start,
    End,
    LoopStart,
    LoopEnd,
    LoopCrossfade,
    FltrType,
    FltrFreq,
    FltrReso,
    FltrEnvDepth,
    FltrAtk,
    FltrDec,
    FltrSus,
    FltrRel,
    BaseFreq,
    Width,
    AmpAtk,
    AmpHold,
    AmpDec,
    AmpSus,
    AmpRel,
    Overdrive,
    Pan,
    Volume,
    Fx1Mix,
    Fx2Mix,
    Routing,
    Send1,
    Send2,
    Send3,
}

impl Default for ModDestination {
    fn default() -> Self {
        ModDestination::None
    }
}

#[derive(Debug, Clone)]
pub struct VoiceLfo {
    pub waveform: LfoWaveform,
    pub speed: f32,
    pub tempo_sync: bool,
    pub tempo_num: u8,
    pub tempo_denom: u8,
    pub multiplier: u16,
    pub fade: i8,
    pub depth: i8,
    pub start_phase: f32,
    pub mode: LfoMode,
    pub destinations: Vec<ModDestination>,
    phase: f32,
}

impl Default for VoiceLfo {
    fn default() -> Self {
        VoiceLfo {
            waveform: LfoWaveform::Triangle,
            speed: 1.0,
            tempo_sync: true,
            tempo_num: 8,
            tempo_denom: 8,
            multiplier: 64,
            fade: 0,
            depth: 64,
            start_phase: 0.0,
            mode: LfoMode::Free,
            destinations: Vec::new(),
            phase: 0.0,
        }
    }
}

impl VoiceLfo {
    pub fn new() -> Self {
        VoiceLfo::default()
    }

    pub fn reset(&mut self) {
        self.phase = self.start_phase;
    }

    pub fn process(&mut self, sample_rate: f32, tempo: f32, samples: usize) -> f32 {
        let phase_increment = if self.tempo_sync {
            let beat_duration = 60.0 / tempo;
            let note_duration = beat_duration * (self.tempo_num as f32 / self.tempo_denom as f32);
            let multiplier = self.multiplier as f32 / 64.0;
            1.0 / (note_duration * sample_rate * multiplier)
        } else {
            self.speed / sample_rate
        };

        let fade_factor = if self.fade != 0 {
            let fade_samples = self.fade.abs() as f32 * 0.01 * sample_rate;
            let current_sample = self.phase * sample_rate / self.speed;
            (current_sample / fade_samples).min(1.0)
        } else {
            1.0
        };

        let depth_factor = (self.depth as f32 / 64.0) * fade_factor;
        let increment = phase_increment * depth_factor;

        for _ in 0..samples {
            self.phase += increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        self.value() * depth_factor
    }

    fn value(&self) -> f32 {
        match self.waveform {
            LfoWaveform::Triangle => {
                let mut v = 4.0 * (self.phase - 0.5).abs() - 1.0;
                if v < -1.0 {
                    v = -1.0;
                }
                v
            }
            LfoWaveform::Sine => (self.phase * std::f32::consts::TAU).sin(),
            LfoWaveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoWaveform::Sawtooth => 2.0 * self.phase - 1.0,
            LfoWaveform::Random => {
                use rand::random;
                (random::<f32>() * 2.0 - 1.0)
            }
            LfoWaveform::Exponential => {
                let exp = if self.phase < 0.5 {
                    2.0 * self.phase
                } else {
                    2.0 * (1.0 - self.phase)
                };
                exp.exp() / std::f32::consts::E
            }
            LfoWaveform::Ramp => {
                if self.phase < 0.5 {
                    2.0 * self.phase
                } else {
                    2.0 * self.phase - 2.0
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModEnvelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub reset: bool,
    pub depth: i8,
    pub destination: ModDestination,
    current_stage: ModEnvelopeStage,
    stage_time: f32,
    current_value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModEnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

impl Default for ModEnvelope {
    fn default() -> Self {
        ModEnvelope {
            attack: 0.0,
            decay: 100.0,
            sustain: 1.0,
            release: 100.0,
            reset: false,
            depth: 64,
            destination: ModDestination::None,
            current_stage: ModEnvelopeStage::Idle,
            stage_time: 0.0,
            current_value: 0.0,
        }
    }
}

impl ModEnvelope {
    pub fn new() -> Self {
        ModEnvelope::default()
    }

    pub fn trigger(&mut self) {
        self.current_stage = ModEnvelopeStage::Attack;
        self.stage_time = 0.0;
        self.current_value = 0.0;
    }

    pub fn release(&mut self) {
        if self.current_stage != ModEnvelopeStage::Idle {
            self.current_stage = ModEnvelopeStage::Release;
            self.stage_time = 0.0;
        }
    }

    pub fn reset(&mut self) {
        self.current_stage = ModEnvelopeStage::Idle;
        self.stage_time = 0.0;
        self.current_value = 0.0;
    }

    pub fn process(&mut self, delta_time: f32) -> f32 {
        let depth_factor = self.depth as f32 / 64.0;

        match self.current_stage {
            ModEnvelopeStage::Idle => {
                if self.reset {
                    self.current_value = 0.0;
                }
                0.0
            }
            ModEnvelopeStage::Attack => {
                self.stage_time += delta_time;
                let progress = if self.attack > 0.0 {
                    (self.stage_time / self.attack).min(1.0)
                } else {
                    1.0
                };
                self.current_value = progress;
                if progress >= 1.0 {
                    self.current_stage = ModEnvelopeStage::Decay;
                    self.stage_time = 0.0;
                }
                self.current_value * depth_factor
            }
            ModEnvelopeStage::Decay => {
                self.stage_time += delta_time;
                let progress = if self.decay > 0.0 {
                    (self.stage_time / self.decay).min(1.0)
                } else {
                    1.0
                };
                self.current_value = 1.0 - (1.0 - self.sustain) * progress;
                if progress >= 1.0 {
                    self.current_stage = ModEnvelopeStage::Sustain;
                }
                self.current_value * depth_factor
            }
            ModEnvelopeStage::Sustain => {
                self.current_value = self.sustain;
                self.current_value * depth_factor
            }
            ModEnvelopeStage::Release => {
                self.stage_time += delta_time;
                let progress = if self.release > 0.0 {
                    (self.stage_time / self.release).min(1.0)
                } else {
                    1.0
                };
                self.current_value = self.sustain * (1.0 - progress);
                if progress >= 1.0 {
                    self.current_stage = ModEnvelopeStage::Finished;
                }
                self.current_value * depth_factor
            }
            ModEnvelopeStage::Finished => 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.current_stage != ModEnvelopeStage::Idle
            && self.current_stage != ModEnvelopeStage::Finished
    }

    pub fn is_released(&self) -> bool {
        self.current_stage == ModEnvelopeStage::Release
            || self.current_stage == ModEnvelopeStage::Finished
    }
}

#[derive(Debug, Clone)]
pub struct FxLfo {
    pub waveform: LfoWaveform,
    pub speed: f32,
    pub tempo_sync: bool,
    pub tempo_num: u8,
    pub tempo_denom: u8,
    pub depth: i8,
    pub phase_offset: f32,
    pub destination: String,
    phase: f32,
}

impl Default for FxLfo {
    fn default() -> Self {
        FxLfo {
            waveform: LfoWaveform::Triangle,
            speed: 0.5,
            tempo_sync: true,
            tempo_num: 8,
            tempo_denom: 8,
            depth: 64,
            phase_offset: 0.0,
            destination: String::new(),
            phase: 0.0,
        }
    }
}

impl FxLfo {
    pub fn new() -> Self {
        FxLfo::default()
    }

    pub fn reset(&mut self) {
        self.phase = self.phase_offset;
    }

    pub fn process(&mut self, sample_rate: f32, tempo: f32, samples: usize) -> f32 {
        let beat_duration = 60.0 / tempo;
        let note_duration = beat_duration * (self.tempo_num as f32 / self.tempo_denom as f32);
        let phase_increment = 1.0 / (note_duration * sample_rate);

        let depth_factor = self.depth as f32 / 64.0;

        for _ in 0..samples {
            self.phase += phase_increment;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        let value = match self.waveform {
            LfoWaveform::Triangle => 4.0 * (self.phase - 0.5).abs() - 1.0,
            LfoWaveform::Sine => (self.phase * std::f32::consts::TAU).sin(),
            LfoWaveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoWaveform::Sawtooth => 2.0 * self.phase - 1.0,
            LfoWaveform::Random => rand::random::<f32>() * 2.0 - 1.0,
            LfoWaveform::Exponential => {
                let exp = if self.phase < 0.5 {
                    2.0 * self.phase
                } else {
                    2.0 * (1.0 - self.phase)
                };
                exp.exp() / std::f32::consts::E
            }
            LfoWaveform::Ramp => {
                if self.phase < 0.5 {
                    2.0 * self.phase
                } else {
                    2.0 * self.phase - 2.0
                }
            }
        };

        value * depth_factor
    }
}
