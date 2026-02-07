use super::*;

#[derive(Debug, Clone)]
pub struct SinglePlayer {
    pub sample: Option<SampleHandle>,
    pub tune: f32,
    pub play_mode: PlayMode,
    pub loop_crossfade: f32,
    pub sample_slot: u16,
    pub strt: u32,
    pub end: u32,
    pub lstr: u32,
    pub lend: u32,
    pub velocity: f32,
    pub playback_position: f32,
}

#[derive(Debug, Clone)]
pub struct SampleHandle {
    pub id: u32,
    pub name: String,
    pub data: Vec<f32>,
    pub sample_rate: u32,
    pub num_channels: u32,
    pub num_frames: u32,
    pub loop_mode: PlayMode,
    pub loop_start: u32,
    pub loop_end: u32,
    pub playback_position: f32,
}

impl Default for SinglePlayer {
    fn default() -> Self {
        SinglePlayer {
            sample: None,
            tune: 64.0,
            play_mode: PlayMode::Forward,
            loop_crossfade: 0.0,
            sample_slot: 0,
            strt: 0,
            end: 0,
            lstr: 0,
            lend: 0,
            velocity: 1.0,
            playback_position: 0.0,
        }
    }
}

impl SinglePlayer {
    pub fn new() -> Self {
        SinglePlayer::default()
    }

    pub fn set_sample(&mut self, sample: SampleHandle) {
        self.sample = Some(sample);
        self.end = self
            .sample
            .as_ref()
            .map(|s| s.num_frames as u32)
            .unwrap_or(0);
        self.lstr = 0;
        self.lend = self.end;
    }

    pub fn process(&mut self, output: &mut [f32], sample_rate: u32) {
        let Some(ref sample) = self.sample else {
            output.fill(0.0);
            return;
        };

        let pitch_factor = 2.0_f32.powf((self.tune - 64.0) / 12.0);
        let step = pitch_factor * sample.sample_rate as f32 / sample_rate as f32;

        match self.play_mode {
            PlayMode::Forward | PlayMode::ForwardLoop => {
                self.playback_position += step;
                if self.play_mode == PlayMode::ForwardLoop {
                    let loop_end = if self.lend > 0 {
                        self.lend
                    } else {
                        sample.num_frames as u32
                    };
                    let loop_start = self.lstr;

                    for (i, out) in output.iter_mut().enumerate() {
                        let idx =
                            self.playback_position as usize + i * sample.num_channels as usize;
                        let loop_end_usize = loop_end as usize;
                        let loop_start_usize = loop_start as usize;

                        if idx >= loop_end_usize {
                            let crossfade_len =
                                (self.loop_crossfade * 0.01 * (loop_end - loop_start) as f32)
                                    as usize;
                            let fade_out = if idx < loop_end_usize + crossfade_len {
                                1.0 - (idx - loop_end_usize) as f32 / crossfade_len as f32
                            } else {
                                0.0
                            };
                            let fade_in = if idx >= loop_start_usize
                                && idx < loop_start_usize + crossfade_len
                            {
                                (idx - loop_start_usize) as f32 / crossfade_len as f32
                            } else {
                                1.0
                            };

                            let loop_idx = loop_start_usize
                                + ((idx - loop_start_usize) % (loop_end_usize - loop_start_usize));
                            *out = if loop_idx < sample.data.len() {
                                sample.data[loop_idx] * fade_out * fade_in
                            } else {
                                0.0
                            };
                        } else if idx < sample.data.len() {
                            *out = sample.data[idx];
                        } else {
                            *out = 0.0;
                        }
                    }
                    self.playback_position = loop_start as f32
                        + (self.playback_position - loop_end as f32)
                            % (loop_end - loop_start) as f32;
                } else {
                    for (i, out) in output.iter_mut().enumerate() {
                        let idx =
                            self.playback_position as usize + i * sample.num_channels as usize;
                        *out = if idx < sample.data.len() {
                            sample.data[idx] * self.velocity
                        } else {
                            0.0
                        };
                    }
                    if self.playback_position >= sample.num_frames as f32 {
                        output.fill(0.0);
                    }
                }
            }
            PlayMode::Reverse | PlayMode::ReverseLoop => {
                self.playback_position -= step;
                if self.playback_position < 0.0 {
                    if self.play_mode == PlayMode::ReverseLoop {
                        self.playback_position = sample.num_frames as f32 + self.playback_position;
                    } else {
                        self.playback_position = 0.0;
                        output.fill(0.0);
                    }
                }
                for (i, out) in output.iter_mut().enumerate() {
                    let idx = self.playback_position as usize + i * sample.num_channels as usize;
                    *out = if idx < sample.data.len() {
                        sample.data[idx] * self.velocity
                    } else {
                        0.0
                    };
                }
            }
        }
    }

    pub fn trigger(&mut self, note: u8, velocity: f32) {
        self.playback_position = self.strt as f32;
        self.velocity = velocity;
    }

    pub fn stop(&mut self) {
        self.playback_position = self.strt as f32;
    }
}

#[derive(Debug, Clone)]
pub struct MultiPlayer {
    pub keyzones: Vec<KeyZone>,
    pub tune: f32,
    pub vibrato: f32,
    pub speed: f32,
    pub fade: f32,
    pub velocity: f32,
}

#[derive(Debug, Clone)]
pub struct KeyZone {
    pub note_low: u8,
    pub note_high: u8,
    pub sample: SampleHandle,
    pub tune_offset: f32,
}

impl Default for MultiPlayer {
    fn default() -> Self {
        MultiPlayer {
            keyzones: Vec::new(),
            tune: 64.0,
            vibrato: 0.0,
            speed: 64.0,
            fade: 0.0,
            velocity: 1.0,
        }
    }
}

impl MultiPlayer {
    pub fn new() -> Self {
        MultiPlayer::default()
    }

    pub fn add_keyzone(&mut self, zone: KeyZone) {
        self.keyzones.push(zone);
    }

    pub fn process(&mut self, output: &mut [f32], sample_rate: u32, note: u8) {
        let zone_opt = self
            .keyzones
            .iter_mut()
            .find(|z| note >= z.note_low && note <= z.note_high);

        let Some(zone) = zone_opt else {
            output.fill(0.0);
            return;
        };

        let pitch_factor = 2.0_f32.powf((self.tune - 64.0 + zone.tune_offset) / 12.0);
        let step = pitch_factor * zone.sample.sample_rate as f32 / sample_rate as f32;

        for (i, out) in output.iter_mut().enumerate() {
            let idx =
                zone.sample.playback_position as usize + i * zone.sample.num_channels as usize;
            zone.sample.playback_position += step;

            if idx < zone.sample.data.len() {
                *out = zone.sample.data[idx] * self.velocity;
            } else {
                *out = 0.0;
            }
        }
    }

    pub fn trigger(&mut self, note: u8, velocity: f32) {
        self.velocity = velocity;
        for zone in &mut self.keyzones {
            zone.sample.playback_position = 0.0;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubtrackMachine {
    pub subtracks: [Option<Subtrack>; 8],
    pub supertrack_fx: Vec<EffectSlot>,
}

#[derive(Debug, Clone)]
pub struct Subtrack {
    pub enabled: bool,
    pub machine: Box<Machine>,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
}

impl Default for SubtrackMachine {
    fn default() -> Self {
        SubtrackMachine {
            subtracks: [None, None, None, None, None, None, None, None],
            supertrack_fx: Vec::new(),
        }
    }
}

impl SubtrackMachine {
    pub fn new() -> Self {
        SubtrackMachine::default()
    }

    pub fn process(&mut self, output: &mut [f32], sample_rate: u32, note: u8, active_mask: u8) {
        let mut mixed = vec![0.0; output.len()];

        for (i, subtrack_opt) in self.subtracks.iter_mut().enumerate() {
            if let Some(subtrack) = subtrack_opt {
                if subtrack.enabled && !subtrack.muted {
                    let mask = 1 << i;
                    if active_mask & mask != 0 {
                        let mut sub_output = vec![0.0; output.len()];
                        match &mut *subtrack.machine {
                            Machine::Single(single) => {
                                single.process(&mut sub_output, sample_rate);
                            }
                            Machine::Multi(multi) => {
                                multi.process(&mut sub_output, sample_rate, note);
                            }
                            _ => {}
                        }
                        for (j, sample) in sub_output.iter().enumerate() {
                            mixed[j] += sample
                                * subtrack.volume
                                * if subtrack.pan >= 0.0 {
                                    1.0 - subtrack.pan
                                } else {
                                    1.0 + subtrack.pan
                                };
                        }
                    }
                }
            }
        }

        for (i, sample) in mixed.iter().enumerate() {
            output[i] = *sample;
        }
    }

    pub fn trigger(&mut self, note: u8, velocity: f32, active_mask: u8) {
        for (i, subtrack_opt) in self.subtracks.iter_mut().enumerate() {
            let mask = 1 << i;
            if active_mask & mask != 0 {
                if let Some(subtrack) = subtrack_opt {
                    match &mut *subtrack.machine {
                        Machine::Single(single) => single.trigger(note, velocity),
                        Machine::Multi(multi) => multi.trigger(note, velocity),
                        _ => {}
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MidiMachine {
    pub channel: u8,
    pub bank: u8,
    pub sub_bank: u8,
    pub program: u8,
    pub pitch_bend_range: u8,
    pub aftertouch_enabled: bool,
    pub mod_wheel_enabled: bool,
    pub breath_control_enabled: bool,
    pub cc_assignments: [CcAssignment; 16],
}

#[derive(Debug, Clone)]
pub struct CcAssignment {
    pub cc_number: u8,
    pub parameter: String,
    pub min: f32,
    pub max: f32,
    pub current_value: f32,
}

impl Default for CcAssignment {
    fn default() -> Self {
        CcAssignment {
            cc_number: 0,
            parameter: String::new(),
            min: 0.0,
            max: 1.0,
            current_value: 0.0,
        }
    }
}

impl Default for MidiMachine {
    fn default() -> Self {
        let mut assignments = core::array::from_fn(|i| CcAssignment {
            cc_number: i as u8,
            parameter: String::new(),
            min: 0.0,
            max: 1.0,
            current_value: 0.0,
        });
        MidiMachine {
            channel: 1,
            bank: 0,
            sub_bank: 0,
            program: 0,
            pitch_bend_range: 2,
            aftertouch_enabled: true,
            mod_wheel_enabled: true,
            breath_control_enabled: false,
            cc_assignments: assignments,
        }
    }
}

impl MidiMachine {
    pub fn new() -> Self {
        MidiMachine::default()
    }

    pub fn set_cc(&mut self, cc_number: u8, value: f32) {
        if let Some(assignment) = self.cc_assignments.get_mut(cc_number as usize) {
            assignment.current_value = value;
        }
    }

    pub fn get_param(&self, param: &str) -> f32 {
        match param {
            "volume" => self.get_cc_value(7),
            "pan" => self.get_cc_value(10),
            "modwheel" => self.get_cc_value(1),
            "pitchbend" => 0.5,
            _ => 0.0,
        }
    }

    fn get_cc_value(&self, cc_number: u8) -> f32 {
        self.cc_assignments[cc_number as usize].current_value
    }
}

#[derive(Debug, Clone)]
pub enum Machine {
    Single(SinglePlayer),
    Multi(MultiPlayer),
    Subtrack(Box<SubtrackMachine>),
    Midi(MidiMachine),
}

impl Default for Machine {
    fn default() -> Self {
        Machine::Single(SinglePlayer::new())
    }
}

impl Machine {
    pub fn process(&mut self, output: &mut [f32], sample_rate: u32, note: u8) {
        match self {
            Machine::Single(single) => single.process(output, sample_rate),
            Machine::Multi(multi) => multi.process(output, sample_rate, note),
            Machine::Subtrack(subtrack) => subtrack.process(output, sample_rate, note, 0xFF),
            Machine::Midi(_midi) => {
                output.fill(0.0);
            }
        }
    }

    pub fn trigger(&mut self, note: u8, velocity: f32) {
        match self {
            Machine::Single(single) => single.trigger(note, velocity),
            Machine::Multi(multi) => multi.trigger(note, velocity),
            Machine::Subtrack(subtrack) => subtrack.trigger(note, velocity, 0xFF),
            Machine::Midi(_midi) => {}
        }
    }

    pub fn set_param(&mut self, param: &str, value: f32) {
        match self {
            Machine::Single(single) => match param {
                "tune" => single.tune = value,
                "playmode" => {
                    single.play_mode = match value as u8 {
                        0 => PlayMode::Forward,
                        1 => PlayMode::Reverse,
                        2 => PlayMode::ForwardLoop,
                        3 => PlayMode::ReverseLoop,
                        _ => PlayMode::Forward,
                    }
                }
                "loopcrossfade" => single.loop_crossfade = value,
                "samplestart" => single.strt = value as u32,
                "sampleend" => single.end = value as u32,
                "loopstart" => single.lstr = value as u32,
                "loopend" => single.lend = value as u32,
                _ => {}
            },
            Machine::Multi(multi) => match param {
                "tune" => multi.tune = value,
                "vibrato" => multi.vibrato = value,
                "speed" => multi.speed = value,
                "fade" => multi.fade = value,
                _ => {}
            },
            Machine::Midi(midi) => match param {
                "channel" => midi.channel = value as u8,
                "bank" => midi.bank = value as u8,
                "program" => midi.program = value as u8,
                _ => {}
            },
            _ => {}
        }
    }
}
