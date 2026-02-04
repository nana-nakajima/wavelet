// WAVELET - Send Effects Module
// 参考: Elektron Tonverk 发送效果系统
//
// 功能:
// - 3个发送效果轨 (Send A, B, C)
// - 每个发送轨2个效果槽
// - 可串行或并行连接
// - 独立发送量控制
// - 立体声返回
//
// 与Tonverk对比:
// Tonverk: 3轨发送, 每轨2个效果槽
// WAVELET: 3轨发送, 每轨2个效果槽 (对齐!)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Number of send effect tracks
pub const NUM_SEND_TRACKS: usize = 3;
/// Number of effect slots per send track
pub const SEND_EFFECT_SLOTS: usize = 2;

/// Send effect connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendConnection {
    Serial,   // 串行连接 (效果1 -> 效果2)
    Parallel, // 并行连接 (效果1 + 效果2)
}

/// Pre/Post fader selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrePost {
    PreFx,  // 前置效果
    PostFx, // 后置效果
}

/// A single effect slot in a send track
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendEffectSlot {
    /// Effect type (None if empty)
    pub effect_type: Option<String>,
    /// Effect parameters (JSON serialized)
    pub parameters: Vec<f64>,
    /// Whether this slot is enabled
    pub enabled: bool,
}

impl SendEffectSlot {
    /// Create an empty slot
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a slot with an effect
    pub fn with_effect(effect_type: &'static str) -> Self {
        Self {
            effect_type: Some(effect_type.to_string()),
            parameters: Vec::new(),
            enabled: true,
        }
    }

    /// Check if slot is empty
    pub fn is_empty(&self) -> bool {
        self.effect_type.is_none()
    }

    /// Clear the slot
    pub fn clear(&mut self) {
        self.effect_type = None;
        self.parameters.clear();
        self.enabled = false;
    }
}

/// A complete send effect track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTrack {
    /// Send track ID (0-2: A, B, C)
    pub id: u8,
    /// Send track name
    pub name: String,
    /// Effect slots
    pub slots: [SendEffectSlot; SEND_EFFECT_SLOTS],
    /// Connection type (Serial/Parallel)
    pub connection: SendConnection,
    /// Pre/Post fader selection
    pub pre_post: PrePost,
    /// Send level (0.0 - 1.0)
    pub send_level: f64,
    /// Return level (0.0 - 1.0)
    pub return_level: f64,
    /// Return pan (-1.0 to 1.0)
    pub return_pan: f64,
    /// Whether this send is active
    pub active: bool,
}

impl Default for SendTrack {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("Send A"),
            slots: [
                SendEffectSlot { effect_type: None, parameters: Vec::new(), enabled: false },
                SendEffectSlot { effect_type: None, parameters: Vec::new(), enabled: false },
            ],
            connection: SendConnection::Serial,
            pre_post: PrePost::PostFx,
            send_level: 0.5,
            return_level: 0.8,
            return_pan: 0.0,
            active: false,
        }
    }
}

impl SendTrack {
    /// Create a new send track
    pub fn new(id: u8, _name: &str) -> Self {
        let names = ["Send A", "Send B", "Send C"];
        Self {
            id,
            name: names[id as usize].to_string(),
            slots: [
                SendEffectSlot { effect_type: None, parameters: Vec::new(), enabled: false },
                SendEffectSlot { effect_type: None, parameters: Vec::new(), enabled: false },
            ],
            ..Default::default()
        }
    }

    /// Add an effect to a slot
    pub fn add_effect(&mut self, slot_index: usize, effect_type: &'static str) -> Result<(), SendFxError> {
        if slot_index >= SEND_EFFECT_SLOTS {
            return Err(SendFxError::InvalidSlot);
        }
        self.slots[slot_index] = SendEffectSlot::with_effect(effect_type);
        self.active = true;
        Ok(())
    }

    /// Remove effect from a slot
    pub fn remove_effect(&mut self, slot_index: usize) -> Result<(), SendFxError> {
        if slot_index >= SEND_EFFECT_SLOTS {
            return Err(SendFxError::InvalidSlot);
        }
        self.slots[slot_index].clear();
        // Check if any slot still has effects
        self.active = self.slots.iter().any(|slot| !slot.is_empty());
        Ok(())
    }

    /// Set send level
    pub fn set_send_level(&mut self, level: f64) {
        self.send_level = level.clamp(0.0, 1.0);
    }

    /// Set return level
    pub fn set_return_level(&mut self, level: f64) {
        self.return_level = level.clamp(0.0, 1.0);
    }

    /// Set return pan
    pub fn set_return_pan(&mut self, pan: f64) {
        self.return_pan = pan.clamp(-1.0, 1.0);
    }

    /// Check if send has any effects
    pub fn has_effects(&self) -> bool {
        self.slots.iter().any(|slot| !slot.is_empty())
    }

    /// Get the number of active effect slots
    pub fn active_slot_count(&self) -> usize {
        self.slots.iter().filter(|slot| !slot.is_empty()).count()
    }
}

/// Send effects manager - handles all send tracks
#[derive(Debug, Clone)]
pub struct SendFxManager {
    /// All send tracks
    tracks: Vec<SendTrack>,
    /// Master dry/wet mix
    master_mix: f64,
}

impl Default for SendFxManager {
    fn default() -> Self {
        let mut tracks = Vec::with_capacity(NUM_SEND_TRACKS);
        for i in 0..NUM_SEND_TRACKS {
            tracks.push(SendTrack::new(i as u8, ["Send A", "Send B", "Send C"][i]));
        }
        Self {
            tracks,
            master_mix: 1.0,
        }
    }
}

impl SendFxManager {
    /// Create a new send FX manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a send track by ID
    pub fn get_track(&self, id: u8) -> Option<&SendTrack> {
        self.tracks.get(id as usize)
    }

    /// Get a mutable send track by ID
    pub fn get_track_mut(&mut self, id: u8) -> Option<&mut SendTrack> {
        self.tracks.get_mut(id as usize)
    }

    /// Add an effect to a send track
    pub fn add_effect(&mut self, send_id: u8, slot_index: usize, effect_type: &'static str) -> Result<(), SendFxError> {
        if send_id >= NUM_SEND_TRACKS as u8 {
            return Err(SendFxError::InvalidSendId);
        }
        self.tracks[send_id as usize].add_effect(slot_index, effect_type)
    }

    /// Remove an effect from a send track
    pub fn remove_effect(&mut self, send_id: u8, slot_index: usize) -> Result<(), SendFxError> {
        if send_id >= NUM_SEND_TRACKS as u8 {
            return Err(SendFxError::InvalidSendId);
        }
        self.tracks[send_id as usize].remove_effect(slot_index)
    }

    /// Set send level
    pub fn set_send_level(&mut self, send_id: u8, level: f64) -> Result<(), SendFxError> {
        if send_id >= NUM_SEND_TRACKS as u8 {
            return Err(SendFxError::InvalidSendId);
        }
        self.tracks[send_id as usize].set_send_level(level);
        Ok(())
    }

    /// Set return level
    pub fn set_return_level(&mut self, send_id: u8, level: f64) -> Result<(), SendFxError> {
        if send_id >= NUM_SEND_TRACKS as u8 {
            return Err(SendFxError::InvalidSendId);
        }
        self.tracks[send_id as usize].set_return_level(level);
        Ok(())
    }

    /// Set return pan
    pub fn set_return_pan(&mut self, send_id: u8, pan: f64) -> Result<(), SendFxError> {
        if send_id >= NUM_SEND_TRACKS as u8 {
            return Err(SendFxError::InvalidSendId);
        }
        self.tracks[send_id as usize].set_return_pan(pan);
        Ok(())
    }

    /// Set connection type for a send track
    pub fn set_connection(&mut self, send_id: u8, connection: SendConnection) -> Result<(), SendFxError> {
        if send_id >= NUM_SEND_TRACKS as u8 {
            return Err(SendFxError::InvalidSendId);
        }
        self.tracks[send_id as usize].connection = connection;
        Ok(())
    }

    /// Set pre/post for a send track
    pub fn set_pre_post(&mut self, send_id: u8, pre_post: PrePost) -> Result<(), SendFxError> {
        if send_id >= NUM_SEND_TRACKS as u8 {
            return Err(SendFxError::InvalidSendId);
        }
        self.tracks[send_id as usize].pre_post = pre_post;
        Ok(())
    }

    /// Get total number of active send tracks
    pub fn active_track_count(&self) -> usize {
        self.tracks.iter().filter(|track| track.active).count()
    }

    /// Get master mix level
    pub fn master_mix(&self) -> f64 {
        self.master_mix
    }

    /// Set master mix level
    pub fn set_master_mix(&mut self, mix: f64) {
        self.master_mix = mix.clamp(0.0, 1.0);
    }

    /// Clear all send effects
    pub fn clear_all(&mut self) {
        for track in &mut self.tracks {
            track.slots = [
                SendEffectSlot { effect_type: None, parameters: Vec::new(), enabled: false },
                SendEffectSlot { effect_type: None, parameters: Vec::new(), enabled: false },
            ];
            track.active = false;
        }
    }

    /// Get send track names
    pub fn track_names(&self) -> Vec<&str> {
        self.tracks.iter().map(|t| t.name.as_str()).collect()
    }
}

/// Send FX related errors
#[derive(Debug, Clone)]
pub enum SendFxError {
    InvalidSendId,
    InvalidSlot,
    InvalidParameter,
    EffectNotFound,
}

impl fmt::Display for SendFxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendFxError::InvalidSendId => write!(f, "Invalid send track ID"),
            SendFxError::InvalidSlot => write!(f, "Invalid effect slot"),
            SendFxError::InvalidParameter => write!(f, "Invalid parameter value"),
            SendFxError::EffectNotFound => write!(f, "Effect not found"),
        }
    }
}

impl std::error::Error for SendFxError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_track_creation() {
        let track = SendTrack::new(0, "Reverb Send");
        assert_eq!(track.id, 0);
        assert_eq!(track.name, "Send A");
        assert!(!track.active);
        assert!(!track.has_effects());
    }

    #[test]
    fn test_send_effect_slot() {
        let slot = SendEffectSlot::empty();
        assert!(slot.is_empty());
        assert!(!slot.enabled);
    }

    #[test]
    fn test_add_effect() {
        let mut track = SendTrack::new(0, "Test");
        assert!(track.add_effect(0, "Reverb").is_ok());
        assert!(track.has_effects());
        assert_eq!(track.active_slot_count(), 1);
    }

    #[test]
    fn test_remove_effect() {
        let mut track = SendTrack::new(0, "Test");
        track.add_effect(0, "Reverb").unwrap();
        assert!(track.remove_effect(0).is_ok());
        assert!(!track.has_effects());
    }

    #[test]
    fn test_level_control() {
        let mut track = SendTrack::new(0, "Test");
        track.set_send_level(0.75);
        track.set_return_level(0.8);
        track.set_return_pan(-0.5);
        
        assert!((track.send_level - 0.75).abs() < 0.001);
        assert!((track.return_level - 0.8).abs() < 0.001);
        assert!((track.return_pan - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_send_fx_manager() {
        let mut manager = SendFxManager::new();
        assert_eq!(manager.active_track_count(), 0);
        
        // Add an effect
        assert!(manager.add_effect(0, 0, "Reverb").is_ok());
        assert_eq!(manager.active_track_count(), 1);
        
        // Set levels
        assert!(manager.set_send_level(0, 0.7).is_ok());
        assert!(manager.set_return_level(0, 0.9).is_ok());
        assert!(manager.set_return_pan(0, 0.3).is_ok());
    }

    #[test]
    fn test_connection_types() {
        let mut track = SendTrack::new(0, "Test");
        track.connection = SendConnection::Serial;
        assert_eq!(track.connection, SendConnection::Serial);
        
        track.connection = SendConnection::Parallel;
        assert_eq!(track.connection, SendConnection::Parallel);
    }

    #[test]
    fn test_pre_post() {
        let mut track = SendTrack::new(0, "Test");
        assert_eq!(track.pre_post, PrePost::PostFx);
        
        track.pre_post = PrePost::PreFx;
        assert_eq!(track.pre_post, PrePost::PreFx);
    }

    #[test]
    fn test_clear_all() {
        let mut manager = SendFxManager::new();
        manager.add_effect(0, 0, "Reverb").unwrap();
        manager.add_effect(1, 0, "Delay").unwrap();
        
        assert_eq!(manager.active_track_count(), 2);
        
        manager.clear_all();
        assert_eq!(manager.active_track_count(), 0);
    }

    #[test]
    fn test_multi_slot_effects() {
        let mut track = SendTrack::new(0, "Test");
        track.add_effect(0, "Reverb").unwrap();
        track.add_effect(1, "Delay").unwrap();
        
        assert_eq!(track.active_slot_count(), 2);
    }

    #[test]
    fn test_level_clamping() {
        let mut track = SendTrack::new(0, "Test");
        track.set_send_level(1.5); // Should clamp to 1.0
        track.set_return_level(-0.5); // Should clamp to 0.0
        track.set_return_pan(2.0); // Should clamp to 1.0
        
        assert!((track.send_level - 1.0).abs() < 0.001);
        assert!((track.return_level - 0.0).abs() < 0.001);
        assert!((track.return_pan - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_track_names() {
        let manager = SendFxManager::new();
        let names = manager.track_names();
        assert_eq!(names.len(), NUM_SEND_TRACKS);
        assert_eq!(names[0], "Send A");
        assert_eq!(names[1], "Send B");
        assert_eq!(names[2], "Send C");
    }

    #[test]
    fn test_error_cases() {
        let mut manager = SendFxManager::new();
        assert!(manager.add_effect(5, 0, "Reverb").is_err()); // Invalid send ID
        assert!(manager.remove_effect(5, 0).is_err()); // Invalid send ID
        assert!(manager.set_send_level(5, 0.5).is_err()); // Invalid send ID
    }
}
