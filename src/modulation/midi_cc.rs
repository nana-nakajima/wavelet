//! MIDI CC Mapping Module
//!
//! This module provides comprehensive MIDI CC (Control Change) mapping functionality
//! for the WAVELET synthesizer, implementing 16 assignable CCs plus standard MIDI controls
//! as specified in the Tonverk specification.
//!
//! # Features
//!
//! - 16 assignable MIDI CCs (CC0-CC15)
//! - Standard MIDI CC support (Mod Wheel, Pitch Bend, etc.)
//! - Program Change (PC) support
//! - Bank Change (MSB/LSB) support
//! - CC learn functionality
//! - Real-time CC value monitoring
//!
//! # MIDI CC Specification (Tonverk Compatible)
//!
//! | CC# | Name | Default Function |
//! |-----|------|------------------|
//! | CC0 | Modulation | Modulation depth |
//! | CC1 | Modulation | Modulation depth |
//! | CC7 | Volume | Channel volume |
//! | CC10 | Pan | Stereo pan |
//! | CC11 | Expression | Expression pedal |
//! | CC64 | Sustain | Sustain pedal |
//! | CC65 | Portamento | Portamento on/off |
//! | CC74 | Filter Cutoff | Filter cutoff frequency |
//! | CC71 | Resonance | Filter resonance |
//! | CC91 | Reverb Send | Reverb amount |
//! | CC93 | Chorus Send | Chorus amount |
//! | CC120 | All Sound Off | Emergency mute |
//! | CC121 | Reset All Controllers | Reset to defaults |
//! | CC123 | Note Off | All notes off |

use std::collections::HashMap;

/// Maximum number of assignable CCs
pub const MAX_CC_COUNT: usize = 16;

/// CC learn timeout in milliseconds
pub const CC_LEARN_TIMEOUT_MS: u64 = 5000;

/// Standard MIDI CC numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardCC {
    /// CC0: Modulation (MSB)
    Modulation = 0,
    /// CC1: Modulation
    ModulationAlt = 1,
    /// CC2: Breath Controller
    Breath = 2,
    /// CC3: Undefined
    Undefined3 = 3,
    /// CC4: Foot Controller
    Foot = 4,
    /// CC5: Portamento Time
    PortamentoTime = 5,
    /// CC6: Data Entry MSB
    DataEntryMSB = 6,
    /// CC7: Channel Volume
    Volume = 7,
    /// CC8: Balance
    Balance = 8,
    /// CC9: Undefined
    Undefined9 = 9,
    /// CC10: Pan
    Pan = 10,
    /// CC11: Expression
    Expression = 11,
    /// CC12: Effect Control 1
    EffectControl1 = 12,
    /// CC13: Effect Control 2
    EffectControl2 = 13,
    /// CC14: Undefined
    Undefined14 = 14,
    /// CC15: Undefined
    Undefined15 = 15,
    /// CC16-31: General Purpose Controllers 1-16
    GeneralPurpose1 = 16,
    GeneralPurpose2 = 17,
    GeneralPurpose3 = 18,
    GeneralPurpose4 = 19,
    /// CC32-63: LSB for CC0-31
    ModulationLSB = 32,
    BreathLSB = 33,
    FootLSB = 34,
    PortamentoTimeLSB = 37,
    VolumeLSB = 39,
    BalanceLSB = 40,
    PanLSB = 42,
    ExpressionLSB = 43,
    GeneralPurpose1LSB = 48,
    GeneralPurpose2LSB = 49,
    GeneralPurpose3LSB = 50,
    GeneralPurpose4LSB = 51,
    /// CC64: Sustain Pedal
    Sustain = 64,
    /// CC65: Portamento On/Off
    Portamento = 65,
    /// CC66: Sostenuto
    Sostenuto = 66,
    /// CC67: Soft Pedal
    SoftPedal = 67,
    /// CC68: Legato Footswitch
    Legato = 68,
    /// CC69: Hold 2
    Hold2 = 69,
    /// CC70: Sound Variation
    SoundVariation = 70,
    /// CC71: Resonance (Filter)
    Resonance = 71,
    /// CC72: Release Time
    ReleaseTime = 72,
    /// CC73: Attack Time
    AttackTime = 73,
    /// CC74: Cutoff (Filter)
    FilterCutoff = 74,
    /// CC75: Decay Time
    DecayTime = 75,
    /// CC76: Vibrato Rate
    VibratoRate = 76,
    /// CC77: Vibrato Depth
    VibratoDepth = 77,
    /// CC78: Vibrato Delay
    VibratoDelay = 78,
    /// CC79: Undefined
    Undefined79 = 79,
    /// CC80-83: General Purpose 5-8
    GeneralPurpose5 = 80,
    GeneralPurpose6 = 81,
    GeneralPurpose7 = 82,
    GeneralPurpose8 = 83,
    /// CC84: Portamento Control
    PortamentoControl = 84,
    /// CC91: Reverb Send
    ReverbSend = 91,
    /// CC92: Tremolo Depth
    TremoloDepth = 92,
    /// CC93: Chorus Send
    ChorusSend = 93,
    /// CC94: Detune Depth
    DetuneDepth = 94,
    /// CC95: Phaser Depth
    PhaserDepth = 95,
    /// CC96: Data Increment
    DataIncrement = 96,
    /// CC97: Data Decrement
    DataDecrement = 97,
    /// CC98: NRPN LSB
    NRPNLSB = 98,
    /// CC99: NRPN MSB
    NRPNMSB = 99,
    /// CC100: RPN LSB
    RPNLSB = 100,
    /// CC101: RPN MSB
    RPNMSB = 101,
    /// CC120: All Sound Off
    AllSoundOff = 120,
    /// CC121: Reset All Controllers
    ResetAllControllers = 121,
    /// CC122: Local Control
    LocalControl = 122,
    /// CC123: All Notes Off
    AllNotesOff = 123,
    /// CC124: Omni Mode Off
    OmniModeOff = 124,
    /// CC125: Omni Mode On
    OmniModeOn = 125,
    /// CC126: Mono Operation
    MonoOperation = 126,
    /// CC127: Poly Operation
    PolyOperation = 127,
}

impl StandardCC {
    /// Get CC number from enum
    pub fn cc_number(&self) -> u8 {
        *self as u8
    }

    /// Check if CC is a pedal/switch type
    pub fn is_switch(&self) -> bool {
        matches!(
            self,
            StandardCC::Sustain
                | StandardCC::Portamento
                | StandardCC::Sostenuto
                | StandardCC::SoftPedal
                | StandardCC::Legato
                | StandardCC::Hold2
        )
    }

    /// Get default name for CC
    pub fn default_name(&self) -> &'static str {
        match self {
            StandardCC::Modulation => "Modulation",
            StandardCC::ModulationAlt => "Modulation",
            StandardCC::Breath => "Breath",
            StandardCC::Foot => "Foot Controller",
            StandardCC::PortamentoTime => "Portamento Time",
            StandardCC::DataEntryMSB => "Data Entry",
            StandardCC::Volume => "Volume",
            StandardCC::Balance => "Balance",
            StandardCC::Pan => "Pan",
            StandardCC::Expression => "Expression",
            StandardCC::EffectControl1 => "Effect 1",
            StandardCC::EffectControl2 => "Effect 2",
            StandardCC::GeneralPurpose1 => "General 1",
            StandardCC::GeneralPurpose2 => "General 2",
            StandardCC::GeneralPurpose3 => "General 3",
            StandardCC::GeneralPurpose4 => "General 4",
            StandardCC::Sustain => "Sustain Pedal",
            StandardCC::Portamento => "Portamento",
            StandardCC::Sostenuto => "Sostenuto",
            StandardCC::SoftPedal => "Soft Pedal",
            StandardCC::Legato => "Legato",
            StandardCC::Hold2 => "Hold 2",
            StandardCC::SoundVariation => "Sound Variation",
            StandardCC::Resonance => "Resonance",
            StandardCC::ReleaseTime => "Release Time",
            StandardCC::AttackTime => "Attack Time",
            StandardCC::FilterCutoff => "Filter Cutoff",
            StandardCC::DecayTime => "Decay Time",
            StandardCC::VibratoRate => "Vibrato Rate",
            StandardCC::VibratoDepth => "Vibrato Depth",
            StandardCC::VibratoDelay => "Vibrato Delay",
            StandardCC::GeneralPurpose5 => "General 5",
            StandardCC::GeneralPurpose6 => "General 6",
            StandardCC::GeneralPurpose7 => "General 7",
            StandardCC::GeneralPurpose8 => "General 8",
            StandardCC::PortamentoControl => "Portamento Control",
            StandardCC::ReverbSend => "Reverb Send",
            StandardCC::TremoloDepth => "Tremolo Depth",
            StandardCC::ChorusSend => "Chorus Send",
            StandardCC::DetuneDepth => "Detune Depth",
            StandardCC::PhaserDepth => "Phaser Depth",
            StandardCC::AllSoundOff => "All Sound Off",
            StandardCC::ResetAllControllers => "Reset All",
            StandardCC::LocalControl => "Local Control",
            StandardCC::AllNotesOff => "All Notes Off",
            StandardCC::OmniModeOff => "Omni Off",
            StandardCC::OmniModeOn => "Omni On",
            StandardCC::MonoOperation => "Mono Mode",
            StandardCC::PolyOperation => "Poly Mode",
            _ => "Undefined",
        }
    }
}

/// Assignable CC mapping
#[derive(Debug, Clone, PartialEq)]
pub struct AssignableCC {
    /// CC number (0-127)
    pub cc_number: u8,
    /// User-assigned name
    pub name: String,
    /// Target parameter ID
    pub target: CCParameterTarget,
    /// Minimum input range (0.0 - 1.0)
    pub min_input: f64,
    /// Maximum input range (0.0 - 1.0)
    pub max_input: f64,
    /// Polarity: true = bipolar (-1 to 1), false = unipolar (0 to 1)
    pub bipolar: bool,
    /// Enable/disable this CC mapping
    pub enabled: bool,
}

impl Default for AssignableCC {
    fn default() -> Self {
        Self {
            cc_number: 0,
            name: String::from("CC00"),
            target: CCParameterTarget::None,
            min_input: 0.0,
            max_input: 1.0,
            bipolar: false,
            enabled: false,
        }
    }
}

impl AssignableCC {
    /// Create new CC mapping
    pub fn new(cc_number: u8, name: &str, target: CCParameterTarget) -> Self {
        Self {
            cc_number,
            name: String::from(name),
            target,
            min_input: 0.0,
            max_input: 1.0,
            bipolar: false,
            enabled: true,
        }
    }

    /// Set input range
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min_input = min;
        self.max_input = max;
        self
    }

    /// Set bipolar mode
    pub fn with_bipolar(mut self, bipolar: bool) -> Self {
        self.bipolar = bipolar;
        self
    }
}

/// CC parameter targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum CCParameterTarget {
    /// No target
    None = 0,
    /// Master volume
    MasterVolume = 1,
    /// Track volume (0-7)
    TrackVolume(u8) = 2,
    /// Track pan (0-7)
    TrackPan(u8) = 3,
    /// Filter cutoff
    FilterCutoff = 4,
    /// Filter resonance
    FilterResonance = 5,
    /// Filter drive
    FilterDrive = 6,
    /// Oscillator pitch (0-1 for OSC1/OSC2)
    OscillatorPitch(u8) = 7,
    /// Oscillator mix
    OscillatorMix = 8,
    /// LFO rate
    LFORate = 9,
    /// LFO depth
    LFODepth = 10,
    /// Attack time
    AttackTime = 11,
    /// Decay time
    DecayTime = 12,
    /// Sustain level
    SustainLevel = 13,
    /// Release time
    ReleaseTime = 14,
    /// Effect mix (0-5 for effects)
    EffectMix(u8) = 15,
    /// Effect parameter
    EffectParameter(u8, u8) = 16, // effect_index, parameter_index
    /// Reverb send
    ReverbSend = 17,
    /// Chorus send
    ChorusSend = 18,
    /// Delay mix
    DelayMix = 19,
    /// Custom parameter
    Custom(u16) = 20,
}

impl CCParameterTarget {
    /// Get parameter ID
    pub fn param_id(&self) -> u16 {
        match self {
            CCParameterTarget::None => 0,
            CCParameterTarget::MasterVolume => 1,
            CCParameterTarget::TrackVolume(n) => 10u16 + *n as u16,
            CCParameterTarget::TrackPan(n) => 20u16 + *n as u16,
            CCParameterTarget::FilterCutoff => 30,
            CCParameterTarget::FilterResonance => 31,
            CCParameterTarget::FilterDrive => 32,
            CCParameterTarget::OscillatorPitch(n) => 40u16 + *n as u16,
            CCParameterTarget::OscillatorMix => 50,
            CCParameterTarget::LFORate => 60,
            CCParameterTarget::LFODepth => 61,
            CCParameterTarget::AttackTime => 70,
            CCParameterTarget::DecayTime => 71,
            CCParameterTarget::SustainLevel => 72,
            CCParameterTarget::ReleaseTime => 73,
            CCParameterTarget::EffectMix(n) => 80u16 + *n as u16,
            CCParameterTarget::EffectParameter(e, p) => 100u16 + *e as u16 * 10u16 + *p as u16,
            CCParameterTarget::ReverbSend => 200,
            CCParameterTarget::ChorusSend => 201,
            CCParameterTarget::DelayMix => 202,
            CCParameterTarget::Custom(n) => 1000u16 + *n,
        }
    }
}

/// MIDI CC Manager
#[derive(Debug, Clone)]
pub struct MidiCCManager {
    /// Assignable CC mappings (CC0-CC15)
    assignable_ccs: [AssignableCC; MAX_CC_COUNT],
    /// Current CC values (0-127)
    cc_values: HashMap<u8, u8>,
    /// Standard CC values
    standard_cc_values: HashMap<u8, u8>,
    /// Program change enabled
    program_change_enabled: bool,
    /// Bank change enabled
    bank_change_enabled: bool,
    /// Current bank MSB
    current_bank_msb: u8,
    /// Current bank LSB
    current_bank_lsb: u8,
    /// Current program number (0-127)
    current_program: u8,
    /// MIDI channel (0-15, where 0 = omni)
    midi_channel: u8,
    /// CC learn mode
    cc_learn_mode: bool,
    /// CC learn target
    cc_learn_target: Option<usize>,
    /// Last CC learn time
    cc_learn_time: Option<u128>,
}

impl Default for MidiCCManager {
    fn default() -> Self {
        Self {
            assignable_ccs: [
                AssignableCC::new(80, "CC80", CCParameterTarget::None),
                AssignableCC::new(81, "CC81", CCParameterTarget::None),
                AssignableCC::new(82, "CC82", CCParameterTarget::None),
                AssignableCC::new(83, "CC83", CCParameterTarget::None),
                AssignableCC::new(84, "CC84", CCParameterTarget::None),
                AssignableCC::new(85, "CC85", CCParameterTarget::None),
                AssignableCC::new(86, "CC86", CCParameterTarget::None),
                AssignableCC::new(87, "CC87", CCParameterTarget::None),
                AssignableCC::new(88, "CC88", CCParameterTarget::None),
                AssignableCC::new(89, "CC89", CCParameterTarget::None),
                AssignableCC::new(90, "CC90", CCParameterTarget::None),
                AssignableCC::new(91, "CC91", CCParameterTarget::None),
                AssignableCC::new(92, "CC92", CCParameterTarget::None),
                AssignableCC::new(93, "CC93", CCParameterTarget::None),
                AssignableCC::new(94, "CC94", CCParameterTarget::None),
                AssignableCC::new(95, "CC95", CCParameterTarget::None),
            ],
            cc_values: HashMap::new(),
            standard_cc_values: HashMap::new(),
            program_change_enabled: true,
            bank_change_enabled: true,
            current_bank_msb: 0,
            current_bank_lsb: 0,
            current_program: 0,
            midi_channel: 0,
            cc_learn_mode: false,
            cc_learn_target: None,
            cc_learn_time: None,
        }
    }
}

impl MidiCCManager {
    /// Create new MIDI CC Manager with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set MIDI channel
    pub fn set_midi_channel(&mut self, channel: u8) {
        self.midi_channel = channel.min(15);
    }

    /// Get current MIDI channel
    pub fn midi_channel(&self) -> u8 {
        self.midi_channel
    }

    /// Process incoming CC message
    pub fn process_cc(&mut self, cc_number: u8, value: u8) -> Option<&mut AssignableCC> {
        // Store standard CC value
        self.standard_cc_values.insert(cc_number, value);

        // Check if in CC learn mode
        if self.cc_learn_mode {
            if let Some(target_index) = self.cc_learn_target {
                self.assignable_ccs[target_index].cc_number = cc_number;
                self.assignable_ccs[target_index].name = format!("CC{:02}", cc_number);
                self.cc_learn_mode = false;
                self.cc_learn_target = None;
                self.cc_learn_time = None;
                return Some(&mut self.assignable_ccs[target_index]);
            }
        }

        // Find matching assignable CC
        for cc in &mut self.assignable_ccs {
            if cc.cc_number == cc_number {
                self.cc_values.insert(cc_number, value);
                return Some(cc);
            }
        }

        None
    }

    /// Process Program Change message
    pub fn process_program_change(&mut self, program: u8) -> Option<u8> {
        if !self.program_change_enabled {
            return None;
        }

        self.current_program = program.min(127);
        Some(self.current_program)
    }

    /// Process Bank Select MSB (CC0)
    pub fn process_bank_select_msb(&mut self, value: u8) {
        if self.bank_change_enabled {
            self.current_bank_msb = value;
            self.standard_cc_values.insert(0, value);
        }
    }

    /// Process Bank Select LSB (CC32)
    pub fn process_bank_select_lsb(&mut self, value: u8) {
        if self.bank_change_enabled {
            self.current_bank_lsb = value;
            self.standard_cc_values.insert(32, value);
        }
    }

    /// Get current bank (MSB << 7 | LSB)
    pub fn current_bank(&self) -> u16 {
        (self.current_bank_msb as u16) << 7 | self.current_bank_lsb as u16
    }

    /// Get current program number
    pub fn current_program(&self) -> u8 {
        self.current_program
    }

    /// Get CC value (0-127)
    pub fn get_cc_value(&self, cc_number: u8) -> u8 {
        self.cc_values
            .get(&cc_number)
            .copied()
            .or(self.standard_cc_values.get(&cc_number).copied())
            .unwrap_or(0)
    }

    /// Get CC value as normalized float (0.0 - 1.0)
    pub fn get_cc_value_normalized(&self, cc_number: u8) -> f64 {
        self.get_cc_value(cc_number) as f64 / 127.0
    }

    /// Set assignable CC mapping
    pub fn set_assignable_cc(
        &mut self,
        index: usize,
        cc_number: u8,
        name: &str,
        target: CCParameterTarget,
    ) -> Result<(), MidiCCError> {
        if index >= MAX_CC_COUNT {
            return Err(MidiCCError::IndexOutOfRange(index));
        }

        self.assignable_ccs[index] = AssignableCC::new(cc_number, name, target);
        Ok(())
    }

    /// Get assignable CC mapping
    pub fn get_assignable_cc(&self, index: usize) -> Option<&AssignableCC> {
        self.assignable_ccs.get(index)
    }

    /// Get mutable assignable CC mapping
    pub fn get_assignable_cc_mut(&mut self, index: usize) -> Option<&mut AssignableCC> {
        self.assignable_ccs.get_mut(index)
    }

    /// Enable CC learn mode for specific CC slot
    pub fn enable_cc_learn(&mut self, index: usize) -> Result<(), MidiCCError> {
        if index >= MAX_CC_COUNT {
            return Err(MidiCCError::IndexOutOfRange(index));
        }

        self.cc_learn_mode = true;
        self.cc_learn_target = Some(index);
        self.cc_learn_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        );
        Ok(())
    }

    /// Disable CC learn mode
    pub fn disable_cc_learn(&mut self) {
        self.cc_learn_mode = false;
        self.cc_learn_target = None;
        self.cc_learn_time = None;
    }

    /// Check if CC learn is active
    pub fn is_cc_learn_active(&self) -> bool {
        self.cc_learn_mode
    }

    /// Check CC learn timeout
    pub fn check_cc_learn_timeout(&mut self) -> bool {
        if let Some(start_time) = self.cc_learn_time {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                - start_time;

            if elapsed > CC_LEARN_TIMEOUT_MS as u128 {
                self.disable_cc_learn();
                return true;
            }
        }
        false
    }

    /// Get normalized value for assignable CC
    pub fn get_assignable_cc_normalized(&self, index: usize) -> f64 {
        if let Some(cc) = self.assignable_ccs.get(index) {
            if !cc.enabled {
                return 0.0;
            }

            let value = self.get_cc_value(cc.cc_number) as f64 / 127.0;
            let range = cc.max_input - cc.min_input;

            let normalized = cc.min_input + value * range;

            if cc.bipolar {
                normalized * 2.0 - 1.0
            } else {
                normalized
            }
        } else {
            0.0
        }
    }

    /// Get all assignable CCs
    pub fn get_all_assignable_ccs(&self) -> &[AssignableCC; MAX_CC_COUNT] {
        &self.assignable_ccs
    }

    /// Reset all CC values
    pub fn reset_cc_values(&mut self) {
        self.cc_values.clear();
        for cc in &mut self.assignable_ccs {
            cc.enabled = false;
        }
    }

    /// Reset all controllers
    pub fn reset_all_controllers(&mut self) {
        self.reset_cc_values();
        self.standard_cc_values.clear();
        self.current_program = 0;
        self.current_bank_msb = 0;
        self.current_bank_lsb = 0;
    }

    /// All sound off
    pub fn all_sound_off(&mut self) {
        self.cc_values.clear();
    }

    /// All notes off
    pub fn all_notes_off(&mut self) {
        // This would trigger the synth to stop all playing notes
    }
}

/// MIDI CC Errors
#[derive(Debug, Clone)]
pub enum MidiCCError {
    /// Index out of range
    IndexOutOfRange(usize),
    /// Invalid CC number
    InvalidCCNumber(u8),
    /// Invalid target
    InvalidTarget,
    /// CC learn timeout
    CCLearnTimeout,
}

impl std::fmt::Display for MidiCCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiCCError::IndexOutOfRange(index) => {
                write!(f, "CC index {} out of range (max: {})", index, MAX_CC_COUNT)
            }
            MidiCCError::InvalidCCNumber(cc) => {
                write!(f, "Invalid CC number: {}", cc)
            }
            MidiCCError::InvalidTarget => write!(f, "Invalid target parameter"),
            MidiCCError::CCLearnTimeout => write!(f, "CC learn mode timed out"),
        }
    }
}

impl std::error::Error for MidiCCError {}

/// Convert CC value to frequency for filter cutoff
pub fn cc_to_cutoff(cc_value: u8) -> f64 {
    // MIDI CC 0-127 -> 20Hz - 20kHz (exponential mapping)
    let min_freq: f64 = 20.0;
    let max_freq: f64 = 20000.0;
    let normalized: f64 = cc_value as f64 / 127.0;
    min_freq * (max_freq / min_freq).powf(normalized)
}

/// Convert CC value to resonance (0-1)
pub fn cc_to_resonance(cc_value: u8) -> f64 {
    cc_value as f64 / 127.0
}

/// Convert CC value to time (seconds)
pub fn cc_to_time(cc_value: u8, min_time: f64, max_time: f64) -> f64 {
    // MIDI CC 0-127 -> min_time - max_time (exponential mapping)
    let normalized: f64 = cc_value as f64 / 127.0;
    min_time * (max_time / min_time).powf(normalized)
}

/// Convert CC value to pitch (semitones)
/// CC 0 = -range semitones, CC 64 = 0, CC 127 = +range semitones (bipolar)
pub fn cc_to_pitch(cc_value: u8, range: i8) -> i8 {
    let normalized = cc_value as f64 / 127.0;
    // Map 0..127 to -range..+range
    let pitch = (normalized * 2.0 - 1.0) * range as f64;
    pitch as i8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_cc_manager_new() {
        let manager = MidiCCManager::new();
        assert_eq!(manager.midi_channel(), 0);
        assert_eq!(manager.current_program(), 0);
        assert!(!manager.is_cc_learn_active());
    }

    #[test]
    fn test_process_cc() {
        let mut manager = MidiCCManager::new();
        let result = manager.process_cc(50, 64); // CC50 not in assignable list by default
        assert!(result.is_none());
    }

    #[test]
    fn test_process_program_change() {
        let mut manager = MidiCCManager::new();
        let result = manager.process_program_change(42);
        assert_eq!(result, Some(42));
        assert_eq!(manager.current_program(), 42);
    }

    #[test]
    fn test_bank_select() {
        let mut manager = MidiCCManager::new();
        manager.process_bank_select_msb(1);
        manager.process_bank_select_lsb(32);
        assert_eq!(manager.current_bank(), 160); // 1*128 + 32
    }

    #[test]
    fn test_cc_value_normalized() {
        let manager = MidiCCManager::new();
        let value = manager.get_cc_value_normalized(7);
        assert!((value - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_assignable_cc() {
        let mut manager = MidiCCManager::new();
        manager
            .set_assignable_cc(0, 74, "Filter Cutoff", CCParameterTarget::FilterCutoff)
            .unwrap();

        let cc = manager.get_assignable_cc(0).unwrap();
        assert_eq!(cc.cc_number, 74);
        assert_eq!(cc.name, "Filter Cutoff");
        assert_eq!(cc.target, CCParameterTarget::FilterCutoff);
    }

    #[test]
    fn test_cc_learn() {
        let mut manager = MidiCCManager::new();
        manager.enable_cc_learn(0).unwrap();
        assert!(manager.is_cc_learn_active());

        // Receive CC message
        manager.process_cc(74, 100);
        let cc = manager.get_assignable_cc(0).unwrap();
        assert_eq!(cc.cc_number, 74);
        assert!(!manager.is_cc_learn_active());
    }

    #[test]
    fn test_standard_cc_names() {
        assert_eq!(StandardCC::Modulation.default_name(), "Modulation");
        assert_eq!(StandardCC::Volume.default_name(), "Volume");
        assert_eq!(StandardCC::ReverbSend.default_name(), "Reverb Send");
    }

    #[test]
    fn test_standard_cc_switch_detection() {
        assert!(StandardCC::Sustain.is_switch());
        assert!(!StandardCC::Volume.is_switch());
        assert!(!StandardCC::FilterCutoff.is_switch()); // FilterCutoff is continuous, not a switch
    }

    #[test]
    fn test_cc_to_cutoff() {
        let cutoff = cc_to_cutoff(0);
        assert!((cutoff - 20.0).abs() < 0.1);

        let cutoff = cc_to_cutoff(127);
        assert!((cutoff - 20000.0).abs() < 1.0);
    }

    #[test]
    fn test_cc_to_time() {
        let time = cc_to_time(0, 0.001, 10.0);
        assert!((time - 0.001).abs() < 0.0001);

        let time = cc_to_time(127, 0.001, 10.0);
        assert!((time - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_cc_to_pitch() {
        let pitch = cc_to_pitch(0, 24);
        assert_eq!(pitch, -24);

        let pitch = cc_to_pitch(64, 24);
        assert_eq!(pitch, 0);

        let pitch = cc_to_pitch(127, 24);
        assert_eq!(pitch, 24);
    }

    #[test]
    fn test_reset_controllers() {
        let mut manager = MidiCCManager::new();
        manager.process_cc(7, 100);
        manager.process_program_change(50);
        manager.process_bank_select_msb(2);

        manager.reset_all_controllers();

        assert_eq!(manager.current_program(), 0);
        assert_eq!(manager.current_bank(), 0);
        assert_eq!(manager.get_cc_value(7), 0);
    }
}
