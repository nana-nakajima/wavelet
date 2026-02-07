//! WebAssembly Audio Engine Bridge
//!
//! This module provides the bridge between the Rust DSP core and
//! Web Audio API via WebAssembly. It handles:
//! - Real-time audio processing via AudioWorklet
//! - Parameter messaging via postMessage
//! - Memory sharing between Rust and JavaScript

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
use wee_alloc::WeeAlloc;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

/// Audio processing state
static mut AUDIO_RUNNING: AtomicBool = AtomicBool::new(false);

/// Sample rate for audio processing
const DEFAULT_SAMPLE_RATE: f64 = 48000.0;

/// Buffer size for audio processing
const DEFAULT_BUFFER_SIZE: usize = 128;

/// Message types for postMessage communication
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum AudioMessage {
    /// Note on event
    NoteOn { note: u8, velocity: f32, track: u8 },
    /// Note off event
    NoteOff { note: u8, track: u8 },
    /// Set parameter value
    SetParam {
        track: u8,
        param: String,
        value: f32,
    },
    /// Set track mute
    SetMute { track: u8, muted: bool },
    /// Set track solo
    SetSolo { track: u8, solo: bool },
    /// Set track volume
    SetVolume { track: u8, volume: f32 },
    /// Set master volume
    SetMasterVolume { volume: f32 },
    /// Set tempo (BPM)
    SetTempo { tempo: f32 },
    /// Start playback
    Play,
    /// Stop playback
    Stop,
    /// Start recording
    Record,
    /// Stop recording
    StopRecord,
    /// Load sample
    LoadSample {
        track: u8,
        sample_id: String,
        data: Vec<f32>,
    },
    /// Clear sample
    ClearSample { track: u8 },
    /// Get current state
    GetState,
}

/// Response types for postMessage
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum AudioResponse {
    /// State update
    State(StateUpdate),
    /// Parameter value
    ParamValue {
        track: u8,
        param: String,
        value: f32,
    },
    /// Waveform data for visualization
    Waveform { track: u8, data: Vec<f32> },
    /// Spectrum data for visualization
    Spectrum { track: u8, data: Vec<f32> },
    /// Latency measurement
    Latency { round_trip: f32, processing: f32 },
    /// Error message
    Error { message: String },
}

/// Audio engine state
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateUpdate {
    pub playing: bool,
    pub recording: bool,
    pub tempo: f32,
    pub current_step: u16,
    pub tracks: Vec<TrackState>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackState {
    pub id: u8,
    pub name: String,
    pub muted: bool,
    pub solo: bool,
    pub volume: f32,
    pub pan: f32,
    pub has_sample: bool,
}

/// Parameter smoothing configuration
#[derive(Clone, Copy, Debug)]
pub struct SmoothingConfig {
    pub attack_ms: f32,
    pub release_ms: f32,
    pub sample_rate: f64,
}

impl Default for SmoothingConfig {
    fn default() -> Self {
        Self {
            attack_ms: 1.0,
            release_ms: 10.0,
            sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }
}

impl SmoothingConfig {
    /// Create smoothing config with sample rate
    pub fn new(sample_rate: f64) -> Self {
        Self {
            attack_ms: 1.0,
            release_ms: 10.0,
            sample_rate,
        }
    }

    /// Calculate smoothing coefficient from time constant
    pub fn coeff(&self, time_ms: f32, rising: bool) -> f32 {
        let tau = time_ms / 1000.0;
        let alpha = 1.0 - (-1.0 / (tau * self.sample_rate)).exp();
        if rising {
            alpha
        } else {
            alpha
        }
    }
}

/// Linear parameter smoother for anti-pop
#[derive(Clone, Debug)]
pub struct ParameterSmoother {
    current: f32,
    target: f32,
    coeff_rise: f32,
    coeff_fall: f32,
}

impl ParameterSmoother {
    pub fn new(config: SmoothingConfig) -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            coeff_rise: config.coeff(config.attack_ms, true),
            coeff_fall: config.coeff(config.release_ms, false),
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target.clamp(0.0, 1.0);
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let diff = self.target - self.current;
        let coeff = if diff > 0.0 {
            self.coeff_rise
        } else {
            self.coeff_fall
        };
        self.current += diff * coeff;

        // Snap to target if very close
        if diff.abs() < 0.0001 {
            self.current = self.target;
        }
        self.current
    }

    pub fn set_value(&mut self, value: f32) {
        self.current = value.clamp(0.0, 1.0);
        self.target = self.current;
    }
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Initialize audio context (call from JavaScript)
#[wasm_bindgen]
pub fn init_audio(sample_rate: f64) -> Result<JsValue, JsValue> {
    let sample_rate = if sample_rate > 0.0 {
        sample_rate
    } else {
        DEFAULT_SAMPLE_RATE
    };

    // Create and return the audio host as a JS object
    let host = WasmAudioHost::new(sample_rate);
    Ok(JsValue::from_serde(&host).map_err(|e| JsValue::from(e.to_string()))?)
}

/// Get version string
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// WasmAudioHost - Main interface for Web Audio integration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmAudioHost {
    sample_rate: f64,
    buffer_size: usize,
    smoothing: SmoothingConfig,
    #[serde(skip)]
    smoother: ParameterSmoother,
    master_volume: f32,
    tempo: f32,
    playing: bool,
    recording: bool,
    current_step: u16,
    tracks: Vec<WasmTrack>,
}

impl Default for WasmAudioHost {
    fn default() -> Self {
        Self::new(DEFAULT_SAMPLE_RATE)
    }
}

impl WasmAudioHost {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,
            buffer_size: DEFAULT_BUFFER_SIZE,
            smoothing: SmoothingConfig::new(sample_rate),
            smoother: ParameterSmoother::new(SmoothingConfig::new(sample_rate)),
            master_volume: 0.8,
            tempo: 120.0,
            playing: false,
            recording: false,
            current_step: 0,
            tracks: (0..16).map(|i| WasmTrack::new(i, sample_rate)).collect(),
        }
    }

    /// Process audio buffer (called from AudioWorklet)
    pub fn process(&mut self, output: &mut [f32]) {
        if !self.playing {
            // Silence output when not playing
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
            return;
        }

        // Mix all tracks
        let mut mix = vec![0.0; output.len()];
        for track in &mut self.tracks {
            track.process(&mut mix);
        }

        // Apply master volume with smoothing
        let smoothed_vol = self.smoother.process();
        for (i, sample) in output.iter_mut().enumerate() {
            *sample = mix[i] * smoothed_vol;
        }

        // Advance step sequencer if needed
        self.advance_sequencer(output.len());
    }

    fn advance_sequencer(&mut self, _frames: usize) {
        // TODO: Implement step advancement based on tempo
        self.current_step = (self.current_step + 1) % 256;
    }

    /// Handle incoming message from JavaScript
    pub fn handle_message(&mut self, message: &str) -> Result<String, String> {
        let msg: AudioMessage = serde_json::from_str(message).map_err(|e| e.to_string())?;

        self.process_message(msg);

        // Return current state
        self.get_state().map_err(|e| e.to_string())
    }

    fn process_message(&mut self, message: AudioMessage) {
        match message {
            AudioMessage::NoteOn {
                note,
                velocity,
                track,
            } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.note_on(note, velocity);
                }
            }
            AudioMessage::NoteOff { note, track } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.note_off(note);
                }
            }
            AudioMessage::SetParam {
                track,
                param,
                value,
            } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.set_param(&param, value);
                }
            }
            AudioMessage::SetMute { track, muted } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.set_mute(muted);
                }
            }
            AudioMessage::SetSolo { track, solo } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.set_solo(solo);
                }
            }
            AudioMessage::SetVolume { track, volume } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.set_volume(volume);
                }
            }
            AudioMessage::SetMasterVolume { volume } => {
                self.master_volume = volume.clamp(0.0, 1.0);
                self.smoother.set_target(self.master_volume);
            }
            AudioMessage::SetTempo { tempo } => {
                self.tempo = tempo.clamp(20.0, 300.0);
            }
            AudioMessage::Play => {
                self.playing = true;
            }
            AudioMessage::Stop => {
                self.playing = false;
                self.current_step = 0;
            }
            AudioMessage::Record => {
                self.recording = true;
            }
            AudioMessage::StopRecord => {
                self.recording = false;
            }
            AudioMessage::LoadSample { track, data, .. } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.load_sample(&data);
                }
            }
            AudioMessage::ClearSample { track } => {
                if let Some(t) = self.tracks.get_mut(track as usize) {
                    t.clear_sample();
                }
            }
            AudioMessage::GetState => {}
        }
    }

    fn get_state(&self) -> Result<String, String> {
        let state = StateUpdate {
            playing: self.playing,
            recording: self.recording,
            tempo: self.tempo,
            current_step: self.current_step,
            tracks: self.tracks.iter().map(|t| t.get_state()).collect(),
        };
        serde_json::to_string(&state).map_err(|e| e.to_string())
    }

    /// Get audio buffers for visualization
    pub fn get_waveform(&self, track: usize) -> Vec<f32> {
        self.tracks
            .get(track)
            .map(|t| t.get_waveform())
            .unwrap_or_default()
    }

    pub fn get_spectrum(&self, track: usize) -> Vec<f32> {
        self.tracks
            .get(track)
            .map(|t| t.get_spectrum())
            .unwrap_or_default()
    }
}

/// Individual track state for WASM
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmTrack {
    id: u8,
    name: String,
    muted: bool,
    solo: bool,
    volume: f32,
    pan: f32,
    has_sample: bool,
    #[serde(skip)]
    sample_data: Option<Vec<f32>>,
    #[serde(skip)]
    active_voices: Vec<ActiveVoice>,
    sample_rate: f64,
}

impl WasmTrack {
    fn new(id: u8, sample_rate: f64) -> Self {
        Self {
            id,
            name: format!("TRACK {}", id + 1),
            muted: false,
            solo: false,
            volume: 0.8,
            pan: 0.0,
            has_sample: false,
            sample_data: None,
            active_voices: Vec::new(),
            sample_rate,
        }
    }

    fn process(&mut self, output: &mut [f32]) {
        if self.muted && self.solo {
            // Only muted if nothing is soloed
        }

        for voice in &mut self.active_voices {
            voice.process(output);
        }

        // Clean up finished voices
        self.active_voices.retain(|v| !v.finished());
    }

    fn note_on(&mut self, note: u8, velocity: f32) {
        if let Some(ref sample) = self.sample_data {
            let voice = ActiveVoice::new(note, velocity, sample, self.volume, self.sample_rate);
            self.active_voices.push(voice);
        }
    }

    fn note_off(&mut self, _note: u8) {
        // Release voices - simplified for now
    }

    fn set_param(&mut self, _param: &str, _value: f32) {
        // TODO: Connect to DSP parameters
    }

    fn set_mute(&mut self, muted: bool) {
        self.muted = muted;
    }

    fn set_solo(&mut self, solo: bool) {
        self.solo = solo;
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    fn load_sample(&mut self, data: &[f32]) {
        self.sample_data = Some(data.to_vec());
        self.has_sample = true;
    }

    fn clear_sample(&mut self) {
        self.sample_data = None;
        self.has_sample = false;
        self.active_voices.clear();
    }

    fn get_state(&self) -> TrackState {
        TrackState {
            id: self.id,
            name: self.name.clone(),
            muted: self.muted,
            solo: self.solo,
            volume: self.volume,
            pan: self.pan,
            has_sample: self.has_sample,
        }
    }

    fn get_waveform(&self) -> Vec<f32> {
        self.sample_data
            .as_ref()
            .map(|s| s.iter().take(1024).copied().collect())
            .unwrap_or_default()
    }

    fn get_spectrum(&self) -> Vec<f32> {
        // Placeholder - would need FFT implementation
        vec![0.0; 64]
    }
}

/// Active voice for sample playback
#[derive(Clone, Debug)]
struct ActiveVoice {
    note: u8,
    velocity: f32,
    position: usize,
    playing: bool,
    sample: Vec<f32>,
    volume: f32,
    pitch_ratio: f32,
}

impl ActiveVoice {
    fn new(note: u8, velocity: f32, sample: &[f32], volume: f32, sample_rate: f64) -> Self {
        let root_note = 60.0; // C4
        let pitch_ratio = (2.0_f64).powf((note as f64 - root_note) / 12.0);

        Self {
            note,
            velocity,
            position: 0,
            playing: true,
            sample: sample.to_vec(),
            volume: velocity * volume,
            pitch_ratio: pitch_ratio as f32,
        }
    }

    fn process(&mut self, output: &mut [f32]) {
        if !self.playing {
            return;
        }

        for (i, out_sample) in output.iter_mut().enumerate() {
            if self.position < self.sample.len() {
                let sample = self.sample[self.position];
                *out_sample += sample * self.volume;
                self.position = (self.position as f32 * self.pitch_ratio) as usize;
            } else {
                self.playing = false;
                break;
            }
        }
    }

    fn finished(&self) -> bool {
        !self.playing
    }
}

/// Memory allocation for WASM (optional with wee_alloc)
#[cfg(feature = "wee_alloc")]
#[wasm_bindgen]
pub fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[cfg(feature = "wee_alloc")]
#[wasm_bindgen]
pub fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        Vec::from_raw_parts(ptr, 0, size);
    }
}
