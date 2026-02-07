use crate::{Machine, MachineType, MidiMachine, Track, TrackType, NUM_TRACKS, FX_SLOTS_AUDIO};

#[derive(Debug, Clone)]
pub struct MidiEngine {
    pub input_ports: Vec<MidiPort>,
    pub output_ports: Vec<MidiPort>,
    pub active_input: Option<usize>,
    pub active_output: Option<usize>,
    pub channels: [MidiChannel; 16],
    pub global_channel: u8,
    pub sync: MidiSync,
    pub clock_divider: u32,
    pub last_clock: u64,
}

#[derive(Debug, Clone)]
pub struct MidiPort {
    pub name: String,
    pub device_id: u32,
    pub manufacturer: String,
}

#[derive(Debug, Clone)]
pub struct MidiChannel {
    pub track: Option<usize>,
    pub program: u8,
    pub bank_msb: u8,
    pub bank_lsb: u8,
    pub volume: u8,
    pub pan: u8,
    pub pitch_bend: i16,
    pub modulation: u8,
    pub aftertouch: u8,
    pub cc_values: [u8; 128],
    pub notes_on: [u8; 128],
}

#[derive(Debug, Clone)]
pub struct MidiSync {
    pub clock_enabled: bool,
    pub start_enabled: bool,
    pub stop_enabled: bool,
    pub continue_enabled: bool,
    pub song_position: u16,
    pub tempo: u16,
    pub running: bool,
}

impl Default for MidiEngine {
    fn default() -> Self {
        MidiEngine {
            input_ports: Vec::new(),
            output_ports: Vec::new(),
            active_input: None,
            active_output: None,
            channels: [MidiChannel::default(); 16],
            global_channel: 0,
            sync: MidiSync::default(),
            clock_divider: 0,
            last_clock: 0,
        }
    }
}

impl MidiEngine {
    pub fn new() -> Self {
        MidiEngine::default()
    }

    pub fn process_midi_message(&mut self, message: MidiMessage) {
        match message {
            MidiMessage::NoteOn { channel, note, velocity } => {
                if velocity > 0 {
                    self.channels[channel as usize].notes_on[note as usize] = velocity;
                    self.handle_note_on(channel, note, velocity);
                } else {
                    self.channels[channel as usize].notes_on[note as usize] = 0;
                    self.handle_note_off(channel, note);
                }
            }
            MidiMessage::NoteOff { channel, note } => {
                self.channels[channel as usize].notes_on[note as usize] = 0;
                self.handle_note_off(channel, note);
            }
            MidiMessage::ControlChange { channel, cc, value } => {
                self.channels[channel as usize].cc_values[cc as usize] = value;
                self.handle_cc(channel, cc, value);
            }
            MidiMessage::PitchBend { channel, value } => {
                self.channels[channel as usize].pitch_bend = value;
            }
            MidiMessage::ProgramChange { channel, program } => {
                self.channels[channel as usize].program = program;
            }
            MidiMessage::Aftertouch { channel, note: _, pressure } => {
                self.channels[channel as usize].aftertouch = pressure;
            }
            MidiMessage::Clock => {
                self.clock_divider = (self.clock_divider + 1) % 24;
            }
            MidiMessage::Start => {
                self.sync.running = true;
            }
            MidiMessage::Stop => {
                self.sync.running = false;
            }
            MidiMessage::Continue => {
                self.sync.running = true;
            }
            MidiMessage::SongPosition { position } => {
                self.sync.song_position = position;
            }
            _ => {}
        }
    }

    fn handle_note_on(&self, channel: u8, note: u8, velocity: u8) {
        let track_index = self.get_track_for_channel(channel);
        if let Some(track) = self.get_track_mut(track_index) {
            if let Machine::Midi(ref mut midi_machine) = track.machine {
                midi_machine.set_cc(7, velocity as f32);
            }
        }
    }

    fn handle_note_off(&self, _channel: u8, _note: u8) {}

    fn handle_cc(&self, channel: u8, cc: u8, value: u8) {
        let track_index = self.get_track_for_channel(channel);
        if let Some(track) = self.get_track_mut(track_index) {
            if let Machine::Midi(ref mut midi_machine) = track.machine {
                midi_machine.set_cc(cc, value as f32);
            }
        }
    }

    fn get_track_for_channel(&self, channel: u8) -> Option<usize> {
        for (i, ch) in self.channels.iter().enumerate() {
            if ch.track == Some(channel as usize) {
                return Some(channel as usize);
            }
        }
        None
    }

    fn get_track_mut(&mut self, index: Option<usize>) -> Option<&mut Track> {
        if let Some(i) = index {
            Some(&mut unsafe { &mut *(&mut self as *mut MidiEngine as *mut [Track; 16] }[i] })
        } else {
            None
        }
    }

    pub fn set_track_channel(&mut self, track: usize, channel: u8) {
        if channel > 0 && channel <= 16 {
            self.channels[channel as usize - 1].track = Some(track);
        }
    }

    pub fn send_midi_message(&self, _message: MidiMessage) {}
}

impl Default for MidiChannel {
    fn default() -> Self {
        MidiChannel {
            track: None,
            program: 0,
            bank_msb: 0,
            bank_lsb: 0,
            volume: 100,
            pan: 64,
            pitch_bend: 8192,
            modulation: 0,
            aftertouch: 0,
            cc_values: [0; 128],
            notes_on: [0; 128],
        }
    }
}

impl Default for MidiSync {
    fn default() -> Self {
        MidiSync {
            clock_enabled: true,
            start_enabled: true,
            stop_enabled: true,
            continue_enabled: true,
            song_position: 0,
            tempo: 120,
            running: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MidiMessage {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    ControlChange { channel: u8, cc: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    PitchBend { channel: u8, value: i16 },
    Aftertouch { channel: u8, note: u8, pressure: u8 },
    ChannelPressure { channel: u8, pressure: u8 },
    Clock,
    Start,
    Stop,
    Continue,
    SongPosition { position: u16 },
    SystemExclusive { data: Vec<u8> },
}

pub const CC_VOLUME: u8 = 7;
pub const CC_PAN: u8 = 10;
pub const CC_MODULATION: u8 = 1;
pub const CC_SUSTAIN: u8 = 64;
pub const CC_REVERB: u8 = 91;
pub const CC_CHORUS: u8 = 93;
pub const CC_ALL_NOTES_OFF: u8 = 123;
