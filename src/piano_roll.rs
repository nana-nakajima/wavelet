//! Piano Roll Editor - MIDI Note Editing Interface
//!
//! This module provides a piano roll editor for visual MIDI note editing,
//! featuring drag-and-drop note placement, velocity editing, and quantize functions.
//!
//! # Features
//!
//! - Visual note grid (piano keys + time grid)
//! - Drag to create/resize/move notes
//! - Velocity editing per note
//! - Snap-to-grid quantization
//! - Multiple octaves support
//! - Copy/paste patterns
//! - Undo/redo support
//!
//! # Example
//!
//! ```rust
//! use wavelet::piano_roll::{PianoRoll, NoteEvent};
//!
//! let mut pr = PianoRoll::new();
//! pr.set_grid_resolution(Resolution::Sixteenth);
//! pr.add_note(60, 0.0, 1.0, 100); // C4, beat 0-1, velocity 100
//! ```

use std::collections::HashMap;

/// Grid resolution for piano roll
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Quarter = 4,      // 1 note per quarter note
    Eighth = 8,       // 2 notes per quarter
    Sixteenth = 16,   // 4 notes per quarter
    ThirtySecond = 32, // 8 notes per quarter
}

/// Note event for piano roll
#[derive(Debug, Clone, PartialEq)]
pub struct NoteEvent {
    /// MIDI note number (0-127)
    pub note: u8,
    /// Start time in beats
    pub start_beat: f64,
    /// Duration in beats
    pub duration: f64,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Track index (for multi-track)
    pub track: u8,
}

/// Piano roll editor state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PianoRoll {
    /// Current resolution
    resolution: Resolution,
    /// Total beats displayed
    total_beats: f64,
    /// Notes in the piano roll
    notes: Vec<NoteEvent>,
    /// Selected notes
    selected: Vec<usize>,
    /// Current edit mode
    edit_mode: EditMode,
    /// Snap to grid enabled
    snap_enabled: bool,
    /// Quantize value (in steps)
    quantize_steps: u8,
    /// Note length multiplier (for drawing)
    note_length_multiplier: f64,
    /// Copy buffer
    copy_buffer: Vec<NoteEvent>,
    /// Undo history
    undo_stack: Vec<Vec<NoteEvent>>,
    /// Redo history
    redo_stack: Vec<Vec<NoteEvent>>,
    /// Velocity color mode
    velocity_color: bool,
    /// Track colors
    track_colors: HashMap<u8, u32>,
    /// Current octave display range
    octave_range: (u8, u8),
    /// Loop region (start, end in beats)
    loop_region: Option<(f64, f64)>,
    /// Grid color theme
    grid_color: u32,
    /// Note height in pixels
    note_height: f64,
    /// Beat width in pixels
    beat_width: f64,
}

/// Edit mode for piano roll
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// Select notes
    Select,
    /// Draw new notes
    Draw,
    /// Erase notes
    Erase,
    /// Move notes
    Move,
    /// Resize notes
    Resize,
    /// Velocity edit
    Velocity,
}

/// Piano roll configuration
#[derive(Debug, Clone)]
pub struct PianoRollConfig {
    /// Resolution setting
    pub resolution: Resolution,
    /// Total beats to display
    pub total_beats: f64,
    /// Snap to grid
    pub snap_enabled: bool,
    /// Quantize steps
    pub quantize_steps: u8,
    /// Note length multiplier
    pub note_length_multiplier: f64,
    /// Show velocity colors
    pub velocity_color: bool,
    /// Grid color (RGB)
    pub grid_color: u32,
    /// Note height in pixels
    pub note_height: f64,
    /// Beat width in pixels
    pub beat_width: f64,
    /// Loop enabled
    pub loop_enabled: bool,
    /// Loop start beat
    pub loop_start: f64,
    /// Loop end beat
    pub loop_end: f64,
}

impl Default for PianoRollConfig {
    fn default() -> Self {
        Self {
            resolution: Resolution::Sixteenth,
            total_beats: 16.0,
            snap_enabled: true,
            quantize_steps: 16,
            note_length_multiplier: 0.9,
            velocity_color: true,
            grid_color: 0x333333,
            note_height: 12.0,
            beat_width: 40.0,
            loop_enabled: false,
            loop_start: 0.0,
            loop_end: 16.0,
        }
    }
}

impl PianoRoll {
    /// Create a new piano roll editor
    pub fn new() -> Self {
        Self {
            resolution: Resolution::Sixteenth,
            total_beats: 16.0,
            notes: Vec::new(),
            selected: Vec::new(),
            edit_mode: EditMode::Draw,
            snap_enabled: true,
            quantize_steps: 16,
            note_length_multiplier: 0.9,
            copy_buffer: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            velocity_color: true,
            track_colors: HashMap::new(),
            octave_range: (3, 6), // Display C3 to C7
            loop_region: None,
            grid_color: 0x333333,
            note_height: 12.0,
            beat_width: 40.0,
        }
    }

    /// Create with configuration
    pub fn with_config(config: PianoRollConfig) -> Self {
        Self {
            resolution: config.resolution,
            total_beats: config.total_beats,
            notes: Vec::new(),
            selected: Vec::new(),
            edit_mode: EditMode::Draw,
            snap_enabled: config.snap_enabled,
            quantize_steps: config.quantize_steps,
            note_length_multiplier: config.note_length_multiplier,
            copy_buffer: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            velocity_color: config.velocity_color,
            track_colors: HashMap::new(),
            octave_range: (3, 6),
            loop_region: if config.loop_enabled {
                Some((config.loop_start, config.loop_end))
            } else {
                None
            },
            grid_color: config.grid_color,
            note_height: config.note_height,
            beat_width: config.beat_width,
        }
    }

    // ==================== Note Operations ====================

    /// Add a note to the piano roll
    pub fn add_note(&mut self, note: u8, start_beat: f64, duration: f64, velocity: u8) -> usize {
        self.save_undo();
        let note_event = NoteEvent {
            note: note.clamp(0, 127),
            start_beat: self.snap_to_grid(start_beat),
            duration: duration.max(0.0625),
            velocity: velocity.clamp(0, 127),
            track: 0,
        };
        self.notes.push(note_event);
        self.notes.len() - 1
    }

    /// Add a note to a specific track
    pub fn add_note_to_track(
        &mut self,
        note: u8,
        start_beat: f64,
        duration: f64,
        velocity: u8,
        track: u8,
    ) -> usize {
        self.save_undo();
        let note_event = NoteEvent {
            note: note.clamp(0, 127),
            start_beat: self.snap_to_grid(start_beat),
            duration: duration.max(0.0625),
            velocity: velocity.clamp(0, 127),
            track,
        };
        self.notes.push(note_event);
        self.notes.len() - 1
    }

    /// Remove a note by index
    pub fn remove_note(&mut self, index: usize) -> bool {
        if index < self.notes.len() {
            self.save_undo();
            self.notes.remove(index);
            self.selected.retain(|&i| i != index);
            // Adjust selected indices
            for sel in &mut self.selected {
                if *sel > index {
                    *sel -= 1;
                }
            }
            true
        } else {
            false
        }
    }

    /// Remove selected notes
    pub fn remove_selected(&mut self) -> usize {
        self.save_undo();
        let count = self.selected.len();
        self.selected.sort_by(|a, b| b.cmp(a)); // Sort descending
        for &index in &self.selected {
            if index < self.notes.len() {
                self.notes.remove(index);
            }
        }
        self.selected.clear();
        count
    }

    /// Update a note
    pub fn update_note(
        &mut self,
        index: usize,
        note: Option<u8>,
        start_beat: Option<f64>,
        duration: Option<f64>,
        velocity: Option<u8>,
    ) -> bool {
        if index >= self.notes.len() {
            return false;
        }
        self.save_undo();
        let step = self.grid_step();
        let note_event = &mut self.notes[index];
        if let Some(n) = note {
            note_event.note = n.clamp(0, 127);
        }
        if let Some(s) = start_beat {
            if self.snap_enabled {
                note_event.start_beat = (s / step).round() * step;
            } else {
                note_event.start_beat = s;
            }
        }
        if let Some(d) = duration {
            note_event.duration = d.max(0.0625);
        }
        if let Some(v) = velocity {
            note_event.velocity = v.clamp(0, 127);
        }
        true
    }

    /// Move a note
    pub fn move_note(&mut self, index: usize, delta_note: i8, delta_beat: f64) -> bool {
        if index >= self.notes.len() {
            return false;
        }
        self.save_undo();
        let step = self.grid_step();
        let note_event = &mut self.notes[index];
        let new_note = note_event.note as i16 + delta_note as i16;
        note_event.note = new_note.clamp(0, 127) as u8;
        let new_beat = note_event.start_beat + delta_beat;
        if self.snap_enabled {
            note_event.start_beat = (new_beat / step).round() * step;
        } else {
            note_event.start_beat = new_beat;
        }
        true
    }

    /// Resize a note
    pub fn resize_note(&mut self, index: usize, delta_duration: f64) -> bool {
        if index >= self.notes.len() {
            return false;
        }
        self.save_undo();
        let note_event = &mut self.notes[index];
        note_event.duration = (note_event.duration + delta_duration).max(0.0625);
        true
    }

    // ==================== Selection ====================

    /// Select a note
    pub fn select_note(&mut self, index: usize, add_to_selection: bool) -> bool {
        if index < self.notes.len() {
            if !add_to_selection {
                self.selected.clear();
            }
            if !self.selected.contains(&index) {
                self.selected.push(index);
            }
            true
        } else {
            false
        }
    }

    /// Select notes in a rectangle
    pub fn select_rect(&mut self, start_beat: f64, start_note: u8, end_beat: f64, end_note: u8) {
        self.save_undo();
        self.selected.clear();
        let start = start_beat.min(end_beat);
        let end = start_beat.max(end_beat);
        let note_low = start_note.min(end_note);
        let note_high = start_note.max(end_note);

        for (i, note) in self.notes.iter().enumerate() {
            if note.note >= note_low
                && note.note <= note_high
                && note.start_beat >= start
                && note.start_beat <= end
            {
                self.selected.push(i);
            }
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Get selected notes
    pub fn get_selected_notes(&self) -> Vec<&NoteEvent> {
        self.selected
            .iter()
            .filter_map(|&i| self.notes.get(i))
            .collect()
    }

    /// Get selected note indices
    pub fn get_selected_indices(&self) -> Vec<usize> {
        self.selected.clone()
    }

    // ==================== Grid & Quantization ====================

    /// Get grid step size in beats
    pub fn grid_step(&self) -> f64 {
        let res_value = match self.resolution {
            Resolution::Quarter => 4,
            Resolution::Eighth => 8,
            Resolution::Sixteenth => 16,
            Resolution::ThirtySecond => 32,
        };
        1.0 / res_value as f64
    }

    /// Snap a beat value to grid
    pub fn snap_to_grid(&self, beat: f64) -> f64 {
        if !self.snap_enabled {
            return beat;
        }
        let step = self.grid_step();
        (beat / step).round() * step
    }

    /// Quantize selected notes
    pub fn quantize_selected(&mut self) -> usize {
        self.save_undo();
        let step = self.grid_step();
        let mut count = 0;
        for &index in &self.selected {
            if let Some(note) = self.notes.get_mut(index) {
                note.start_beat = (note.start_beat / step).round() * step;
                count += 1;
            }
        }
        count
    }

    /// Set resolution
    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
    }

    /// Get current resolution
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Toggle snap to grid
    pub fn set_snap_enabled(&mut self, enabled: bool) {
        self.snap_enabled = enabled;
    }

    /// Is snap enabled?
    pub fn snap_enabled(&self) -> bool {
        self.snap_enabled
    }

    // ==================== Edit Mode ====================

    /// Set edit mode
    pub fn set_edit_mode(&mut self, mode: EditMode) {
        self.edit_mode = mode;
    }

    /// Get edit mode
    pub fn edit_mode(&self) -> EditMode {
        self.edit_mode
    }

    // ==================== Copy/Paste ====================

    /// Copy selected notes
    pub fn copy_selected(&mut self) {
        self.copy_buffer = self
            .selected
            .iter()
            .filter_map(|&i| self.notes.get(i).cloned())
            .collect();
    }

    /// Cut selected notes
    pub fn cut_selected(&mut self) {
        self.copy_selected();
        self.remove_selected();
    }

    /// Paste notes at position
    pub fn paste_at(&mut self, start_beat: f64) -> Vec<usize> {
        self.save_undo();
        let mut new_indices = Vec::new();
        let offset = if self.copy_buffer.is_empty() {
            0.0
        } else {
            start_beat
                - self
                    .copy_buffer
                    .iter()
                    .map(|n| n.start_beat)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0)
        };

        for note in &self.copy_buffer {
            let new_note = NoteEvent {
                note: note.note,
                start_beat: self.snap_to_grid(note.start_beat + offset),
                duration: note.duration,
                velocity: note.velocity,
                track: note.track,
            };
            self.notes.push(new_note);
            new_indices.push(self.notes.len() - 1);
        }
        new_indices
    }

    /// Duplicate selected notes
    pub fn duplicate_selected(&mut self, offset_beats: f64) -> Vec<usize> {
        self.save_undo();
        let mut new_indices = Vec::new();
        for &index in &self.selected {
            if let Some(note) = self.notes.get(index).cloned() {
                let new_note = NoteEvent {
                    start_beat: self.snap_to_grid(note.start_beat + offset_beats),
                    ..note
                };
                self.notes.push(new_note);
                new_indices.push(self.notes.len() - 1);
            }
        }
        new_indices
    }

    // ==================== Undo/Redo ====================

    /// Save current state to undo stack
    fn save_undo(&mut self) {
        self.undo_stack.push(self.notes.clone());
        self.redo_stack.clear();
        // Limit undo history
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    /// Undo last action
    pub fn undo(&mut self) -> bool {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(self.notes.clone());
            self.notes = state;
            self.selected.clear();
            true
        } else {
            false
        }
    }

    /// Redo last action
    pub fn redo(&mut self) -> bool {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(self.notes.clone());
            self.notes = state;
            self.selected.clear();
            true
        } else {
            false
        }
    }

    /// Can undo?
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Can redo?
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    // ==================== Utilities ====================

    /// Get note name (e.g., "C4", "F#3")
    pub fn note_name(note: u8) -> String {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let octave = (note / 12) as i8 - 1;
        let note_name = note_names[(note % 12) as usize];
        format!("{}{}", note_name, octave)
    }

    /// Check if note is black key
    pub fn is_black_key(note: u8) -> bool {
        matches!(note % 12, 1 | 3 | 6 | 8 | 10)
    }

    /// Get velocity as normalized value (0.0-1.0)
    pub fn velocity_normalized(velocity: u8) -> f64 {
        velocity as f64 / 127.0
    }

    /// Get velocity from normalized value
    pub fn velocity_from_normalized(value: f64) -> u8 {
        (value.clamp(0.0, 1.0) * 127.0) as u8
    }

    /// Get color for velocity
    pub fn velocity_color(velocity: u8) -> u32 {
        // Green to Red gradient based on velocity
        let normalized = velocity as f64 / 127.0;
        let r = (normalized * 255.0) as u8;
        let g = ((1.0 - normalized) * 200.0 + 55.0) as u8;
        let b = 100u8;
        (r as u32) << 16 | (g as u32) << 8 | b as u32
    }

    /// Get color for track
    pub fn track_color(track: u8) -> u32 {
        let colors = [
            0x4ECDC4, // Teal
            0xFF6B6B, // Coral
            0x95E1D3, // Mint
            0xF38181, // Salmon
            0xAA96DA, // Lavender
            0xFCBAD3, // Pink
            0xA8D8EA, // Light Blue
            0xFFB6B9, // Light Coral
        ];
        colors[(track as usize) % colors.len()]
    }

    // ==================== Rendering Helpers ====================

    /// Get piano key width
    pub fn piano_key_width(&self) -> f64 {
        40.0
    }

    /// Get total piano height
    pub fn piano_height(&self) -> f64 {
        let notes = (self.octave_range.1 - self.octave_range.0 + 1) * 12;
        notes as f64 * self.note_height
    }

    /// Get grid width
    pub fn grid_width(&self) -> f64 {
        self.total_beats * self.beat_width
    }

    /// Convert beat to x position
    pub fn beat_to_x(&self, beat: f64) -> f64 {
        beat * self.beat_width
    }

    /// Convert x position to beat
    pub fn x_to_beat(&self, x: f64) -> f64 {
        x / self.beat_width
    }

    /// Convert note to y position
    pub fn note_to_y(&self, note: u8) -> f64 {
        let total_notes = (self.octave_range.1 - self.octave_range.0 + 1) * 12;
        let note_index = total_notes as i16 - 1 - (note as i16 - self.octave_range.0 as i16 * 12);
        note_index as f64 * self.note_height
    }

    /// Convert y position to note
    pub fn y_to_note(&self, y: f64) -> u8 {
        let total_notes = (self.octave_range.1 - self.octave_range.0 + 1) * 12;
        let note_index = total_notes as i16 - 1 - (y / self.note_height) as i16;
        (self.octave_range.0 as i16 * 12 + note_index) as u8
    }

    // ==================== Query Operations ====================

    /// Get notes in time range
    pub fn get_notes_in_range(&self, start_beat: f64, end_beat: f64) -> Vec<&NoteEvent> {
        self.notes
            .iter()
            .filter(|n| n.start_beat >= start_beat && n.start_beat < end_beat)
            .collect()
    }

    /// Get notes in note range
    pub fn get_notes_in_note_range(&self, start_note: u8, end_note: u8) -> Vec<&NoteEvent> {
        self.notes
            .iter()
            .filter(|n| n.note >= start_note && n.note <= end_note)
            .collect()
    }

    /// Get notes on track
    pub fn get_notes_on_track(&self, track: u8) -> Vec<&NoteEvent> {
        self.notes.iter().filter(|n| n.track == track).collect()
    }

    /// Get all notes
    pub fn get_all_notes(&self) -> &[NoteEvent] {
        &self.notes
    }

    /// Get note count
    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }

    /// Clear all notes
    pub fn clear(&mut self) {
        self.save_undo();
        self.notes.clear();
        self.selected.clear();
    }

    // ==================== MIDI Conversion ====================

    /// Convert to MIDI-like format (note, time, duration, velocity)
    pub fn to_midi_format(&self) -> Vec<(u8, u32, u32, u8)> {
        // Convert beats to ticks (assuming 480 ticks per quarter note)
        let ticks_per_beat = 480;
        self.notes
            .iter()
            .map(|n| {
                (
                    n.note,
                    (n.start_beat * ticks_per_beat as f64) as u32,
                    (n.duration * ticks_per_beat as f64) as u32,
                    n.velocity,
                )
            })
            .collect()
    }

    /// Load from MIDI-like format
    pub fn from_midi_format(&mut self, midi_data: &[(u8, u32, u32, u8)]) {
        self.save_undo();
        self.notes.clear();
        let ticks_per_beat = 480;
        for &(note, start, duration, velocity) in midi_data {
            self.notes.push(NoteEvent {
                note,
                start_beat: start as f64 / ticks_per_beat as f64,
                duration: duration as f64 / ticks_per_beat as f64,
                velocity,
                track: 0,
            });
        }
    }

    // ==================== Pattern Operations ====================

    /// Transpose selected notes
    pub fn transpose_selected(&mut self, semitones: i8) -> usize {
        self.save_undo();
        let mut count = 0;
        for &index in &self.selected {
            if let Some(note) = self.notes.get_mut(index) {
                let new_note = note.note as i16 + semitones as i16;
                if new_note >= 0 && new_note <= 127 {
                    note.note = new_note as u8;
                    count += 1;
                }
            }
        }
        count
    }

    /// Velocity fade selected notes (from start to end velocity)
    pub fn velocity_fade_selected(&mut self, start_vel: u8, end_vel: u8) -> usize {
        self.save_undo();
        let mut count = 0;
        for &index in &self.selected {
            if let Some(note) = self.notes.get_mut(index) {
                // Simple linear fade based on position in selection
                let fade = (end_vel as f64 - start_vel as f64)
                    / (self.selected.len().max(1) as f64);
                let pos = self
                    .selected
                    .iter()
                    .position(|&i| i == index)
                    .unwrap_or(0) as f64;
                note.velocity = (start_vel as f64 + fade * pos) as u8;
                count += 1;
            }
        }
        count
    }

    /// Randomize velocity of selected notes
    pub fn randomize_velocity_selected(&mut self, range: u8) -> usize {
        self.save_undo();
        let mut count = 0;
        for &index in &self.selected {
            if let Some(note) = self.notes.get_mut(index) {
                let delta = (rand() as u8 % (range * 2)) as i16 - range as i16;
                let new_vel = note.velocity as i16 + delta;
                note.velocity = new_vel.clamp(0, 127) as u8;
                count += 1;
            }
        }
        count
    }

    /// Legato selected notes
    pub fn legato_selected(&mut self, min_overlap: f64) -> usize {
        self.save_undo();
        // Sort selected notes by start time
        let mut selected_notes: Vec<_> = self
            .selected
            .iter()
            .filter_map(|&i| self.notes.get(i).cloned())
            .collect();
        selected_notes.sort_by(|a, b| a.start_beat.partial_cmp(&b.start_beat).unwrap());

        let mut count = 0;
        for i in 0..selected_notes.len().saturating_sub(1) {
            let current = &selected_notes[i];
            let next = &selected_notes[i + 1];

            // Find and update the actual note in the main array
            for &index in &self.selected {
                if let Some(note) = self.notes.get_mut(index) {
                    if note.start_beat == current.start_beat && note.note == current.note {
                        let target_end = next.start_beat - min_overlap;
                        if target_end > note.start_beat {
                            note.duration = target_end - note.start_beat;
                            count += 1;
                        }
                        break;
                    }
                }
            }
        }
        count
    }

    // ==================== Playback Preview ====================

    /// Get notes for playback at specific time
    pub fn get_notes_at_time(&self, beat: f64) -> Vec<&NoteEvent> {
        self.notes
            .iter()
            .filter(|n| n.start_beat <= beat && n.start_beat + n.duration > beat)
            .collect()
    }

    /// Get first note time
    pub fn first_note_time(&self) -> Option<f64> {
        self.notes
            .iter()
            .map(|n| n.start_beat)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Get last note end time
    pub fn last_note_end_time(&self) -> Option<f64> {
        self.notes
            .iter()
            .map(|n| n.start_beat + n.duration)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

/// Simple random function (替代 rand crate)
fn rand() -> u32 {
    static mut STATE: u64 = 1;
    unsafe {
        STATE = STATE.wrapping_mul(1103515245).wrapping_add(12345);
        (STATE >> 16) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_note() {
        let mut pr = PianoRoll::new();
        let index = pr.add_note(60, 0.0, 1.0, 100);
        assert_eq!(pr.notes.len(), 1);
        assert_eq!(index, 0);
    }

    #[test]
    fn test_snap_to_grid() {
        let mut pr = PianoRoll::new();
        pr.set_resolution(Resolution::Sixteenth);
        // 0.1 is closer to 0.125 than 0.0625 (0.0375 vs 0.025 diff)
        assert!((pr.snap_to_grid(0.1) - 0.125).abs() < 0.001);
        // 0.2 is closer to 0.1875 than 0.125 or 0.25
        assert!((pr.snap_to_grid(0.2) - 0.1875).abs() < 0.001);
        // 0.3 is closer to 0.3125 (which rounds to 0.3125)
        assert!((pr.snap_to_grid(0.3) - 0.3125).abs() < 0.001);
    }

    #[test]
    fn test_note_name() {
        assert_eq!(PianoRoll::note_name(60), "C4");
        assert_eq!(PianoRoll::note_name(61), "C#4");
        assert_eq!(PianoRoll::note_name(62), "D4");
        assert_eq!(PianoRoll::note_name(48), "C3");
    }

    #[test]
    fn test_is_black_key() {
        assert!(!PianoRoll::is_black_key(60)); // C
        assert!(PianoRoll::is_black_key(61)); // C#
        assert!(!PianoRoll::is_black_key(62)); // D
        assert!(PianoRoll::is_black_key(63)); // D#
        assert!(!PianoRoll::is_black_key(64)); // E
    }

    #[test]
    fn test_velocity_color() {
        let low_color = PianoRoll::velocity_color(32);
        let high_color = PianoRoll::velocity_color(120);
        // High velocity should have more red
        let low_r = (low_color >> 16) & 0xFF;
        let high_r = (high_color >> 16) & 0xFF;
        assert!(high_r > low_r);
    }

    #[test]
    fn test_undo_redo() {
        let mut pr = PianoRoll::new();
        pr.add_note(60, 0.0, 1.0, 100);
        assert!(pr.can_undo());
        assert!(!pr.can_redo());

        pr.undo();
        assert!(pr.notes.is_empty());
        assert!(!pr.can_undo());
        assert!(pr.can_redo());

        pr.redo();
        assert_eq!(pr.notes.len(), 1);
        assert!(pr.can_undo());
        assert!(!pr.can_redo());
    }

    #[test]
    fn test_transpose() {
        let mut pr = PianoRoll::new();
        pr.add_note(60, 0.0, 1.0, 100);
        pr.select_note(0, false);
        pr.transpose_selected(2);
        assert_eq!(pr.notes[0].note, 62);
    }

    #[test]
    fn test_midi_conversion() {
        let mut pr = PianoRoll::new();
        pr.add_note(60, 0.0, 1.0, 100);
        let midi = pr.to_midi_format();
        assert_eq!(midi.len(), 1);
        assert_eq!(midi[0].0, 60); // note
        assert_eq!(midi[0].1, 0); // start tick
        assert_eq!(midi[0].2, 480); // duration tick
        assert_eq!(midi[0].3, 100); // velocity
    }

    #[test]
    fn test_clear() {
        let mut pr = PianoRoll::new();
        pr.add_note(60, 0.0, 1.0, 100);
        pr.add_note(62, 1.0, 0.5, 80);
        pr.clear();
        assert!(pr.notes.is_empty());
    }

    #[test]
    fn test_quantize() {
        let mut pr = PianoRoll::new();
        pr.set_resolution(Resolution::Sixteenth);
        pr.add_note(60, 0.05, 0.5, 100); // Slightly off-grid
        pr.select_note(0, false);
        pr.quantize_selected();
        assert!((pr.notes[0].start_beat - 0.0625).abs() < 0.001);
    }
}
