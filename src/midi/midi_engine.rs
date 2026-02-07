use crate::Machine;

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

    pub fn process_midi_message(&mut self, message: MidiMessage) -> Option<MidiEvent> {
        match message {
            MidiMessage::NoteOn {
                channel,
                note,
                velocity,
            } => {
                if velocity > 0 {
                    self.channels[channel as usize].notes_on[note as usize] = velocity;
                    Some(MidiEvent::NoteOn {
                        channel,
                        note,
                        velocity,
                    })
                } else {
                    self.channels[channel as usize].notes_on[note as usize] = 0;
                    Some(MidiEvent::NoteOff { channel, note })
                }
            }
            MidiMessage::NoteOff { channel, note } => {
                self.channels[channel as usize].notes_on[note as usize] = 0;
                Some(MidiEvent::NoteOff { channel, note })
            }
            MidiMessage::ControlChange { channel, cc, value } => {
                self.channels[channel as usize].cc_values[cc as usize] = value;
                Some(MidiEvent::ControlChange { channel, cc, value })
            }
            MidiMessage::PitchBend { channel, value } => {
                self.channels[channel as usize].pitch_bend = value;
                Some(MidiEvent::PitchBend { channel, value })
            }
            MidiMessage::ProgramChange { channel, program } => {
                self.channels[channel as usize].program = program;
                Some(MidiEvent::ProgramChange { channel, program })
            }
            MidiMessage::Aftertouch {
                channel,
                note: _,
                pressure,
            } => {
                self.channels[channel as usize].aftertouch = pressure;
                Some(MidiEvent::Aftertouch { channel, pressure })
            }
            MidiMessage::Clock => {
                self.clock_divider = (self.clock_divider + 1) % 24;
                None
            }
            MidiMessage::Start => {
                self.sync.running = true;
                Some(MidiEvent::Start)
            }
            MidiMessage::Stop => {
                self.sync.running = false;
                Some(MidiEvent::Stop)
            }
            MidiMessage::Continue => {
                self.sync.running = true;
                Some(MidiEvent::Continue)
            }
            MidiMessage::SongPosition { position } => {
                self.sync.song_position = position;
                Some(MidiEvent::SongPosition { position })
            }
            _ => None,
        }
    }

    pub fn handle_cc(&mut self, channel: u8, cc: u8, value: u8) {
        self.channels[channel as usize].cc_values[cc as usize] = value;
    }

    pub fn get_cc(&self, channel: u8, cc: u8) -> u8 {
        self.channels[channel as usize].cc_values[cc as usize]
    }

    pub fn get_track_for_channel(&self, channel: u8) -> Option<usize> {
        for (i, ch) in self.channels.iter().enumerate() {
            if ch.track == Some(channel as usize) {
                return Some(channel as usize);
            }
        }
        None
    }

    pub fn set_track_channel(&mut self, track: usize, channel: u8) {
        if channel > 0 && channel <= 16 {
            self.channels[channel as usize - 1].track = Some(track);
        }
    }

    pub fn get_channel_volume(&self, channel: u8) -> u8 {
        self.channels[channel as usize].volume
    }

    pub fn get_channel_pan(&self, channel: u8) -> u8 {
        self.channels[channel as usize].pan
    }

    pub fn get_channel_pitch_bend(&self, channel: u8) -> i16 {
        self.channels[channel as usize].pitch_bend
    }

    pub fn get_channel_aftertouch(&self, channel: u8) -> u8 {
        self.channels[channel as usize].aftertouch
    }

    pub fn is_note_on(&self, channel: u8, note: u8) -> bool {
        self.channels[channel as usize].notes_on[note as usize] > 0
    }

    pub fn set_tempo(&mut self, tempo: u16) {
        self.sync.tempo = tempo.clamp(20, 300);
    }

    pub fn tempo(&self) -> u16 {
        self.sync.tempo
    }

    pub fn is_running(&self) -> bool {
        self.sync.running
    }

    pub fn clock_division(&self) -> u32 {
        self.clock_divider
    }

    pub fn song_position(&self) -> u16 {
        self.sync.song_position
    }

    pub fn send_midi_message(&self, _message: MidiMessage) {}

    pub fn add_input_port(&mut self, port: MidiPort) {
        self.input_ports.push(port);
    }

    pub fn add_output_port(&mut self, port: MidiPort) {
        self.output_ports.push(port);
    }

    pub fn set_active_input(&mut self, index: Option<usize>) {
        self.active_input = index;
    }

    pub fn set_active_output(&mut self, index: Option<usize>) {
        self.active_output = index;
    }
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
pub enum MidiEvent {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    ControlChange { channel: u8, cc: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    PitchBend { channel: u8, value: i16 },
    Aftertouch { channel: u8, pressure: u8 },
    Clock,
    Start,
    Stop,
    Continue,
    SongPosition { position: u16 },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_engine_creation() {
        let engine = MidiEngine::new();
        assert_eq!(engine.sync.tempo, 120);
        assert!(!engine.sync.running);
    }

    #[test]
    fn test_midi_note_on_event() {
        let mut engine = MidiEngine::new();
        let event = engine.process_midi_message(MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100,
        });
        assert!(matches!(
            event,
            Some(MidiEvent::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100
            })
        ));
        assert!(engine.is_note_on(0, 60));
    }

    #[test]
    fn test_midi_note_off_event() {
        let mut engine = MidiEngine::new();
        engine.process_midi_message(MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 100,
        });
        let event = engine.process_midi_message(MidiMessage::NoteOff {
            channel: 0,
            note: 60,
        });
        assert!(matches!(
            event,
            Some(MidiEvent::NoteOff {
                channel: 0,
                note: 60
            })
        ));
        assert!(!engine.is_note_on(0, 60));
    }

    #[test]
    fn test_midi_cc_event() {
        let mut engine = MidiEngine::new();
        let event = engine.process_midi_message(MidiMessage::ControlChange {
            channel: 0,
            cc: 7,
            value: 127,
        });
        assert!(matches!(
            event,
            Some(MidiEvent::ControlChange {
                channel: 0,
                cc: 7,
                value: 127
            })
        ));
        assert_eq!(engine.get_cc(0, 7), 127);
    }

    #[test]
    fn test_midi_clock() {
        let mut engine = MidiEngine::new();
        for _ in 0..24 {
            engine.process_midi_message(MidiMessage::Clock);
        }
        assert_eq!(engine.clock_division(), 0);
    }

    #[test]
    fn test_midi_start_stop() {
        let mut engine = MidiEngine::new();
        assert!(!engine.is_running());
        engine.process_midi_message(MidiMessage::Start);
        assert!(engine.is_running());
        engine.process_midi_message(MidiMessage::Stop);
        assert!(!engine.is_running());
    }

    #[test]
    fn test_midi_tempo() {
        let mut engine = MidiEngine::new();
        engine.set_tempo(140);
        assert_eq!(engine.tempo(), 140);
        engine.set_tempo(400); // should clamp
        assert_eq!(engine.tempo(), 300);
    }

    #[test]
    fn test_track_channel_assignment() {
        let mut engine = MidiEngine::new();
        engine.set_track_channel(5, 3);
        assert_eq!(engine.get_track_for_channel(3), Some(3));
        assert_eq!(engine.get_track_for_channel(1), None);
    }

    #[test]
    fn test_velocity_zero_is_note_off() {
        let mut engine = MidiEngine::new();
        let event = engine.process_midi_message(MidiMessage::NoteOn {
            channel: 0,
            note: 60,
            velocity: 0,
        });
        assert!(matches!(
            event,
            Some(MidiEvent::NoteOff {
                channel: 0,
                note: 60
            })
        ));
        assert!(!engine.is_note_on(0, 60));
    }

    #[test]
    fn test_channel_volume() {
        let mut engine = MidiEngine::new();
        assert_eq!(engine.get_channel_volume(0), 100);
        engine.handle_cc(0, CC_VOLUME, 80);
        assert_eq!(engine.get_channel_volume(0), 80);
    }

    #[test]
    fn test_channel_pan() {
        let mut engine = MidiEngine::new();
        assert_eq!(engine.get_channel_pan(0), 64);
        engine.handle_cc(0, CC_PAN, 32);
        assert_eq!(engine.get_channel_pan(0), 32);
    }

    #[test]
    fn test_pitch_bend() {
        let mut engine = MidiEngine::new();
        assert_eq!(engine.get_channel_pitch_bend(0), 8192);
        engine.process_midi_message(MidiMessage::PitchBend {
            channel: 0,
            value: 16383,
        });
        assert_eq!(engine.get_channel_pitch_bend(0), 16383);
    }
}
