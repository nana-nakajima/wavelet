// WAVELET - Step Sequencer Module
// Reference: Elektron Digitakt / Octatrack style
//
// Features:
// - 8 tracks × 16 steps
// - Per-step note + velocity
// - Parameter Lock (different parameters per step)
// - Probability trigger
// - Swing/Shuffle
// - Per-track length

#![allow(dead_code)] // Reserve sequencer fields for future MIDI sync features

/// Number of tracks in the sequencer
pub const NUM_TRACKS: usize = 8;
/// Number of steps per track
pub const NUM_STEPS: usize = 16;

/// Step trigger condition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrigCondition {
    Normal,      // Normal trigger
    Probability, // Probability trigger
    Mute,        // Muted
    Solo,        // Solo
}

/// A single step in the sequencer
#[derive(Debug, Clone)]
pub struct Step {
    /// Whether this step is active
    pub active: bool,
    /// MIDI note number (0-127)
    pub note: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Note length as percentage of step time (0.0 - 1.0)
    pub gate_length: f64,
    /// Probability of triggering (0.0 - 1.0)
    pub probability: f64,
    /// Trigger condition
    pub condition: TrigCondition,
    /// Parameter locks - custom values for this step
    pub param_locks: ParamLocks,
    /// Swing offset for this step (-1.0 to 1.0)
    pub swing: f64,
}

impl Default for Step {
    fn default() -> Self {
        Self {
            active: false,
            note: 60, // Middle C
            velocity: 100,
            gate_length: 0.75,
            probability: 1.0,
            condition: TrigCondition::Normal,
            param_locks: ParamLocks::default(),
            swing: 0.0,
        }
    }
}

/// Parameter locks for a step
/// Allows per-step customization of synth parameters
#[derive(Debug, Clone, Default)]
pub struct ParamLocks {
    /// Filter cutoff frequency (0.0 - 1.0)
    pub filter_cutoff: Option<f64>,
    /// Filter resonance (0.0 - 1.0)
    pub filter_resonance: Option<f64>,
    /// Oscillator waveform
    pub waveform: Option<Waveform>,
    /// Oscillator pitch offset (semitones)
    pub pitch_offset: Option<i8>,
    /// LFO rate (0.0 - 1.0)
    pub lfo_rate: Option<f64>,
    /// LFO depth (0.0 - 1.0)
    pub lfo_depth: Option<f64>,
    /// Attack time (seconds)
    pub attack: Option<f64>,
    /// Decay time (seconds)
    pub decay: Option<f64>,
    /// Sustain level (0.0 - 1.0)
    pub sustain: Option<f64>,
    /// Release time (seconds)
    pub release: Option<f64>,
    /// Effect send levels
    pub reverb_send: Option<f64>,
    pub delay_send: Option<f64>,
    pub distortion_amount: Option<f64>,
}

impl ParamLocks {
    /// Check if any parameter is locked
    pub fn is_empty(&self) -> bool {
        self.filter_cutoff.is_none()
            && self.filter_resonance.is_none()
            && self.waveform.is_none()
            && self.pitch_offset.is_none()
            && self.lfo_rate.is_none()
            && self.lfo_depth.is_none()
            && self.attack.is_none()
            && self.decay.is_none()
            && self.sustain.is_none()
            && self.release.is_none()
            && self.reverb_send.is_none()
            && self.delay_send.is_none()
            && self.distortion_amount.is_none()
    }

    /// Clear all parameter locks
    pub fn clear(&mut self) {
        self.filter_cutoff = None;
        self.filter_resonance = None;
        self.waveform = None;
        self.pitch_offset = None;
        self.lfo_rate = None;
        self.lfo_depth = None;
        self.attack = None;
        self.decay = None;
        self.sustain = None;
        self.release = None;
        self.reverb_send = None;
        self.delay_send = None;
        self.distortion_amount = None;
    }
}

/// A single track in the sequencer
#[derive(Debug, Clone)]
pub struct Track {
    /// Steps in this track
    pub steps: Vec<Step>,
    /// Current step position (0-15)
    pub current_step: usize,
    /// Track length in steps (1-16)
    pub length: usize,
    /// Track is muted
    pub muted: bool,
    /// Track is soloed
    pub solo: bool,
    /// MIDI channel for this track (0-15)
    pub midi_channel: u8,
    /// Swing amount (0.0 - 1.0, where 0.5 is no swing)
    pub swing: f64,
    /// Swing every N steps (typically 2 or 4)
    pub swing_interval: usize,
    /// Scale quantization (None = no quantization)
    pub scale_quantization: Option<Scale>,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            steps: vec![Step::default(); NUM_STEPS],
            current_step: 0,
            length: 16,
            muted: false,
            solo: false,
            midi_channel: 0,
            swing: 0.5,
            swing_interval: 2,
            scale_quantization: None,
        }
    }
}

impl Track {
    /// Create a new track with the first step active
    pub fn new() -> Self {
        let mut track = Self::default();
        track.steps[0] = Step {
            active: true,
            note: 60,
            ..Step::default()
        };
        track
    }

    /// Get the current step (immutable)
    pub fn current_step(&self) -> &Step {
        &self.steps[self.current_step]
    }

    /// Advance to the next step
    pub fn advance(&mut self) -> bool {
        self.current_step = (self.current_step + 1) % self.length;
        self.current_step == 0 // Return true if we wrapped around
    }

    /// Set step length
    pub fn set_length(&mut self, length: usize) {
        self.length = length.clamp(1, NUM_STEPS);
        if self.current_step >= self.length {
            self.current_step = 0;
        }
    }

    /// Reset to step 0
    pub fn reset(&mut self) {
        self.current_step = 0;
    }

    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Toggle solo
    pub fn toggle_solo(&mut self) {
        self.solo = !self.solo;
    }
}

/// Musical scales for quantization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    HarmonicMinor,
    MelodicMinor,
    WholeTone,
    Chromatic,
}

/// Scale degree to semitone offset
pub fn scale_offset(scale: Scale, degree: i8) -> i8 {
    // Returns semitone offset for a given scale degree
    // Chromatic scale has 12 notes, others have 7
    if scale == Scale::Chromatic {
        return degree.rem_euclid(12);
    }

    let scale_pattern: [i8; 7] = match scale {
        Scale::Major => [0, 2, 4, 5, 7, 9, 11],
        Scale::Minor => [0, 2, 3, 5, 7, 8, 10],
        Scale::Dorian => [0, 2, 3, 5, 7, 9, 10],
        Scale::Phrygian => [0, 1, 3, 5, 7, 8, 10],
        Scale::Lydian => [0, 2, 4, 6, 7, 9, 11],
        Scale::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
        Scale::Locrian => [0, 1, 3, 5, 6, 8, 10],
        Scale::HarmonicMinor => [0, 2, 3, 5, 7, 8, 11],
        Scale::MelodicMinor => [0, 2, 3, 5, 7, 9, 11],
        Scale::WholeTone => [0, 2, 4, 6, 8, 10, 10],
        // Chromatic is handled separately above
        Scale::Chromatic => [0, 1, 2, 3, 4, 5, 6], // Won't be used
    };

    let normalized = degree.rem_euclid(7); // Handle negative degrees
    scale_pattern[normalized as usize]
}

/// Quantize a note to the nearest scale note
pub fn quantize_to_scale(note: u8, root: u8, scale: Scale) -> u8 {
    let root = root % 12;
    let octave = (note / 12) as i8;
    let note_in_octave = (note % 12) as i8;

    // Chromatic scale: any note is valid
    if scale == Scale::Chromatic {
        return note;
    }

    // Find the closest scale note
    let scale_pattern: [i8; 7] = match scale {
        Scale::Major => [0, 2, 4, 5, 7, 9, 11],
        Scale::Minor => [0, 2, 3, 5, 7, 8, 10],
        _ => [0, 2, 4, 5, 7, 9, 11],
    };

    let mut closest_diff = 12;
    let mut closest_note = note_in_octave;

    for &scale_note in &scale_pattern {
        let diff = (note_in_octave - scale_note).abs();
        if diff < closest_diff {
            closest_diff = diff;
            closest_note = scale_note;
        }
    }

    let result = (octave * 12) + (root as i8) + closest_note;
    result as u8
}

use crate::oscillator::Waveform;
use crate::synth::Synth;

/// Main step sequencer
#[derive(Debug, Clone)]
pub struct StepSequencer {
    /// All tracks
    pub tracks: Vec<Track>,
    /// Master BPM
    pub bpm: f64,
    /// Current beat position
    pub beat_position: f64,
    /// Beats per measure
    pub beats_per_measure: usize,
    /// Whether the sequencer is playing
    pub playing: bool,
    /// Current track for recording
    pub record_track: Option<usize>,
    /// Swing enabled
    pub swing_enabled: bool,
    /// Master swing amount (0.0 - 1.0)
    pub swing_amount: f64,
    /// Swing interval (every N steps)
    pub swing_interval: usize,
    /// Random state for deterministic generation
    random_state: u64,
}

impl Default for StepSequencer {
    fn default() -> Self {
        let mut tracks = Vec::with_capacity(NUM_TRACKS);
        for _ in 0..NUM_TRACKS {
            tracks.push(Track::default());
        }

        Self {
            tracks,
            bpm: 120.0,
            beat_position: 0.0,
            beats_per_measure: 4,
            playing: false,
            record_track: None,
            swing_enabled: true,
            swing_amount: 0.5,
            swing_interval: 2,
            random_state: 12345,
        }
    }
}

impl StepSequencer {
    /// Create a new step sequencer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific BPM
    pub fn with_bpm(bpm: f64) -> Self {
        Self {
            bpm,
            ..Default::default()
        }
    }

    /// Get track by index
    pub fn track(&self, index: usize) -> Option<&Track> {
        self.tracks.get(index)
    }

    /// Get track by index (mutable)
    pub fn track_mut(&mut self, index: usize) -> Option<&mut Track> {
        self.tracks.get_mut(index)
    }

    /// Get all non-muted tracks (respecting solo)
    pub fn active_tracks(&self) -> Vec<usize> {
        let has_solo = self.tracks.iter().any(|t| t.solo);

        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| {
                if has_solo {
                    track.solo && !track.muted
                } else {
                    !track.muted
                }
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Start playback
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.playing = false;
        self.beat_position = 0.0;
        for track in &mut self.tracks {
            track.reset();
        }
    }

    /// Toggle play/stop
    pub fn toggle(&mut self) {
        if self.playing {
            self.stop();
        } else {
            self.play();
        }
    }

    /// Reset to the beginning
    pub fn reset(&mut self) {
        self.beat_position = 0.0;
        for track in &mut self.tracks {
            track.reset();
        }
    }

    /// Set BPM
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm.clamp(20.0, 300.0);
    }

    /// Calculate time per step in seconds
    pub fn step_time(&self) -> f64 {
        60.0 / self.bpm / 4.0 // 16th notes
    }

    /// Simple LCG random number generator
    fn random(&mut self) -> f64 {
        self.random_state = self
            .random_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        ((self.random_state >> 33) as u32) as f64 / u32::MAX as f64
    }

    /// Process one audio sample
    /// Returns (trigger, track_index, step) for triggered steps
    pub fn process(&mut self, sample_rate: f64) -> Vec<(bool, usize, usize)> {
        if !self.playing {
            return Vec::new();
        }

        let step_duration = 60.0 / self.bpm / 4.0; // 16th note at given BPM
        let samples_per_step = step_duration * sample_rate;

        self.beat_position += 1.0 / samples_per_step;

        // Pre-calculate random values for this buffer
        let mut rng_values: Vec<f64> = Vec::with_capacity(self.tracks.len());
        for _ in 0..self.tracks.len() {
            rng_values.push(self.random());
        }

        let mut triggers = Vec::new();

        for (track_idx, track) in self.tracks.iter_mut().enumerate() {
            let current_beat = self.beat_position * 4.0; // Convert to 16th notes
            let step_float = current_beat.floor() as usize % track.length;

            if step_float != track.current_step {
                track.current_step = step_float;
                let step = &track.steps[track.current_step];

                // Check if step should trigger
                let should_trigger = if track.muted {
                    false
                } else {
                    match step.condition {
                        TrigCondition::Normal => true,
                        TrigCondition::Probability => rng_values[track_idx] < step.probability,
                        TrigCondition::Mute => false,
                        TrigCondition::Solo => false,
                    }
                };

                if should_trigger {
                    triggers.push((true, track_idx, track.current_step));
                }
            }
        }

        triggers
    }

    /// Get the next note to play
    /// Returns (note, velocity, gate_length, track_index) or None
    pub fn get_next_note(&mut self, sample_rate: f64) -> Option<(u8, u8, f64, usize, ParamLocks)> {
        let triggers = self.process(sample_rate);

        for (trigger, track_idx, step_idx) in triggers {
            if trigger {
                let track = &self.tracks[track_idx];
                let step = &track.steps[step_idx];

                if step.active {
                    return Some((
                        step.note,
                        step.velocity,
                        step.gate_length,
                        track_idx,
                        step.param_locks.clone(),
                    ));
                }
            }
        }

        None
    }

    /// Apply parameter locks to a synth
    pub fn apply_param_locks(&self, synth: &mut Synth, locks: &ParamLocks) {
        if let Some(cutoff) = locks.filter_cutoff {
            synth.set_filter_cutoff(cutoff as f32);
        }
        if let Some(resonance) = locks.filter_resonance {
            synth.set_filter_resonance(resonance as f32);
        }
        // Note: Synth doesn't have set_waveform/set_detune,
        // parameter locks for pitch/waveform would need UI handling
    }

    /// Clear all patterns
    pub fn clear_all(&mut self) {
        for track in &mut self.tracks {
            for step in &mut track.steps {
                *step = Step::default();
            }
            track.reset();
        }
    }

    /// Randomize all tracks
    pub fn randomize(&mut self) {
        // Pre-generate random values to avoid borrow issues
        let num_steps = self.tracks.iter().map(|t| t.steps.len()).sum::<usize>();
        let mut rng_values: Vec<f64> = Vec::with_capacity(num_steps * 4);
        for _ in 0..rng_values.capacity() {
            rng_values.push(self.random());
        }

        let mut rng_idx = 0;
        for track in &mut self.tracks {
            for step in &mut track.steps {
                let active = rng_values[rng_idx] > 0.5;
                rng_idx += 1;

                if active {
                    step.active = true;
                    step.note = (36.0 + rng_values[rng_idx] * 60.0) as u8;
                    rng_idx += 1;
                    step.velocity = (64.0 + rng_values[rng_idx] * 63.0) as u8;
                    rng_idx += 1;
                    step.gate_length = rng_values[rng_idx] * 0.5 + 0.25;
                    rng_idx += 1;
                    step.probability = rng_values[rng_idx] * 0.5 + 0.5;
                    rng_idx += 1;
                } else {
                    step.active = false;
                }
            }
        }
    }

    /// Generate a drum pattern for a track
    pub fn generate_drum_pattern(&mut self, track_idx: usize, style: DrumStyle) {
        if track_idx >= self.tracks.len() {
            return;
        }

        let track = &mut self.tracks[track_idx];

        match style {
            DrumStyle::FourOnTheFloor => {
                for step in 0..16 {
                    track.steps[step].active = step % 4 == 0;
                    track.steps[step].note = 36;
                    track.steps[step].velocity = if step % 4 == 0 { 127 } else { 0 };
                }
            }
            DrumStyle::Breakbeat => {
                let pattern = [1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0];
                for (i, &active) in pattern.iter().enumerate() {
                    track.steps[i].active = active == 1;
                    track.steps[i].note = 36;
                    track.steps[i].velocity = 100;
                }
            }
            DrumStyle::Techno => {
                for step in 0..16 {
                    track.steps[step].active = step % 4 == 0 || step % 8 == 6;
                    track.steps[step].note = 36;
                    track.steps[step].velocity = 120;
                }
            }
            DrumStyle::HipHop => {
                let pattern = [1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0];
                for (i, &active) in pattern.iter().enumerate() {
                    track.steps[i].active = active == 1;
                    track.steps[i].note = 36;
                    track.steps[i].velocity = 100;
                }
            }
        }
    }

    /// Simple random helper methods
    fn bool(&mut self) -> bool {
        self.random() > 0.5
    }

    fn next_u8(&mut self, range: std::ops::Range<u8>) -> u8 {
        let r = self.random();
        (range.start as f64 + r * (range.end - range.start) as f64) as u8
    }
}

/// Drum pattern styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumStyle {
    FourOnTheFloor,
    Breakbeat,
    Techno,
    HipHop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_sequencer_creation() {
        let seq = StepSequencer::new();
        assert_eq!(seq.bpm, 120.0);
        assert!(!seq.playing);
        assert_eq!(seq.tracks.len(), NUM_TRACKS);
    }

    #[test]
    fn test_track_creation() {
        let track = Track::new();
        assert_eq!(track.length, 16);
        assert!(!track.muted);
        assert!(!track.solo);
        assert_eq!(track.current_step, 0);
    }

    #[test]
    fn test_step_defaults() {
        let step = Step::default();
        assert!(!step.active);
        assert_eq!(step.note, 60);
        assert_eq!(step.velocity, 100);
        assert_eq!(step.gate_length, 0.75);
        assert_eq!(step.probability, 1.0);
        assert_eq!(step.condition, TrigCondition::Normal);
    }

    #[test]
    fn test_param_locks() {
        let locks = ParamLocks::default();
        assert!(locks.is_empty());

        let mut locks = ParamLocks::default();
        locks.filter_cutoff = Some(0.5);
        locks.filter_resonance = Some(0.3);
        assert!(!locks.is_empty());

        locks.clear();
        assert!(locks.is_empty());
    }

    #[test]
    fn test_track_advance() {
        let mut track = Track::new();
        assert_eq!(track.current_step, 0);

        track.advance();
        assert_eq!(track.current_step, 1);

        track.set_length(4);
        // Advance from step 1: 1->2->3->0 (3 steps to wrap)
        for _ in 0..3 {
            track.advance();
        }
        assert_eq!(track.current_step, 0);
    }

    #[test]
    fn test_track_length() {
        let mut track = Track::new();
        track.set_length(8);
        assert_eq!(track.length, 8);

        track.set_length(20);
        assert_eq!(track.length, 16);
    }

    #[test]
    fn test_scale_quantize() {
        let quantized = quantize_to_scale(61, 0, Scale::Major);
        assert_eq!(quantized, 60);

        let quantized = quantize_to_scale(62, 0, Scale::Major);
        assert_eq!(quantized, 62);
    }

    #[test]
    fn test_drum_pattern() {
        let mut seq = StepSequencer::new();
        seq.generate_drum_pattern(0, DrumStyle::FourOnTheFloor);

        let track = &seq.tracks[0];
        for i in 0..16 {
            if i % 4 == 0 {
                assert!(track.steps[i].active);
            } else {
                assert!(!track.steps[i].active);
            }
        }
    }

    #[test]
    fn test_mute_solo() {
        let mut track = Track::new();
        assert!(!track.muted);
        assert!(!track.solo);

        track.toggle_mute();
        assert!(track.muted);

        track.toggle_solo();
        assert!(track.solo);
    }

    #[test]
    fn test_bpm_setting() {
        let mut seq = StepSequencer::new();
        seq.set_bpm(140.0);
        assert_eq!(seq.bpm, 140.0);

        seq.set_bpm(500.0);
        assert_eq!(seq.bpm, 300.0);
    }

    #[test]
    fn test_step_sequencer_playback() {
        let mut seq = StepSequencer::new();
        seq.bpm = 120.0;

        seq.tracks[0].steps[0].active = true;
        seq.tracks[0].steps[0].note = 60;

        seq.play();

        for _ in 0..1000 {
            seq.process(44100.0);
        }

        assert!(seq.playing);
    }

    #[test]
    fn test_process_returns_triggers_for_active_steps() {
        let mut seq = StepSequencer::new();
        seq.tracks[0].steps[0].active = true;
        seq.tracks[0].steps[0].note = 60;
        seq.tracks[0].steps[1].active = true;
        seq.tracks[0].steps[1].note = 64;

        seq.play();

        // Collect triggers over enough samples to advance through steps
        let mut triggered_steps: Vec<usize> = Vec::new();
        for _ in 0..100_000 {
            let triggers = seq.process(44100.0);
            for (trigger, _track_idx, step_idx) in &triggers {
                if *trigger {
                    triggered_steps.push(*step_idx);
                }
            }
        }

        // Should have triggered step 0 and step 1 at minimum
        assert!(triggered_steps.contains(&0), "Step 0 should have triggered");
        assert!(triggered_steps.contains(&1), "Step 1 should have triggered");
    }

    #[test]
    fn test_muted_track_produces_no_triggers() {
        let mut seq = StepSequencer::new();
        seq.tracks[0].steps[0].active = true;
        seq.tracks[0].muted = true;

        seq.play();

        let mut any_trigger = false;
        for _ in 0..50_000 {
            let triggers = seq.process(44100.0);
            for (trigger, track_idx, _) in &triggers {
                if *trigger && *track_idx == 0 {
                    any_trigger = true;
                }
            }
        }

        assert!(!any_trigger, "Muted track should not produce triggers");
    }

    #[test]
    fn test_get_next_note_returns_correct_note() {
        let mut seq = StepSequencer::new();
        seq.tracks[0].steps[0].active = true;
        seq.tracks[0].steps[0].note = 72;
        seq.tracks[0].steps[0].velocity = 110;

        seq.play();

        let mut found_note = false;
        for _ in 0..100_000 {
            if let Some((note, vel, _gate, _track, _locks)) = seq.get_next_note(44100.0) {
                assert_eq!(note, 72);
                assert_eq!(vel, 110);
                found_note = true;
                break;
            }
        }
        assert!(found_note, "Should have found the note");
    }

    #[test]
    fn test_stop_resets_position() {
        let mut seq = StepSequencer::new();
        seq.tracks[0].steps[0].active = true;
        seq.play();

        for _ in 0..10_000 {
            seq.process(44100.0);
        }
        assert!(seq.beat_position > 0.0);

        seq.stop();
        assert!(!seq.playing);
        assert_eq!(seq.beat_position, 0.0);
        assert_eq!(seq.tracks[0].current_step, 0);
    }

    #[test]
    fn test_step_time_calculation() {
        let seq = StepSequencer::with_bpm(120.0);
        // At 120 BPM, one beat = 0.5s, one 16th note = 0.5/4 = 0.125s
        let expected = 60.0 / 120.0 / 4.0;
        assert!((seq.step_time() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_active_tracks_respects_solo() {
        let mut seq = StepSequencer::new();
        // Solo track 2
        seq.tracks[2].solo = true;

        let active = seq.active_tracks();
        assert_eq!(active, vec![2], "Only soloed track should be active");
    }

    #[test]
    fn test_active_tracks_excludes_muted() {
        let mut seq = StepSequencer::new();
        seq.tracks[0].muted = true;
        seq.tracks[3].muted = true;

        let active = seq.active_tracks();
        assert!(!active.contains(&0));
        assert!(!active.contains(&3));
        assert_eq!(active.len(), NUM_TRACKS - 2);
    }

    #[test]
    fn test_track_wraps_at_custom_length() {
        let mut track = Track::new();
        track.set_length(4);

        // Advance 4 times: 0->1->2->3->0
        for _ in 0..4 {
            track.advance();
        }
        assert_eq!(track.current_step, 0, "Track should wrap at length 4");
    }

    #[test]
    fn test_scale_quantize_chromatic_passthrough() {
        // Chromatic scale should pass through any note unchanged
        for note in 48..72 {
            let q = quantize_to_scale(note, 0, Scale::Chromatic);
            assert_eq!(q, note, "Chromatic should not alter note {}", note);
        }
    }

    #[test]
    fn test_scale_quantize_major_snaps_to_nearest() {
        // C major: C D E F G A B = 0 2 4 5 7 9 11
        // C# (1) should snap to C (0) or D (2)
        let q = quantize_to_scale(61, 0, Scale::Major);
        assert!(q == 60 || q == 62, "C# should snap to C or D, got {}", q);

        // F# (66) should snap to F (65) or G (67)
        let q = quantize_to_scale(66, 0, Scale::Major);
        assert!(q == 65 || q == 67, "F# should snap to F or G, got {}", q);
    }

    #[test]
    fn test_drum_pattern_breakbeat() {
        let mut seq = StepSequencer::new();
        seq.generate_drum_pattern(0, DrumStyle::Breakbeat);

        let track = &seq.tracks[0];
        let active_count = track.steps.iter().filter(|s| s.active).count();
        assert!(active_count > 0, "Breakbeat should have active steps");
    }

    #[test]
    fn test_clear_all_deactivates_steps() {
        let mut seq = StepSequencer::new();
        seq.tracks[0].steps[0].active = true;
        seq.tracks[0].steps[4].active = true;
        seq.tracks[1].steps[8].active = true;

        seq.clear_all();

        for track in &seq.tracks {
            for step in &track.steps {
                assert!(!step.active, "All steps should be inactive after clear");
            }
        }
    }

    #[test]
    fn test_bpm_clamping() {
        let mut seq = StepSequencer::new();
        seq.set_bpm(10.0);
        assert_eq!(seq.bpm, 20.0, "BPM should clamp to minimum 20");

        seq.set_bpm(999.0);
        assert_eq!(seq.bpm, 300.0, "BPM should clamp to maximum 300");
    }

    #[test]
    fn test_not_playing_returns_no_triggers() {
        let mut seq = StepSequencer::new();
        seq.tracks[0].steps[0].active = true;
        // Don't call play()

        let triggers = seq.process(44100.0);
        assert!(
            triggers.is_empty(),
            "Stopped sequencer should produce no triggers"
        );
    }
}
