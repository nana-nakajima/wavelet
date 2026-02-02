#![cfg(feature = "gdext")]

//! GDExtension Bindings Module
//!
//! This module provides the Godot 4 GDExtension bindings for WAVELET.
//! It allows the Rust synthesizer to be used as a native Godot extension.
//!
//! # GDExtension Setup
//!
//! To use WAVELET in Godot 4:
//!
//! 1. Build the Rust library: `cargo build --release`
//! 2. Copy the resulting `.dylib`/`.so` to the godot project
//! 3. Add the GDExtension registration code
//! 4. Use the WAVELET node in Godot scenes

use godot::prelude::*;
use crate::synth::Synth;

/// WAVELET synthesizer node for Godot 4.
///
/// This node provides the main interface for using WAVELET in Godot.
/// It exposes properties and methods for controlling the synthesizer.
///
/// # Godot Usage
///
/// ```gdscript
/// extends WaveletSynth
///
/// func _ready():
///     note_on(60, 127)  # Play C4
///     await get_tree().create_timer(0.5).timeout
///     note_off(60)      # Release C4
/// ```
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct WaveletSynth {
    /// Internal synthesizer instance
    synth: Synth,
    
    /// Base class reference
    base: Base<Node>,
    
    /// Currently held notes
    held_notes: Vec<u8>,
}

#[godot_api]
impl INode for WaveletSynth {
    /// Initializes the WAVELET synthesizer node.
    fn init(base: Base<Node>) -> Self {
        Self {
            synth: Synth::default(),
            base,
            held_notes: Vec::new(),
        }
    }
    
    /// Called when the node enters the scene tree.
    fn ready(&mut self) {
        godot_print!("WAVELET Synthesizer initialized!");
    }
}

#[godot_api]
impl WaveletSynth {
    /// Plays a note.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    /// * `velocity` - Note velocity (0-127)
    #[func]
    pub fn note_on(&mut self, note: i32, velocity: i32) {
        self.synth.note_on(note as u8, velocity as u8);
        if !self.held_notes.contains(&(note as u8)) {
            self.held_notes.push(note as u8);
        }
    }
    
    /// Releases a note.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    #[func]
    pub fn note_off(&mut self, note: i32) {
        self.synth.note_off_specific(note as u8);
        self.held_notes.retain(|&n| n != note as u8);
    }
    
    /// Releases all held notes.
    #[func]
    pub fn all_notes_off(&mut self) {
        self.synth.note_off();
        self.held_notes.clear();
    }
    
    /// Sets the master volume.
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume (0.0 to 1.0)
    #[func]
    pub fn set_volume(&mut self, volume: f32) {
        self.synth.set_master_volume(volume);
    }
    
    /// Sets the filter cutoff frequency.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff frequency in Hz
    #[func]
    pub fn set_filter_cutoff(&mut self, cutoff: f32) {
        self.synth.set_filter_cutoff(cutoff);
    }
    
    /// Sets the filter resonance.
    ///
    /// # Arguments
    ///
    /// * `resonance` - Q value
    #[func]
    pub fn set_filter_resonance(&mut self, resonance: f32) {
        self.synth.set_filter_resonance(resonance);
    }
    
    /// Loads a preset by name.
    ///
    /// # Arguments
    ///
    /// * `preset_name` - Name of the preset to load
    #[func]
    pub fn load_preset(&mut self, preset_name: GodotString) {
        let name = preset_name.to_string();
        
        match name.as_str() {
            "init" => self.load_init_preset(),
            "bass" => self.load_bass_preset(),
            "pad" => self.load_pad_preset(),
            "lead" => self.load_lead_preset(),
            _ => godot_print!("Unknown preset: {}", name),
        }
    }
    
    /// Loads the init/default preset.
    fn load_init_preset(&mut self) {
        self.synth.set_master_volume(0.7);
        self.synth.set_filter_cutoff(2000.0);
        self.synth.set_filter_resonance(1.0);
    }
    
    /// Loads a bass preset.
    fn load_bass_preset(&mut self) {
        self.synth.set_master_volume(0.8);
        self.synth.set_filter_cutoff(500.0);
        self.synth.set_filter_resonance(3.0);
    }
    
    /// Loads a pad preset.
    fn load_pad_preset(&mut self) {
        self.synth.set_master_volume(0.5);
        self.synth.set_filter_cutoff(3000.0);
        self.synth.set_filter_resonance(0.5);
    }
    
    /// Loads a lead preset.
    fn load_lead_preset(&mut self) {
        self.synth.set_master_volume(0.6);
        self.synth.set_filter_cutoff(1500.0);
        self.synth.set_filter_resonance(2.0);
    }
}

/// Registers all WAVELET classes with Godot.
#[gdextension]
unsafe impl EntryPoint for WaveletSynth {
    fn entry_point(_interface: &gdext::GdextInterface, _library: &gdext::GdextLibrary, _generator: &mut gdext::ClassGenerator) {
        godot_print!("WAVELET GDExtension loaded!");
    }
}
