//! Per-track Effects Module
//!
//! This module provides per-track audio effects processing for the Step Sequencer.
//! Each track can have its own effects chain with up to 4 effect slots.
//!
//! # Architecture
//!
//! - `TrackEffectSlot`: Single effect slot (type, instance, parameters)
//! - `TrackEffects`: Effects chain for a single track (4 slots)
//! - `EffectFactory`: Factory for creating effect instances
//! - `PerTrackEffectsManager`: Manages all 8 track effects

use std::collections::HashMap;
use std::fmt;
use crate::effects::{Effect, EffectType, Delay, Distortion, Saturation, Compressor, SimpleEq, Chorus, BiquadFilter};

/// Parameter identifier for effects (for parameter locks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectParameterId {
    /// Filter cutoff frequency (0.0 - 1.0)
    FilterCutoff,
    
    /// Filter resonance (0.0 - 1.0)
    FilterResonance,
    
    /// Distortion/Saturation amount (0.0 - 1.0)
    DistortionAmount,
    
    /// Compressor threshold (-60.0 - 0.0 dB)
    CompressorThreshold,
    
    /// Compressor ratio (1.0 - 20.0)
    CompressorRatio,
    
    /// Compressor attack (0.001 - 0.5 s)
    CompressorAttack,
    
    /// Compressor release (0.01 - 1.0 s)
    CompressorRelease,
    
    /// EQ low gain (-12.0 - 12.0 dB)
    EqLowGain,
    
    /// EQ mid gain (-12.0 - 12.0 dB)
    EqMidGain,
    
    /// EQ high gain (-12.0 - 12.0 dB)
    EqHighGain,
    
    /// Chorus rate (0.1 - 10.0 Hz)
    ChorusRate,
    
    /// Chorus depth (0.0 - 1.0)
    ChorusDepth,
    
    /// Chorus feedback (0.0 - 0.9)
    ChorusFeedback,
    
    /// Effect mix (0.0 - 1.0)
    Mix,
    
    /// Custom parameter (reserved for future use)
    Custom(u8),
}

impl Default for EffectParameterId {
    fn default() -> Self {
        EffectParameterId::Mix
    }
}

impl fmt::Display for EffectParameterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectParameterId::FilterCutoff => write!(f, "Filter Cutoff"),
            EffectParameterId::FilterResonance => write!(f, "Filter Resonance"),
            EffectParameterId::DistortionAmount => write!(f, "Distortion Amount"),
            EffectParameterId::CompressorThreshold => write!(f, "Compressor Threshold"),
            EffectParameterId::CompressorRatio => write!(f, "Compressor Ratio"),
            EffectParameterId::CompressorAttack => write!(f, "Compressor Attack"),
            EffectParameterId::CompressorRelease => write!(f, "Compressor Release"),
            EffectParameterId::EqLowGain => write!(f, "EQ Low"),
            EffectParameterId::EqMidGain => write!(f, "EQ Mid"),
            EffectParameterId::EqHighGain => write!(f, "EQ High"),
            EffectParameterId::ChorusRate => write!(f, "Chorus Rate"),
            EffectParameterId::ChorusDepth => write!(f, "Chorus Depth"),
            EffectParameterId::ChorusFeedback => write!(f, "Chorus Feedback"),
            EffectParameterId::Mix => write!(f, "Mix"),
            EffectParameterId::Custom(n) => write!(f, "Custom {}", n),
        }
    }
}

/// Configuration for a single effect slot
#[derive(Debug, Clone, PartialEq)]
pub struct TrackEffectSlotConfig {
    /// Effect type (None if slot is empty)
    pub effect_type: Option<EffectType>,
    
    /// Whether the effect is enabled
    pub enabled: bool,
    
    /// Wet/dry mix (0.0 = dry, 1.0 = wet)
    pub mix: f32,
    
    /// Parameter locks for this effect
    pub param_locks: HashMap<EffectParameterId, f32>,
}

impl Default for TrackEffectSlotConfig {
    fn default() -> Self {
        Self {
            effect_type: None,
            enabled: false,
            mix: 0.5,
            param_locks: HashMap::new(),
        }
    }
}

/// Single effect slot with runtime instance
#[derive(Debug, Clone)]
pub struct TrackEffectSlot {
    /// Effect type
    effect_type: Option<EffectType>,
    
    /// Effect instance (None if slot is empty or disabled)
    effect: Option<Box<dyn Effect>>,
    
    /// Whether the effect is enabled
    enabled: bool,
    
    /// Wet/dry mix (0.0 = dry, 1.0 = wet)
    mix: f32,
    
    /// Parameter locks for this effect
    param_locks: HashMap<EffectParameterId, f32>,
}

impl TrackEffectSlot {
    /// Creates a new empty effect slot
    pub fn new() -> Self {
        Self {
            effect_type: None,
            effect: None,
            enabled: false,
            mix: 0.5,
            param_locks: HashMap::new(),
        }
    }
    
    /// Creates a new effect slot with the specified effect
    pub fn with_effect<E: Effect + 'static>(effect: E) -> Self {
        Self {
            effect_type: Some(effect.effect_type()),
            effect: Some(Box::new(effect)),
            enabled: true,
            mix: 0.5,
            param_locks: HashMap::new(),
        }
    }
    
    /// Creates a slot from configuration
    pub fn from_config(config: &TrackEffectSlotConfig, sample_rate: f32) -> Self {
        let mut slot = Self::new();
        
        if let Some(effect_type) = config.effect_type {
            if let Some(effect) = create_effect_instance(effect_type, sample_rate) {
                slot.effect_type = Some(effect_type);
                slot.effect = Some(effect);
            }
        }
        
        slot.enabled = config.enabled;
        slot.mix = config.mix;
        slot.param_locks = config.param_locks.clone();
        
        slot
    }
    
    /// Gets the current effect type
    pub fn effect_type(&self) -> Option<EffectType> {
        self.effect_type
    }
    
    /// Checks if the slot has an effect
    pub fn is_empty(&self) -> bool {
        self.effect.is_none()
    }
    
    /// Checks if the effect is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.effect.is_some()
    }
    
    /// Gets the wet/dry mix
    pub fn mix(&self) -> f32 {
        self.mix
    }
    
    /// Sets the wet/dry mix
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    /// Enables or disables the effect
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Gets the parameter locks
    pub fn param_locks(&self) -> &HashMap<EffectParameterId, f32> {
        &self.param_locks
    }
    
    /// Adds a parameter lock
    pub fn add_param_lock(&mut self, param: EffectParameterId, value: f32) {
        self.param_locks.insert(param, value);
    }
    
    /// Removes a parameter lock
    pub fn remove_param_lock(&mut self, param: &EffectParameterId) {
        self.param_locks.remove(param);
    }
    
    /// Clears all parameter locks
    pub fn clear_param_locks(&mut self) {
        self.param_locks.clear();
    }
    
    /// Processes a sample through this effect slot
    pub fn process(&mut self, input: f32) -> f32 {
        if !self.enabled || self.effect.is_none() {
            return input;
        }
        
        if let Some(ref mut effect) = self.effect {
            let wet = effect.process(input);
            input * (1.0 - self.mix) + wet * self.mix
        } else {
            input
        }
    }
    
    /// Resets the effect
    pub fn reset(&mut self) {
        if let Some(ref mut effect) = self.effect {
            effect.reset();
        }
    }
    
    /// Converts to config for serialization
    pub fn to_config(&self) -> TrackEffectSlotConfig {
        TrackEffectSlotConfig {
            effect_type: self.effect_type,
            enabled: self.enabled,
            mix: self.mix,
            param_locks: self.param_locks.clone(),
        }
    }
}

impl Default for TrackEffectSlot {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create effect instances by type
fn create_effect_instance(effect_type: EffectType, sample_rate: f32) -> Option<Box<dyn Effect>> {
    match effect_type {
        EffectType::Filter => Some(Box::new(BiquadFilter::new())),
        EffectType::Saturation => Some(Box::new(Saturation::new())),
        EffectType::Compressor => Some(Box::new(Compressor::new(sample_rate))),
        EffectType::SimpleEQ => Some(Box::new(SimpleEq::new(sample_rate))),
        EffectType::Chorus => Some(Box::new(Chorus::new(sample_rate))),
        EffectType::Delay => Some(Box::new(Delay::new(sample_rate))),
        EffectType::Distortion => Some(Box::new(Distortion::new())),
        // Reverb is more expensive, use a simpler version or skip
        EffectType::Reverb => None, 
        EffectType::Phaser => None,
        EffectType::Flanger => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::{Effect, EffectProcessor, EffectType};
    use crate::effects::Delay;
    use crate::effects::Saturation;
    
    #[test]
    fn test_track_effect_slot_creation() {
        let slot = TrackEffectSlot::new();
        assert!(slot.is_empty());
        assert!(!slot.is_enabled());
        assert_eq!(slot.mix(), 0.5);
    }
    
    #[test]
    fn test_track_effect_slot_with_effect() {
        let delay = Delay::new(44100.0);
        let slot = TrackEffectSlot::with_effect(delay);
        
        assert!(!slot.is_empty());
        assert!(slot.is_enabled());
        assert_eq!(slot.effect_type(), Some(EffectType::Delay));
    }
    
    #[test]
    fn test_track_effect_slot_defaults() {
        let slot = TrackEffectSlot::default();
        assert!(slot.is_empty());
        assert!(!slot.is_enabled());
        assert_eq!(slot.mix(), 0.5);
        assert!(slot.param_locks().is_empty());
    }
    
    #[test]
    fn test_track_effect_slot_mix() {
        let mut slot = TrackEffectSlot::new();
        
        slot.set_mix(0.3);
        assert_eq!(slot.mix(), 0.3);
        
        // Mix should be clamped
        slot.set_mix(1.5);
        assert_eq!(slot.mix(), 1.0);
        
        slot.set_mix(-0.5);
        assert_eq!(slot.mix(), 0.0);
    }
    
    #[test]
    fn test_track_effect_slot_enabled() {
        let mut slot = TrackEffectSlot::new();
        
        assert!(!slot.is_enabled());
        
        slot.set_enabled(true);
        assert!(!slot.is_enabled()); // Still no effect
        
        let sat = Saturation::new();
        let slot = TrackEffectSlot::with_effect(sat);
        assert!(slot.is_enabled());
        
        let mut slot = slot;
        slot.set_enabled(false);
        assert!(!slot.is_enabled());
    }
    
    #[test]
    fn test_track_effect_slot_process() {
        let delay = Delay::new(44100.0);
        let mut slot = TrackEffectSlot::with_effect(delay);
        
        // Process should work
        let output = slot.process(0.5);
        assert!(output.abs() <= 1.0);
        
        // Disabled slot should pass through
        slot.set_enabled(false);
        let output = slot.process(0.5);
        assert_eq!(output, 0.5);
    }
    
    #[test]
    fn test_track_effect_slot_param_locks() {
        let mut slot = TrackEffectSlot::new();
        
        slot.add_param_lock(EffectParameterId::Mix, 0.8);
        slot.add_param_lock(EffectParameterId::Custom(1), 0.5);
        
        assert_eq!(slot.param_locks().len(), 2);
        assert_eq!(slot.param_locks().get(&EffectParameterId::Mix), Some(&0.8));
        
        slot.remove_param_lock(&EffectParameterId::Mix);
        assert_eq!(slot.param_locks().len(), 1);
        
        slot.clear_param_locks();
        assert!(slot.param_locks().is_empty());
    }
    
    #[test]
    fn test_track_effect_slot_to_config() {
        let delay = Delay::new(44100.0);
        let slot = TrackEffectSlot::with_effect(delay);
        
        let config = slot.to_config();
        assert_eq!(config.effect_type, Some(EffectType::Delay));
        assert!(config.enabled);
        assert_eq!(config.mix, 0.5);
    }
    
    #[test]
    fn test_track_effect_slot_from_config() {
        let mut config = TrackEffectSlotConfig::default();
        config.effect_type = Some(EffectType::Delay);
        config.enabled = true;
        config.mix = 0.7;
        
        let slot = TrackEffectSlot::from_config(&config, 44100.0);
        
        assert!(!slot.is_empty());
        assert!(slot.is_enabled());
        assert_eq!(slot.mix(), 0.7);
        assert_eq!(slot.effect_type(), Some(EffectType::Delay));
    }
    
    #[test]
    fn test_create_effect_instance() {
        // Test supported effect types
        assert!(create_effect_instance(EffectType::Delay, 44100.0).is_some());
        assert!(create_effect_instance(EffectType::Distortion, 44100.0).is_some());
        assert!(create_effect_instance(EffectType::Saturation, 44100.0).is_some());
        assert!(create_effect_instance(EffectType::Compressor, 44100.0).is_some());
        assert!(create_effect_instance(EffectType::SimpleEQ, 44100.0).is_some());
        assert!(create_effect_instance(EffectType::Chorus, 44100.0).is_some());
        
        // Test unsupported effect types (return None)
        assert!(create_effect_instance(EffectType::Reverb, 44100.0).is_none());
        assert!(create_effect_instance(EffectType::Phaser, 44100.0).is_none());
        assert!(create_effect_instance(EffectType::Flanger, 44100.0).is_none());
    }
}

// ============================================================================
// TrackEffects - Effects chain for a single track
// ============================================================================

/// Maximum number of effect slots per track
pub const MAX_EFFECT_SLOTS: usize = 4;

/// Errors that can occur when manipulating track effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackEffectsError {
    /// Invalid slot index
    InvalidSlotIndex,
    
    /// Slot is already occupied
    SlotOccupied,
    
    /// Slot is empty
    SlotEmpty,
    
    /// Effect type not supported
    UnsupportedEffect,
}

/// Effects chain for a single track
#[derive(Debug, Clone)]
pub struct TrackEffects {
    /// Effect slots (None = empty)
    slots: [Option<TrackEffectSlot>; MAX_EFFECT_SLOTS],
    
    /// Track ID
    track_id: u8,
    
    /// Global enable for this track's effects
    enabled: bool,
    
    /// Bypass all effects
    bypass: bool,
    
    /// Sample rate
    sample_rate: f32,
}

impl TrackEffects {
    /// Creates a new empty effects chain for a track
    pub fn new(track_id: u8, sample_rate: f32) -> Self {
        Self {
            slots: [None, None, None, None],
            track_id,
            enabled: true,
            bypass: false,
            sample_rate,
        }
    }
    
    /// Creates from configuration
    pub fn from_config(track_id: u8, sample_rate: f32, configs: &[Option<TrackEffectSlotConfig>]) -> Self {
        let mut effects = Self::new(track_id, sample_rate);
        
        for (i, config) in configs.iter().enumerate().take(MAX_EFFECT_SLOTS) {
            if let Some(ref config) = config {
                if let Some(effect_type) = config.effect_type {
                    let _ = effects.add_effect(i, effect_type); // Ignore errors for now
                    if let Some(ref mut slot) = effects.slots[i] {
                        slot.enabled = config.enabled;
                        slot.mix = config.mix;
                        slot.param_locks = config.param_locks.clone();
                    }
                }
            }
        }
        
        effects
    }
    
    /// Gets the track ID
    pub fn track_id(&self) -> u8 {
        self.track_id
    }
    
    /// Checks if effects are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled && !self.bypass
    }
    
    /// Enables or disables all effects for this track
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Sets bypass mode
    pub fn set_bypass(&mut self, bypass: bool) {
        self.bypass = bypass;
    }
    
    /// Adds an effect to a slot
    pub fn add_effect(&mut self, slot_index: usize, effect_type: EffectType) -> Result<(), TrackEffectsError> {
        if slot_index >= MAX_EFFECT_SLOTS {
            return Err(TrackEffectsError::InvalidSlotIndex);
        }
        
        if self.slots[slot_index].is_some() {
            return Err(TrackEffectsError::SlotOccupied);
        }
        
        if let Some(effect) = create_effect_instance(effect_type, self.sample_rate) {
            self.slots[slot_index] = Some(TrackEffectSlot::with_effect(effect));
            Ok(())
        } else {
            Err(TrackEffectsError::UnsupportedEffect)
        }
    }
    
    /// Removes an effect from a slot
    pub fn remove_effect(&mut self, slot_index: usize) -> Result<(), TrackEffectsError> {
        if slot_index >= MAX_EFFECT_SLOTS {
            return Err(TrackEffectsError::InvalidSlotIndex);
        }
        
        if self.slots[slot_index].is_none() {
            return Err(TrackEffectsError::SlotEmpty);
        }
        
        self.slots[slot_index] = None;
        Ok(())
    }
    
    /// Gets the effect type in a slot
    pub fn effect_type(&self, slot_index: usize) -> Result<Option<EffectType>, TrackEffectsError> {
        if slot_index >= MAX_EFFECT_SLOTS {
            return Err(TrackEffectsError::InvalidSlotIndex);
        }
        
        Ok(self.slots[slot_index].as_ref().and_then(|s| s.effect_type()))
    }
    
    /// Gets the number of active effects
    pub fn active_effect_count(&self) -> usize {
        self.slots.iter().filter(|s| s.as_ref().map(|s| s.is_enabled()).unwrap_or(false)).count()
    }
    
    /// Processes a sample through the entire effects chain
    pub fn process(&mut self, input: f32) -> f32 {
        if !self.is_enabled() {
            return input;
        }
        
        let mut output = input;
        
        for slot in &mut self.slots {
            if let Some(ref mut effect_slot) = slot {
                if effect_slot.is_enabled() {
                    output = effect_slot.process(output);
                }
            }
        }
        
        output
    }
    
    /// Processes a buffer of samples
    pub fn process_buffer(&mut self, samples: &mut [f32]) {
        if !self.is_enabled() {
            return;
        }
        
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }
    
    /// Resets all effects
    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            if let Some(ref mut effect_slot) = slot {
                effect_slot.reset();
            }
        }
    }
    
    /// Gets the mix for a slot
    pub fn mix(&self, slot_index: usize) -> Result<f32, TrackEffectsError> {
        if slot_index >= MAX_EFFECT_SLOTS {
            return Err(TrackEffectsError::InvalidSlotIndex);
        }
        
        self.slots[slot_index]
            .as_ref()
            .map(|s| s.mix())
            .ok_or(TrackEffectsError::SlotEmpty)
    }
    
    /// Sets the mix for a slot
    pub fn set_mix(&mut self, slot_index: usize, mix: f32) -> Result<(), TrackEffectsError> {
        if slot_index >= MAX_EFFECT_SLOTS {
            return Err(TrackEffectsError::InvalidSlotIndex);
        }
        
        if let Some(ref mut slot) = self.slots[slot_index] {
            slot.set_mix(mix);
            Ok(())
        } else {
            Err(TrackEffectsError::SlotEmpty)
        }
    }
    
    /// Enables or disables a specific effect slot
    pub fn set_slot_enabled(&mut self, slot_index: usize, enabled: bool) -> Result<(), TrackEffectsError> {
        if slot_index >= MAX_EFFECT_SLOTS {
            return Err(TrackEffectsError::InvalidSlotIndex);
        }
        
        if let Some(ref mut slot) = self.slots[slot_index] {
            slot.set_enabled(enabled);
            Ok(())
        } else {
            Err(TrackEffectsError::SlotEmpty)
        }
    }
    
    /// Adds a parameter lock to a slot
    pub fn add_param_lock(
        &mut self,
        slot_index: usize,
        param: EffectParameterId,
        value: f32,
    ) -> Result<(), TrackEffectsError> {
        if slot_index >= MAX_EFFECT_SLOTS {
            return Err(TrackEffectsError::InvalidSlotIndex);
        }
        
        if let Some(ref mut slot) = self.slots[slot_index] {
            slot.add_param_lock(param, value);
            Ok(())
        } else {
            Err(TrackEffectsError::SlotEmpty)
        }
    }
    
    /// Applies parameter locks for a given step
    pub fn apply_param_locks(&mut self, step: u8) {
        // This will be implemented when integrating with the parameter lock system
        // For now, just iterate through slots
        for slot in &mut self.slots {
            if let Some(ref mut effect_slot) = slot {
                for (param, value) in effect_slot.param_locks().clone() {
                    // TODO: Apply parameter lock value
                    // This requires integrating with specific effect parameter setters
                    let _ = (param, value, step);
                }
            }
        }
    }
    
    /// Converts to configurations for serialization
    pub fn to_configs(&self) -> [Option<TrackEffectSlotConfig>; MAX_EFFECT_SLOTS] {
        let mut configs: [Option<TrackEffectSlotConfig>; MAX_EFFECT_SLOTS] = [None, None, None, None];
        
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(ref effect_slot) = slot {
                configs[i] = Some(effect_slot.to_config());
            }
        }
        
        configs
    }
}

impl Default for TrackEffects {
    fn default() -> Self {
        Self::new(0, 44100.0)
    }
}

#[cfg(test)]
mod track_effects_tests {
    use super::*;
    use crate::effects::Delay;
    use crate::effects::Distortion;
    
    #[test]
    fn test_track_effects_creation() {
        let effects = TrackEffects::new(0, 44100.0);
        
        assert_eq!(effects.track_id(), 0);
        assert!(effects.is_enabled());
        assert!(!effects.bypass);
        assert_eq!(effects.active_effect_count(), 0);
    }
    
    #[test]
    fn test_track_effects_from_config() {
        let mut configs: [Option<TrackEffectSlotConfig>; 4] = [None, None, None, None];
        
        let mut config = TrackEffectSlotConfig::default();
        config.effect_type = Some(EffectType::Delay);
        config.enabled = true;
        config.mix = 0.7;
        configs[0] = Some(config);
        
        let effects = TrackEffects::from_config(3, 44100.0, &configs);
        
        assert_eq!(effects.track_id(), 3);
        assert_eq!(effects.active_effect_count(), 1);
        assert_eq!(effects.effect_type(0), Ok(Some(EffectType::Delay)));
    }
    
    #[test]
    fn test_track_effects_add_effect() {
        let mut effects = TrackEffects::new(1, 44100.0);
        
        let result = effects.add_effect(0, EffectType::Delay);
        assert!(result.is_ok());
        assert_eq!(effects.active_effect_count(), 1);
        assert_eq!(effects.effect_type(0), Ok(Some(EffectType::Delay)));
        
        // Add another effect
        let result = effects.add_effect(1, EffectType::Distortion);
        assert!(result.is_ok());
        assert_eq!(effects.active_effect_count(), 2);
    }
    
    #[test]
    fn test_track_effects_add_effect_errors() {
        let mut effects = TrackEffects::new(0, 44100.0);
        
        // Invalid slot index
        let result = effects.add_effect(10, EffectType::Delay);
        assert_eq!(result, Err(TrackEffectsError::InvalidSlotIndex));
        
        // Slot already occupied
        effects.add_effect(0, EffectType::Delay).unwrap();
        let result = effects.add_effect(0, EffectType::Distortion);
        assert_eq!(result, Err(TrackEffectsError::SlotOccupied));
        
        // Unsupported effect
        let result = effects.add_effect(1, EffectType::Reverb);
        assert_eq!(result, Err(TrackEffectsError::UnsupportedEffect));
    }
    
    #[test]
    fn test_track_effects_remove_effect() {
        let mut effects = TrackEffects::new(0, 44100.0);
        
        effects.add_effect(0, EffectType::Delay).unwrap();
        assert_eq!(effects.active_effect_count(), 1);
        
        let result = effects.remove_effect(0);
        assert!(result.is_ok());
        assert_eq!(effects.active_effect_count(), 0);
        assert_eq!(effects.effect_type(0), Ok(None));
        
        // Remove from empty slot
        let result = effects.remove_effect(0);
        assert_eq!(result, Err(TrackEffectsError::SlotEmpty));
    }
    
    #[test]
    fn test_track_effects_process() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        
        // Process should work
        let output = effects.process(0.5);
        assert!(output.abs() <= 1.0);
        
        // Disabled track should pass through
        effects.set_enabled(false);
        let output = effects.process(0.5);
        assert_eq!(output, 0.5);
    }
    
    #[test]
    fn test_track_effects_bypass() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        
        // Enabled and not bypassed
        assert!(effects.is_enabled());
        
        // Set bypass
        effects.set_bypass(true);
        assert!(!effects.is_enabled());
        let output = effects.process(0.5);
        assert_eq!(output, 0.5);
    }
    
    #[test]
    fn test_track_effects_mix() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        
        // Get mix
        let mix = effects.mix(0);
        assert_eq!(mix, Ok(0.5));
        
        // Set mix
        let result = effects.set_mix(0, 0.8);
        assert!(result.is_ok());
        assert_eq!(effects.mix(0), Ok(0.8));
        
        // Set mix on empty slot
        let result = effects.set_mix(1, 0.5);
        assert_eq!(result, Err(TrackEffectsError::SlotEmpty));
    }
    
    #[test]
    fn test_track_effects_slot_enabled() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        effects.add_effect(1, EffectType::Distortion).unwrap();
        
        // Disable slot 1
        let result = effects.set_slot_enabled(1, false);
        assert!(result.is_ok());
        
        // Only slot 0 should be active
        assert_eq!(effects.active_effect_count(), 1);
    }
    
    #[test]
    fn test_track_effects_param_locks() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        
        let result = effects.add_param_lock(0, EffectParameterId::Mix, 0.8);
        assert!(result.is_ok());
        
        // Add to empty slot
        let result = effects.add_param_lock(1, EffectParameterId::Mix, 0.5);
        assert_eq!(result, Err(TrackEffectsError::SlotEmpty));
    }
    
    #[test]
    fn test_track_effects_reset() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        effects.add_effect(1, EffectType::Distortion).unwrap();
        
        // Process some samples to change state
        for _ in 0..100 {
            effects.process(0.8);
        }
        
        // Reset should not change structure
        effects.reset();
        assert_eq!(effects.active_effect_count(), 2);
    }
    
    #[test]
    fn test_track_effects_to_configs() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        effects.add_effect(1, EffectType::Distortion).unwrap();
        effects.set_mix(0, 0.7).unwrap();
        effects.set_mix(1, 0.3).unwrap();
        
        let configs = effects.to_configs();
        
        assert!(configs[0].is_some());
        assert!(configs[1].is_some());
        assert!(configs[2].is_none());
        assert!(configs[3].is_none());
        
        assert_eq!(configs[0].as_ref().unwrap().mix, 0.7);
        assert_eq!(configs[1].as_ref().unwrap().mix, 0.3);
    }
    
    #[test]
    fn test_track_effects_multi_slot_processing() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        effects.add_effect(1, EffectType::Distortion).unwrap();
        
        // Process should go through both effects
        let output = effects.process(0.5);
        assert!(output.abs() <= 1.0);
        
        // If we disable first slot, still process second
        effects.set_slot_enabled(0, false).unwrap();
        let output = effects.process(0.5);
        assert!(output.abs() <= 1.0);
    }
    
    #[test]
    fn test_track_effects_process_buffer() {
        let mut effects = TrackEffects::new(0, 44100.0);
        effects.add_effect(0, EffectType::Delay).unwrap();
        
        let mut buffer = [0.5, 0.3, 0.7, 0.4];
        effects.process_buffer(&mut buffer);
        
        // Buffer should be processed
        for &sample in &buffer {
            assert!(sample.abs() <= 1.0);
        }
    }
}

// ============================================================================
// EffectFactory - Factory for creating effect instances
// ============================================================================

/// Builder function type for creating effects
type EffectBuilder = Box<dyn Fn(f32) -> Option<Box<dyn Effect>> + Send + Sync>;

/// Factory for creating and managing effect instances
#[derive(Debug, Clone)]
pub struct EffectFactory {
    /// Registered effect builders
    builders: HashMap<EffectType, EffectBuilder>,
    
    /// Sample rate for new effects
    sample_rate: f32,
}

impl EffectFactory {
    /// Creates a new effect factory
    pub fn new(sample_rate: f32) -> Self {
        let mut factory = Self {
            builders: HashMap::new(),
            sample_rate,
        };
        
        // Register default effects
        factory.register_default_effects();
        
        factory
    }
    
    /// Registers all default effect types
    fn register_default_effects(&mut self) {
        // Use register_custom for effects that need custom creation
        // Delay
        self.register_custom(EffectType::Delay, Box::new(|sr| {
            Some(Box::new(Delay::new(sr)))
        }));
        
        // Distortion
        self.register_custom(EffectType::Distortion, Box::new(|_sr| {
            Some(Box::new(Distortion::new()))
        }));
        
        // Saturation
        self.register_custom(EffectType::Saturation, Box::new(|_sr| {
            Some(Box::new(Saturation::new()))
        }));
        
        // Compressor
        self.register_custom(EffectType::Compressor, Box::new(|sr| {
            Some(Box::new(Compressor::new(sr)))
        }));
        
        // SimpleEQ
        self.register_custom(EffectType::SimpleEQ, Box::new(|sr| {
            Some(Box::new(SimpleEq::new(sr)))
        }));
        
        // Chorus
        self.register_custom(EffectType::Chorus, Box::new(|sr| {
            Some(Box::new(Chorus::new(sr)))
        }));
    }
    
    /// Registers a new effect type
    pub fn register<E: Effect + Default + 'static>(
        &mut self,
        effect_type: EffectType,
    ) {
        self.register_custom(effect_type, Box::new(move |_sr| {
            Some(Box::new(E::default()))
        }));
    }
    
    /// Registers a custom effect builder
    pub fn register_custom(
        &mut self,
        effect_type: EffectType,
        builder: EffectBuilder,
    ) {
        self.builders.insert(effect_type, builder);
    }
    
    /// Creates an effect instance
    pub fn create_effect(&self, effect_type: EffectType) -> Option<Box<dyn Effect>> {
        self.builders
            .get(&effect_type)
            .and_then(|builder| builder(self.sample_rate))
    }
    
    /// Checks if an effect type is registered
    pub fn is_registered(&self, effect_type: EffectType) -> bool {
        self.builders.contains_key(&effect_type)
    }
    
    /// Gets all registered effect types
    pub fn registered_types(&self) -> Vec<EffectType> {
        self.builders.keys().cloned().collect()
    }
    
    /// Gets the sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
    
    /// Sets the sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl Default for EffectFactory {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

#[cfg(test)]
mod effect_factory_tests {
    use super::*;
    
    #[test]
    fn test_effect_factory_creation() {
        let factory = EffectFactory::new(44100.0);
        
        assert_eq!(factory.sample_rate(), 44100.0);
    }
    
    #[test]
    fn test_effect_factory_create_effects() {
        let factory = EffectFactory::new(44100.0);
        
        // Test creating various effects
        assert!(factory.create_effect(EffectType::Delay).is_some());
        assert!(factory.create_effect(EffectType::Distortion).is_some());
        assert!(factory.create_effect(EffectType::Saturation).is_some());
        assert!(factory.create_effect(EffectType::Compressor).is_some());
        assert!(factory.create_effect(EffectType::SimpleEQ).is_some());
        assert!(factory.create_effect(EffectType::Chorus).is_some());
        assert!(factory.create_effect(EffectType::Filter).is_some());
        
        // Test unregistered effects
        assert!(factory.create_effect(EffectType::Reverb).is_none());
        assert!(factory.create_effect(EffectType::Phaser).is_none());
        assert!(factory.create_effect(EffectType::Flanger).is_none());
    }
    
    #[test]
    fn test_effect_factory_is_registered() {
        let factory = EffectFactory::new(44100.0);
        
        assert!(factory.is_registered(EffectType::Delay));
        assert!(factory.is_registered(EffectType::Distortion));
        assert!(!factory.is_registered(EffectType::Reverb));
        assert!(!factory.is_registered(EffectType::Phaser));
    }
    
    #[test]
    fn test_effect_factory_registered_types() {
        let factory = EffectFactory::new(44100.0);
        let types = factory.registered_types();
        
        assert!(types.contains(&EffectType::Delay));
        assert!(types.contains(&EffectType::Distortion));
        assert!(types.contains(&EffectType::Saturation));
        assert!(types.contains(&EffectType::Compressor));
        assert!(types.contains(&EffectType::SimpleEQ));
        assert!(types.contains(&EffectType::Chorus));
        assert!(types.contains(&EffectType::Filter));
        assert!(!types.contains(&EffectType::Reverb));
    }
    
    #[test]
    fn test_effect_factory_custom_register() {
        let mut factory = EffectFactory::new(44100.0);
        
        // Register a custom builder
        factory.register_custom(EffectType::Reverb, Box::new(|_sr| {
            None // Still returns None, but registered
        }));
        
        assert!(factory.is_registered(EffectType::Reverb));
    }
    
    #[test]
    fn test_effect_factory_sample_rate() {
        let mut factory = EffectFactory::new(44100.0);
        
        factory.set_sample_rate(48000.0);
        assert_eq!(factory.sample_rate(), 48000.0);
    }
}

// ============================================================================
// PerTrackEffectsManager - Manages all track effects
// ============================================================================

/// Number of tracks in the step sequencer
pub const TRACK_COUNT: usize = 8;

/// Manager for all track effects
#[derive(Debug, Clone)]
pub struct PerTrackEffectsManager {
    /// Effects for each track
    track_effects: [TrackEffects; TRACK_COUNT],
    
    /// Effect factory
    factory: EffectFactory,
    
    /// Global bypass
    global_bypass: bool,
    
    /// Sample rate
    sample_rate: f32,
}

impl PerTrackEffectsManager {
    /// Creates a new effects manager
    pub fn new(sample_rate: f32) -> Self {
        let mut track_effects = [TrackEffects::new(0, sample_rate); TRACK_COUNT];
        
        // Initialize each track with correct ID
        for (i, track) in track_effects.iter_mut().enumerate() {
            *track = TrackEffects::new(i as u8, sample_rate);
        }
        
        Self {
            track_effects,
            factory: EffectFactory::new(sample_rate),
            global_bypass: false,
            sample_rate,
        }
    }
    
    /// Creates from track configurations
    pub fn from_track_configs(
        sample_rate: f32,
        track_configs: &[Option<[Option<TrackEffectSlotConfig>; MAX_EFFECT_SLOTS]>; TRACK_COUNT],
    ) -> Self {
        let mut manager = Self::new(sample_rate);
        
        for (track_id, config) in track_configs.iter().enumerate().take(TRACK_COUNT) {
            if let Some(ref configs) = config {
                manager.track_effects[track_id] = TrackEffects::from_config(
                    track_id as u8,
                    sample_rate,
                    configs,
                );
            }
        }
        
        manager
    }
    
    /// Gets effects for a specific track
    pub fn track_effects(&mut self, track_id: u8) -> Option<&mut TrackEffects> {
        if track_id < TRACK_COUNT as u8 {
            Some(&mut self.track_effects[track_id as usize])
        } else {
            None
        }
    }
    
    /// Processes audio for a specific track
    pub fn process_track(&mut self, track_id: u8, input: f32) -> f32 {
        if self.global_bypass {
            return input;
        }
        
        if let Some(track) = self.track_effects(track_id) {
            track.process(input)
        } else {
            input
        }
    }
    
    /// Processes a buffer for a specific track
    pub fn process_track_buffer(&mut self, track_id: u8, samples: &mut [f32]) {
        if self.global_bypass {
            return;
        }
        
        if let Some(track) = self.track_effects(track_id) {
            track.process_buffer(samples);
        }
    }
    
    /// Sets global bypass
    pub fn set_global_bypass(&mut self, bypass: bool) {
        self.global_bypass = bypass;
    }
    
    /// Checks if global bypass is enabled
    pub fn is_global_bypass(&self) -> bool {
        self.global_bypass
    }
    
    /// Gets the factory
    pub fn factory(&self) -> &EffectFactory {
        &self.factory
    }
    
    /// Gets a mutable factory
    pub fn factory_mut(&mut self) -> &mut EffectFactory {
        &mut self.factory
    }
    
    /// Gets the sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
    
    /// Resets all effects
    pub fn reset(&mut self) {
        for track in &mut self.track_effects {
            track.reset();
        }
    }
    
    /// Gets total number of active effects
    pub fn total_active_effects(&self) -> usize {
        self.track_effects
            .iter()
            .map(|t| t.active_effect_count())
            .sum()
    }
    
    /// Gets all effect types in use
    pub fn active_effect_types(&self) -> Vec<EffectType> {
        let mut types = Vec::new();
        
        for track in &self.track_effects {
            for slot in 0..MAX_EFFECT_SLOTS {
                if let Ok(Some(effect_type)) = track.effect_type(slot) {
                    if !types.contains(&effect_type) {
                        types.push(effect_type);
                    }
                }
            }
        }
        
        types
    }
}

impl Default for PerTrackEffectsManager {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

#[cfg(test)]
mod per_track_effects_manager_tests {
    use super::*;
    
    #[test]
    fn test_per_track_effects_manager_creation() {
        let manager = PerTrackEffectsManager::new(44100.0);
        
        assert_eq!(manager.sample_rate(), 44100.0);
        assert!(!manager.is_global_bypass());
        assert_eq!(manager.total_active_effects(), 0);
    }
    
    #[test]
    fn test_per_track_effects_process_track() {
        let mut manager = PerTrackEffectsManager::new(44100.0);
        
        // Add effect to track 0
        if let Some(track) = manager.track_effects(0) {
            let _ = track.add_effect(0, EffectType::Delay);
        }
        
        // Process should work
        let output = manager.process_track(0, 0.5);
        assert!(output.abs() <= 1.0);
        
        // Global bypass should pass through
        manager.set_global_bypass(true);
        let output = manager.process_track(0, 0.5);
        assert_eq!(output, 0.5);
    }
    
    #[test]
    fn test_per_track_effects_process_track_buffer() {
        let mut manager = PerTrackEffectsManager::new(44100.0);
        
        // Add effect to track 0
        if let Some(track) = manager.track_effects(0) {
            let _ = track.add_effect(0, EffectType::Delay);
        }
        
        let mut buffer = [0.5, 0.3, 0.7, 0.4];
        manager.process_track_buffer(0, &mut buffer);
        
        for &sample in &buffer {
            assert!(sample.abs() <= 1.0);
        }
    }
    
    #[test]
    fn test_per_track_effects_invalid_track() {
        let mut manager = PerTrackEffectsManager::new(44100.0);
        
        // Invalid track should pass through
        let output = manager.process_track(99, 0.5);
        assert_eq!(output, 0.5);
    }
    
    #[test]
    fn test_per_track_effects_reset() {
        let mut manager = PerTrackEffectsManager::new(44100.0);
        
        // Add effects to multiple tracks
        for track_id in 0..8 {
            if let Some(track) = manager.track_effects(track_id) {
                let _ = track.add_effect(0, EffectType::Delay);
            }
        }
        
        assert_eq!(manager.total_active_effects(), 8);
        
        manager.reset();
        
        // Reset should keep effects but reset their state
        assert_eq!(manager.total_active_effects(), 8);
    }
    
    #[test]
    fn test_per_track_effects_active_effect_types() {
        let mut manager = PerTrackEffectsManager::new(44100.0);
        
        // No effects yet
        assert!(manager.active_effect_types().is_empty());
        
        // Add effects
        if let Some(track) = manager.track_effects(0) {
            let _ = track.add_effect(0, EffectType::Delay);
        }
        if let Some(track) = manager.track_effects(1) {
            let _ = track.add_effect(0, EffectType::Distortion);
        }
        
        let types = manager.active_effect_types();
        
        assert!(types.contains(&EffectType::Delay));
        assert!(types.contains(&EffectType::Distortion));
        assert_eq!(types.len(), 2);
    }
    
    #[test]
    fn test_per_track_effects_multi_track() {
        let mut manager = PerTrackEffectsManager::new(44100.0);
        
        // Add different effects to different tracks
        for track_id in 0..8 {
            if let Some(track) = manager.track_effects(track_id) {
                let effect_type = match track_id % 3 {
                    0 => EffectType::Delay,
                    1 => EffectType::Distortion,
                    _ => EffectType::Compressor,
                };
                let _ = track.add_effect(0, effect_type);
            }
        }
        
        assert_eq!(manager.total_active_effects(), 8);
        
        // Process each track
        for track_id in 0..8 {
            let output = manager.process_track(track_id, 0.5);
            assert!(output.abs() <= 1.0, "Track {} processing failed", track_id);
        }
    }
    
    #[test]
    fn test_per_track_effects_factory() {
        let mut manager = PerTrackEffectsManager::new(44100.0);
        
        // Factory should be accessible
        assert!(manager.factory().is_registered(EffectType::Delay));
        
        // Factory should be mutable
        manager.factory_mut().set_sample_rate(48000.0);
        assert_eq!(manager.factory().sample_rate(), 48000.0);
    }
    
    #[test]
    fn test_per_track_effects_from_configs() {
        let mut track_configs: [Option<[Option<TrackEffectSlotConfig>; 4]>; 8] = [None; 8];
        
        // Configure track 0
        let mut config = TrackEffectSlotConfig::default();
        config.effect_type = Some(EffectType::Delay);
        config.enabled = true;
        config.mix = 0.7;
        track_configs[0] = Some([Some(config), None, None, None]);
        
        let manager = PerTrackEffectsManager::from_track_configs(44100.0, &track_configs);
        
        assert!(manager.track_effects(0).is_some());
        if let Some(track) = manager.track_effects(0) {
            assert_eq!(track.active_effect_count(), 1);
        }
    }
}
