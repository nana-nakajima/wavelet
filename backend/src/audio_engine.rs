use crate::models::User;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioMessage {
    Play,
    Stop,
    SetTempo {
        tempo: f32,
    },
    SetPosition {
        step: u16,
        page: u8,
    },
    SetTrackParam {
        track: u8,
        param: String,
        value: f32,
    },
    SetTrackMute {
        track: u8,
        muted: bool,
    },
    SetTrackSolo {
        track: u8,
        solo: bool,
    },
    SetTrackVolume {
        track: u8,
        volume: f32,
    },
    SetTrackPan {
        track: u8,
        pan: f32,
    },
    SetFxSlot {
        track: u8,
        slot: u8,
        fx_type: String,
    },
    SetFxParam {
        track: u8,
        slot: u8,
        param: String,
        value: f32,
    },
    SetStep {
        track: u8,
        page: u8,
        step: u8,
        field: String,
        value: f32,
    },
    SetTrig {
        track: u8,
        page: u8,
        step: u8,
        trig_type: String,
    },
    AddParamLock {
        track: u8,
        page: u8,
        step: u8,
        param: String,
        value: f32,
    },
    RemoveParamLock {
        track: u8,
        page: u8,
        step: u8,
        param: String,
    },
    LoadPreset {
        preset_id: String,
    },
    SavePreset {
        name: String,
    },
    Subscribe {
        tracks: Vec<u8>,
    },
    Unsubscribe,
    NoteOn {
        note: u8,
        velocity: f32,
        track: Option<u8>,
    },
    NoteOff {
        note: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioState {
    pub tracks: Vec<TrackState>,
    pub transport: TransportState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackState {
    pub id: u8,
    pub name: String,
    pub muted: bool,
    pub solo: bool,
    pub volume: f32,
    pub pan: f32,
    pub current_page: String,
    pub src_params: serde_json::Value,
    pub fltr_params: serde_json::Value,
    pub amp_params: serde_json::Value,
    pub fx_slots: Vec<FxSlotState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxSlotState {
    pub id: u8,
    pub fx_type: String,
    pub bypass: bool,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportState {
    pub playing: bool,
    pub recording: bool,
    pub tempo: f32,
    pub current_step: u16,
    pub current_page: u8,
    pub pattern_id: u8,
    pub pattern_bank: char,
}

pub struct AudioEngine {
    pub state: Arc<Mutex<AudioState>>,
    pub subscribers: Vec<Arc<Mutex<()>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let tracks = (1..=16)
            .map(|i| TrackState {
                id: i,
                name: format!("TRACK {}", i),
                muted: false,
                solo: false,
                volume: 0.8,
                pan: 0.0,
                current_page: "trig".to_string(),
                src_params: serde_json::json!({}),
                fltr_params: serde_json::json!({}),
                amp_params: serde_json::json!({}),
                fx_slots: vec![
                    FxSlotState {
                        id: 1,
                        fx_type: "Bypass".to_string(),
                        bypass: true,
                        params: serde_json::json!({}),
                    },
                    FxSlotState {
                        id: 2,
                        fx_type: "Bypass".to_string(),
                        bypass: true,
                        params: serde_json::json!({}),
                    },
                ],
            })
            .collect();

        let transport = TransportState {
            playing: false,
            recording: false,
            tempo: 120.0,
            current_step: 0,
            current_page: 0,
            pattern_id: 0,
            pattern_bank: 'A',
        };

        AudioEngine {
            state: Arc::new(Mutex::new(AudioState { tracks, transport })),
            subscribers: Vec::new(),
        }
    }

    pub fn handle_message(&mut self, message: AudioMessage) -> Option<AudioState> {
        let mut state = self.state.lock().unwrap();

        match message {
            AudioMessage::Play => {
                state.transport.playing = true;
            }
            AudioMessage::Stop => {
                state.transport.playing = false;
                state.transport.current_step = 0;
            }
            AudioMessage::SetTempo { tempo } => {
                state.transport.tempo = tempo.clamped(20.0, 300.0);
            }
            AudioMessage::SetTrackMute { track, muted } => {
                if let Some(t) = state.tracks.iter_mut().find(|t| t.id == track) {
                    t.muted = muted;
                }
            }
            AudioMessage::SetTrackSolo { track, solo } => {
                if let Some(t) = state.tracks.iter_mut().find(|t| t.id == track) {
                    t.solo = solo;
                }
            }
            AudioMessage::SetTrackVolume { track, volume } => {
                if let Some(t) = state.tracks.iter_mut().find(|t| t.id == track) {
                    t.volume = volume.clamped(0.0, 1.0);
                }
            }
            AudioMessage::SetTrackPan { track, pan } => {
                if let Some(t) = state.tracks.iter_mut().find(|t| t.id == track) {
                    t.pan = pan.clamped(-1.0, 1.0);
                }
            }
            _ => {}
        }

        Some(state.clone())
    }

    pub fn get_state(&self) -> AudioState {
        self.state.lock().unwrap().clone()
    }

    pub fn subscribe(&mut self) -> Arc<Mutex<()>> {
        let subscriber = Arc::new(Mutex::new(()));
        self.subscribers.push(subscriber.clone());
        subscriber
    }
}

trait Clamped {
    fn clamped(self, min: Self, max: Self) -> Self;
}

impl<T: PartialOrd> Clamped for T {
    fn clamped(self, min: T, max: T) -> T {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}
