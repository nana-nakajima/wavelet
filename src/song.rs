// WAVELET - Song Mode Module
// Reference: Elektron Tonverk song system
//
// Features:
// - 16 song capacity
// - Up to 99 lines per song
// - Per-line settings: pattern selection, repeat count, length modifier, tempo modifier
// - Pattern chain playback
//
// Comparison with Tonverk:
// Tonverk: 16 songs, up to 99 lines each
// WAVELET: 16 songs, up to 99 lines each (aligned!)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Number of songs available
pub const MAX_SONGS: usize = 16;
/// Maximum number of lines per song
pub const MAX_SONG_LINES: usize = 99;

/// Song playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SongPlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// A single line in a song
/// Represents a pattern occurrence with playback settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongLine {
    /// Pattern ID to play (0-127)
    pub pattern_id: u8,
    /// Number of times to repeat this pattern (1-99)
    pub repeats: u8,
    /// Length modifier (0.5x - 2.0x)
    pub length_mod: f32,
    /// Tempo modifier (0.5x - 2.0x)
    pub tempo_mod: f32,
}

impl Default for SongLine {
    fn default() -> Self {
        Self {
            pattern_id: 0,
            repeats: 1,
            length_mod: 1.0,
            tempo_mod: 1.0,
        }
    }
}

impl SongLine {
    /// Create a new song line with basic settings
    pub fn new(pattern_id: u8, repeats: u8) -> Self {
        Self {
            pattern_id,
            repeats,
            length_mod: 1.0,
            tempo_mod: 1.0,
        }
    }

    /// Set the length modifier
    pub fn with_length_mod(mut self, length_mod: f32) -> Self {
        self.length_mod = length_mod.clamp(0.5, 2.0);
        self
    }

    /// Set the tempo modifier
    pub fn with_tempo_mod(mut self, tempo_mod: f32) -> Self {
        self.tempo_mod = tempo_mod.clamp(0.5, 2.0);
        self
    }

    /// Validate the song line
    pub fn is_valid(&self) -> bool {
        self.repeats >= 1 && self.repeats <= 99
    }
}

/// A complete song containing multiple lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    /// Song ID (0-15)
    pub id: u8,
    /// Song name (max 32 characters)
    pub name: String,
    /// Lines in this song
    pub lines: Vec<SongLine>,
    /// BPM for this song (overrides global BPM when playing)
    pub tempo: Option<u16>,
    /// Whether this song is empty
    pub is_empty: bool,
}

impl Default for Song {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("New Song"),
            lines: Vec::new(),
            tempo: None,
            is_empty: true,
        }
    }
}

impl Song {
    /// Create a new empty song
    pub fn new(id: u8, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            lines: Vec::new(),
            tempo: None,
            is_empty: true,
        }
    }

    /// Add a line to the song
    pub fn add_line(&mut self, line: SongLine) -> Result<(), SongError> {
        if self.lines.len() >= MAX_SONG_LINES {
            return Err(SongError::MaxLinesExceeded);
        }
        if !line.is_valid() {
            return Err(SongError::InvalidLine);
        }
        self.lines.push(line);
        self.is_empty = false;
        Ok(())
    }

    /// Add a pattern to the song
    pub fn add_pattern(&mut self, pattern_id: u8, repeats: u8) -> Result<(), SongError> {
        self.add_line(SongLine::new(pattern_id, repeats))
    }

    /// Get the total number of pattern plays
    pub fn total_plays(&self) -> u32 {
        self.lines.iter().map(|line| line.repeats as u32).sum()
    }

    /// Check if the song is valid for playback
    pub fn can_play(&self) -> bool {
        !self.is_empty && !self.lines.is_empty()
    }

    /// Clear all lines
    pub fn clear(&mut self) {
        self.lines.clear();
        self.is_empty = true;
    }

    /// Get the effective tempo (song tempo or global default)
    pub fn get_tempo(&self, global_tempo: u16) -> u16 {
        self.tempo.unwrap_or(global_tempo)
    }
}

/// Song manager - handles multiple songs
#[derive(Debug, Clone)]
pub struct SongManager {
    /// All available songs
    songs: Vec<Song>,
    /// Currently selected song
    current_song_id: u8,
    /// Current playback state
    playback_state: SongPlaybackState,
    /// Current line index
    current_line: usize,
    /// Current repeat count for current line
    current_repeat: u8,
}

impl Default for SongManager {
    fn default() -> Self {
        let mut songs = Vec::with_capacity(MAX_SONGS);
        for i in 0..MAX_SONGS as u8 {
            songs.push(Song::new(i, &format!("Song {}", i + 1)));
        }
        Self {
            songs,
            current_song_id: 0,
            playback_state: SongPlaybackState::Stopped,
            current_line: 0,
            current_repeat: 0,
        }
    }
}

impl SongManager {
    /// Create a new song manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a song by ID
    pub fn get_song(&self, id: u8) -> Option<&Song> {
        self.songs.get(id as usize)
    }

    /// Get a mutable song by ID
    pub fn get_song_mut(&mut self, id: u8) -> Option<&mut Song> {
        self.songs.get_mut(id as usize)
    }

    /// Get the current song
    pub fn current_song(&self) -> Option<&Song> {
        self.get_song(self.current_song_id)
    }

    /// Get the current mutable song
    pub fn current_song_mut(&mut self) -> Option<&mut Song> {
        self.get_song_mut(self.current_song_id)
    }

    /// Set the current song
    pub fn set_current_song(&mut self, id: u8) -> Result<(), SongError> {
        if id >= MAX_SONGS as u8 {
            return Err(SongError::InvalidSongId);
        }
        self.current_song_id = id;
        self.stop();
        Ok(())
    }

    /// Add a line to the current song
    pub fn add_line_to_current(&mut self, line: SongLine) -> Result<(), SongError> {
        if let Some(song) = self.current_song_mut() {
            song.add_line(line)
        } else {
            Err(SongError::NoCurrentSong)
        }
    }

    /// Clear the current song
    pub fn clear_current(&mut self) {
        if let Some(song) = self.current_song_mut() {
            song.clear();
        }
    }

    /// Get the next pattern to play
    pub fn next_pattern(&mut self) -> Option<(u8, f32, f32)> {
        if self.playback_state != SongPlaybackState::Playing {
            return None;
        }

        // Get current song directly without borrowing
        let song_id = self.current_song_id;
        let song = self.songs.get(song_id as usize)?;

        if song.lines.is_empty() {
            return None;
        }

        // Check if we need to advance to next line
        let current_line_repeats = song.lines[self.current_line].repeats;
        if self.current_repeat > current_line_repeats {
            // Move to next line
            self.current_line += 1;
            if self.current_line >= song.lines.len() {
                // Song finished
                self.playback_state = SongPlaybackState::Stopped;
                self.current_line = 0;
                self.current_repeat = 1;
                return None;
            }
            self.current_repeat = 1;
        }

        // Get current line
        let line = &song.lines[self.current_line];
        self.current_repeat += 1;

        Some((line.pattern_id, line.length_mod, line.tempo_mod))
    }

    /// Start playback
    pub fn play(&mut self) {
        let song_id = self.current_song_id;
        if let Some(song) = self.songs.get(song_id as usize) {
            if song.can_play() {
                self.playback_state = SongPlaybackState::Playing;
                self.current_line = 0;
                self.current_repeat = 1; // Start at repeat 1
            }
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.playback_state == SongPlaybackState::Playing {
            self.playback_state = SongPlaybackState::Paused;
        }
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.playback_state = SongPlaybackState::Stopped;
        self.current_line = 0;
        self.current_repeat = 0;
    }

    /// Get playback state
    pub fn playback_state(&self) -> SongPlaybackState {
        self.playback_state
    }

    /// Get current line index
    pub fn current_line_index(&self) -> usize {
        self.current_line
    }

    /// Get total lines in current song
    pub fn current_song_line_count(&self) -> usize {
        self.current_song().map_or(0, |s| s.lines.len())
    }

    /// Get total pattern plays in current song
    pub fn current_song_total_plays(&self) -> u32 {
        self.current_song().map_or(0, |s| s.total_plays())
    }

    /// Export song as pattern chain
    pub fn song_to_pattern_chain(&self, song_id: u8) -> Result<PatternChain, SongError> {
        let song = self.get_song(song_id).ok_or(SongError::InvalidSongId)?;

        let mut chain = PatternChain::new();
        for line in &song.lines {
            for _ in 0..line.repeats {
                chain.add_pattern(line.pattern_id);
            }
        }

        Ok(chain)
    }
}

/// Pattern chain - a linear sequence of patterns
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternChain {
    /// Pattern IDs in order
    patterns: Vec<u8>,
    /// Total length in steps
    total_steps: usize,
}

impl PatternChain {
    /// Create a new empty pattern chain
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            total_steps: 0,
        }
    }

    /// Add a pattern to the chain
    pub fn add_pattern(&mut self, pattern_id: u8) {
        self.patterns.push(pattern_id);
        self.total_steps += 16; // Assume 16 steps per pattern
    }

    /// Get pattern at index
    pub fn get_pattern(&self, index: usize) -> Option<u8> {
        self.patterns.get(index).copied()
    }

    /// Get the length of the chain
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Get total steps
    pub fn total_steps(&self) -> usize {
        self.total_steps
    }
}

/// Song-related errors
#[derive(Debug, Clone)]
pub enum SongError {
    InvalidSongId,
    InvalidLine,
    MaxLinesExceeded,
    NoCurrentSong,
    SongEmpty,
}

impl fmt::Display for SongError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SongError::InvalidSongId => write!(f, "Invalid song ID"),
            SongError::InvalidLine => write!(f, "Invalid song line parameters"),
            SongError::MaxLinesExceeded => write!(f, "Maximum song lines exceeded"),
            SongError::NoCurrentSong => write!(f, "No current song selected"),
            SongError::SongEmpty => write!(f, "Song is empty"),
        }
    }
}

impl std::error::Error for SongError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_song_creation() {
        let song = Song::new(0, "Test Song");
        assert_eq!(song.id, 0);
        assert_eq!(song.name, "Test Song");
        assert!(song.is_empty);
        assert!(song.can_play() == false);
    }

    #[test]
    fn test_song_line_creation() {
        let line = SongLine::new(5, 2);
        assert_eq!(line.pattern_id, 5);
        assert_eq!(line.repeats, 2);
        assert_eq!(line.length_mod, 1.0);
        assert!(line.is_valid());
    }

    #[test]
    fn test_song_line_with_modifiers() {
        let line = SongLine::new(3, 4)
            .with_length_mod(0.75)
            .with_tempo_mod(1.5);
        assert_eq!(line.pattern_id, 3);
        assert_eq!(line.repeats, 4);
        assert!((line.length_mod - 0.75).abs() < 0.001);
        assert!((line.tempo_mod - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_song_add_line() {
        let mut song = Song::new(0, "Test");
        assert!(song.add_line(SongLine::new(1, 2)).is_ok());
        assert!(!song.is_empty);
        assert_eq!(song.lines.len(), 1);
    }

    #[test]
    fn test_song_total_plays() {
        let mut song = Song::new(0, "Test");
        song.add_line(SongLine::new(1, 2)).unwrap();
        song.add_line(SongLine::new(2, 3)).unwrap();
        assert_eq!(song.total_plays(), 5);
    }

    #[test]
    fn test_song_manager() {
        let mut manager = SongManager::new();
        assert_eq!(manager.playback_state(), SongPlaybackState::Stopped);

        // Add a line to current song
        assert!(manager.add_line_to_current(SongLine::new(1, 2)).is_ok());

        // Test playback
        manager.play();
        assert_eq!(manager.playback_state(), SongPlaybackState::Playing);

        // Get next pattern
        let next = manager.next_pattern();
        assert!(next.is_some());
        let (pattern_id, _length, _tempo) = next.unwrap();
        assert_eq!(pattern_id, 1);
    }

    #[test]
    fn test_pattern_chain() {
        let mut chain = PatternChain::new();
        chain.add_pattern(1);
        chain.add_pattern(2);
        chain.add_pattern(3);

        assert_eq!(chain.len(), 3);
        assert_eq!(chain.get_pattern(0), Some(1));
        assert_eq!(chain.get_pattern(2), Some(3));
        assert_eq!(chain.total_steps(), 48);
    }

    #[test]
    fn test_song_to_pattern_chain() {
        let mut manager = SongManager::new();
        manager.add_line_to_current(SongLine::new(1, 2)).unwrap();
        manager.add_line_to_current(SongLine::new(2, 1)).unwrap();

        let chain = manager.song_to_pattern_chain(0).unwrap();
        assert_eq!(chain.len(), 3); // 2 + 1 = 3
        assert_eq!(chain.get_pattern(0), Some(1));
        assert_eq!(chain.get_pattern(1), Some(1));
        assert_eq!(chain.get_pattern(2), Some(2));
    }

    #[test]
    fn test_line_modifier_clamping() {
        let line = SongLine::new(1, 1)
            .with_length_mod(3.0) // Should be clamped to 2.0
            .with_tempo_mod(0.1); // Should be clamped to 0.5

        assert!((line.length_mod - 2.0).abs() < 0.001);
        assert!((line.tempo_mod - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_max_songs() {
        let manager = SongManager::new();
        assert_eq!(manager.songs.len(), MAX_SONGS);
    }
}
