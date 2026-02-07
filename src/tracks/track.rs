use super::*;
use crate::{Arpeggiator, EffectType};

pub trait TrackBehavior {
    fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32);
    fn trigger(&mut self, note: u8, velocity: f32);
    fn set_param(&mut self, param: &str, value: f32);
    fn get_param(&self, param: &str) -> f32;
    fn fx_slots(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: u8,
    pub track_type: TrackType,
    pub name: String,
    pub machine: Machine,
    pub multimode_filter: MultimodeFilter,
    pub base_width_filter: Option<BaseWidthFilter>,
    pub amp: AmpEnvelope,
    pub insert_fx: Vec<EffectSlot>,
    pub mod_matrix: ModMatrix,
    pub lfos: [VoiceLfo; 2],
    pub mod_envelope: ModEnvelope,
    pub sequencer: TrackSequencer,
    pub muted: bool,
    pub solo: bool,
    pub volume: f32,
    pub pan: f32,
    pub send_levels: [f32; 3],
    pub routing: Routing,
    pub overdrive: f32,
    pub current_page: PageType,
    pub voices: [Option<Voice>; MAX_VOICES],
    pub active_voices: usize,
    pub arpeggiator: Option<Arpeggiator>,
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub note: u8,
    pub velocity: f32,
    pub age: u64,
    pub output: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct MultimodeFilter {
    pub filter_type: f32,
    pub frequency: f32,
    pub resonance: f32,
    pub env_depth: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub enabled: bool,
}

impl Default for MultimodeFilter {
    fn default() -> Self {
        MultimodeFilter {
            filter_type: 0.0,
            frequency: 20000.0,
            resonance: 0.0,
            env_depth: 0.0,
            attack: 0.0,
            decay: 100.0,
            sustain: 1.0,
            release: 100.0,
            enabled: true,
        }
    }
}

impl MultimodeFilter {
    pub fn new() -> Self {
        MultimodeFilter::default()
    }

    pub fn process(&mut self, output: &mut [f32], sample_rate: u32) {
        if !self.enabled {
            return;
        }

        let type_param = self.filter_type / 127.0;

        for out in output.iter_mut() {
            let input = *out;
            let lp = self.process_lp(input, sample_rate);
            let bp = self.process_bp(input, sample_rate);
            let hp = self.process_hp(input, sample_rate);

            *out = if type_param < 0.5 {
                lp * (1.0 - type_param * 2.0) + bp * (type_param * 2.0)
            } else {
                bp * (1.0 - (type_param - 0.5) * 2.0) + hp * ((type_param - 0.5) * 2.0)
            };
        }
    }

    fn process_lp(&self, input: f32, _sample_rate: u32) -> f32 {
        input
    }

    fn process_bp(&self, input: f32, _sample_rate: u32) -> f32 {
        input
    }

    fn process_hp(&self, input: f32, _sample_rate: u32) -> f32 {
        input
    }
}

#[derive(Debug, Clone)]
pub struct BaseWidthFilter {
    pub base: f32,
    pub width: f32,
    pub delay: f32,
    pub spread: f32,
    pub key_track: f32,
    pub reset: bool,
    pub enabled: bool,
}

impl Default for BaseWidthFilter {
    fn default() -> Self {
        BaseWidthFilter {
            base: 0.0,
            width: 127.0,
            delay: 0.0,
            spread: 0.0,
            key_track: 64.0,
            reset: false,
            enabled: false,
        }
    }
}

impl BaseWidthFilter {
    pub fn new() -> Self {
        BaseWidthFilter::default()
    }

    pub fn process(&self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        if !self.enabled {
            output.copy_from_slice(input);
            return;
        }

        let base_freq = 20.0 + (self.base / 127.0) * 20000.0;
        let width = 100.0 + (self.width / 127.0) * 20000.0;
        let delay_samples = (self.delay / 127.0 * 100.0) as usize;
        let spread_amount = (self.spread / 127.0) * 0.5;

        for (i, out) in output.iter_mut().enumerate() {
            let left = input[i * 2];
            let right = input[i * 2 + 1];

            let lp_cutoff = base_freq.min(sample_rate as f32 / 2.0);
            let hp_cutoff = (base_freq + width).min(sample_rate as f32 / 2.0);

            *out = left * (1.0 - spread_amount) + right * spread_amount;
        }
    }
}

#[derive(Debug, Clone)]
pub struct AmpEnvelope {
    pub attack: f32,
    pub hold: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub vel_to_vol: f32,
    pub overdrive: f32,
    pub pan: f32,
}

impl Default for AmpEnvelope {
    fn default() -> Self {
        AmpEnvelope {
            attack: 0.0,
            hold: 0.0,
            decay: 100.0,
            sustain: 1.0,
            release: 100.0,
            vel_to_vol: 64.0,
            overdrive: 0.0,
            pan: 0.0,
        }
    }
}

impl AmpEnvelope {
    pub fn new() -> Self {
        AmpEnvelope::default()
    }
}

#[derive(Debug, Clone)]
pub struct EffectSlot {
    pub id: u8,
    pub effect_type: EffectType,
    pub bypass: bool,
    pub params: [f32; 16],
    pub mix: f32,
    pub lfo: Option<FxLfo>,
}

impl Default for EffectSlot {
    fn default() -> Self {
        EffectSlot {
            id: 1,
            effect_type: EffectType::Bypass,
            bypass: true,
            params: [0.0; 16],
            mix: 1.0,
            lfo: None,
        }
    }
}

impl EffectSlot {
    pub fn new(id: u8) -> Self {
        EffectSlot {
            id,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModMatrix {
    pub connections: Vec<ModConnection>,
    pub max_connections: usize,
}

#[derive(Debug, Clone)]
pub struct ModConnection {
    pub source: ModSource,
    pub destination: ModDestination,
    pub amount: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModSource {
    None,
    Lfo1,
    Lfo2,
    ModEnv,
    Velocity,
    Note,
    Aftertouch,
    ModWheel,
    PitchWheel,
}

impl Default for ModMatrix {
    fn default() -> Self {
        ModMatrix {
            connections: Vec::new(),
            max_connections: 32,
        }
    }
}

impl ModMatrix {
    pub fn new() -> Self {
        ModMatrix::default()
    }

    pub fn add_connection(
        &mut self,
        source: ModSource,
        destination: ModDestination,
        amount: f32,
    ) -> bool {
        if self.connections.len() < self.max_connections {
            self.connections.push(ModConnection {
                source,
                destination,
                amount,
            });
            true
        } else {
            false
        }
    }

    pub fn remove_connection(&mut self, index: usize) {
        if index < self.connections.len() {
            self.connections.remove(index);
        }
    }

    pub fn get_modulation(&self, source: ModSource, destination: ModDestination) -> f32 {
        self.connections
            .iter()
            .filter(|c| c.source == source && c.destination == destination)
            .fold(0.0, |acc, c| acc + c.amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    Trig,
    Src,
    Fltr,
    Amp,
    Fx,
    Mod,
}

impl Default for Track {
    fn default() -> Self {
        Track {
            id: 1,
            track_type: TrackType::Audio,
            name: String::new(),
            machine: Machine::default(),
            multimode_filter: MultimodeFilter::new(),
            base_width_filter: None,
            amp: AmpEnvelope::new(),
            insert_fx: vec![EffectSlot::new(1), EffectSlot::new(2)],
            mod_matrix: ModMatrix::new(),
            lfos: [VoiceLfo::new(), VoiceLfo::new()],
            mod_envelope: ModEnvelope::new(),
            sequencer: TrackSequencer::new(),
            muted: false,
            solo: false,
            volume: 0.8,
            pan: 0.0,
            send_levels: [0.0; 3],
            routing: Routing::MixAB,
            overdrive: 0.0,
            current_page: PageType::Trig,
            voices: [const { None }; MAX_VOICES],
            active_voices: 0,
            arpeggiator: None,
        }
    }
}

impl Track {
    pub fn new(id: u8, track_type: TrackType, name: &str) -> Self {
        let fx_slots = match track_type {
            TrackType::Audio | TrackType::Bus => 2,
            TrackType::Send | TrackType::Mix => 1,
        };

        Track {
            id,
            track_type,
            name: name.to_string(),
            insert_fx: (1..=fx_slots).map(|i| EffectSlot::new(i as u8)).collect(),
            ..Default::default()
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        if self.muted {
            output.fill(0.0);
            return;
        }

        let mut track_output = vec![0.0; input.len()];
        self.machine.process(&mut track_output, sample_rate, 60);

        if self.overdrive > 0.0 {
            for sample in &mut track_output {
                *sample = (*sample * (1.0 + self.overdrive)).tanh();
            }
        }

        if let Some(ref mut filter) = self.base_width_filter {
            let mut filtered = vec![0.0; track_output.len()];
            filter.process(&track_output, &mut filtered, sample_rate);
            track_output = filtered;
        }

        self.multimode_filter
            .process(&mut track_output, sample_rate);

        for (i, sample) in track_output.iter().enumerate() {
            let pan_factor = if self.pan >= 0.0 {
                1.0 - self.pan * 0.5
            } else {
                1.0 + self.pan * 0.5
            };
            output[i] = *sample * self.volume * pan_factor;
        }
    }

    pub fn trigger(&mut self, note: u8, velocity: f32) {
        self.machine.trigger(note, velocity);

        let oldest_voice_index = if self.active_voices >= MAX_VOICES {
            let mut oldest_age = u64::MAX;
            let mut oldest_index = 0;
            for (i, voice) in self.voices.iter().enumerate() {
                if let Some(ref v) = voice {
                    if v.age < oldest_age {
                        oldest_age = v.age;
                        oldest_index = i;
                    }
                }
            }
            oldest_index
        } else {
            self.active_voices
        };

        self.voices[oldest_voice_index] = Some(Voice {
            note,
            velocity,
            age: 0,
            output: vec![0.0; 1024],
        });

        if self.active_voices < MAX_VOICES {
            self.active_voices += 1;
        }
    }

    pub fn set_param(&mut self, param: &str, value: f32) {
        match param {
            "mute" => self.muted = value > 0.5,
            "solo" => self.solo = value > 0.5,
            "volume" => self.volume = value / 127.0,
            "pan" => self.pan = (value - 64.0) / 64.0,
            "overdrive" => self.overdrive = value / 127.0,
            "routing" => {
                self.routing = match value as u8 {
                    0 => Routing::MixAB,
                    1 => Routing::OutCD,
                    2 => Routing::OutEF,
                    3 => Routing::Bus1,
                    4 => Routing::Bus2,
                    5 => Routing::Bus3,
                    6 => Routing::Bus4,
                    _ => Routing::MixAB,
                }
            }
            _ => self.machine.set_param(param, value),
        }
    }

    pub fn get_param(&self, param: &str) -> f32 {
        match param {
            "volume" => self.volume * 127.0,
            "pan" => (self.pan + 1.0) * 63.5,
            "overdrive" => self.overdrive * 127.0,
            _ => 0.0,
        }
    }

    pub fn fx_slot(&mut self, slot_id: u8) -> Option<&mut EffectSlot> {
        self.insert_fx.iter_mut().find(|slot| slot.id == slot_id)
    }
}
