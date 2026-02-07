use crate::envelope::Envelope;
use crate::oscillator::Oscillator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    Bus,
    Send,
    Mix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Forward,
    Reverse,
    ForwardLoop,
    ReverseLoop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MachineType {
    Single,
    Multi,
    Subtrack,
    Midi,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LfoWaveform {
    Triangle,
    Sine,
    Square,
    Sawtooth,
    Random,
    Exponential,
    Ramp,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LfoMode {
    Free,
    Trig,
    Hold,
    OneShot,
    Half,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrigType {
    None,
    Note,
    Lock,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrigCondition {
    None,
    Fill,
    NotFill,
    Pre,
    NotPre,
    Nei,
    NotNei,
    First,
    NotFirst,
    Last,
    NotLast,
    AB,
    NotAB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrigRate {
    Whole,
    Half,
    Third,
    Quarter,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    Tenth,
    Twelfth,
    Sixteenth,
    TwentyFourth,
    ThirtySecond,
    FortyEighth,
    NinetySixth,
    OneTwentyEighth,
}

impl RetrigRate {
    pub fn as_fraction(&self) -> (u32, u32) {
        match self {
            RetrigRate::Whole => (1, 1),
            RetrigRate::Half => (1, 2),
            RetrigRate::Third => (1, 3),
            RetrigRate::Quarter => (1, 4),
            RetrigRate::Fifth => (1, 5),
            RetrigRate::Sixth => (1, 6),
            RetrigRate::Seventh => (1, 7),
            RetrigRate::Eighth => (1, 8),
            RetrigRate::Tenth => (1, 10),
            RetrigRate::Twelfth => (1, 12),
            RetrigRate::Sixteenth => (1, 16),
            RetrigRate::TwentyFourth => (1, 24),
            RetrigRate::ThirtySecond => (1, 32),
            RetrigRate::FortyEighth => (1, 48),
            RetrigRate::NinetySixth => (1, 96),
            RetrigRate::OneTwentyEighth => (1, 128),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Routing {
    MixAB,
    OutCD,
    OutEF,
    Bus1,
    Bus2,
    Bus3,
    Bus4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxSlotIndex {
    Slot1,
    Slot2,
}

pub const MAX_VOICES: usize = 8;
pub const NUM_AUDIO_TRACKS: u8 = 8;
pub const NUM_BUS_TRACKS: u8 = 4;
pub const NUM_SEND_TRACKS: u8 = 3;
pub const NUM_TRACKS: u8 = 16;
pub const FX_SLOTS_AUDIO: usize = 2;
pub const FX_SLOTS_BUS: usize = 2;
pub const FX_SLOTS_SEND: usize = 1;
pub const FX_SLOTS_MIX: usize = 1;
pub const STEPS_PER_PAGE: usize = 16;
pub const PAGES: usize = 16;
pub const MAX_STEPS: usize = STEPS_PER_PAGE * PAGES;
