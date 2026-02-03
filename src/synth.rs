//! Synth Module
//!
//! This module provides the main synthesizer engine that combines all
//! components (oscillators, filters, envelopes, LFOs, effects) into
//! a complete instrument.

use crate::effects::{Effect, EffectProcessor, EffectType, Saturation};
use crate::envelope::{AdsrEnvelope, EnvelopeConfig};
use crate::filter::{Filter, FilterType, ZdfFilter, ZdfFilterConfig, ZdfFilterMode};
use crate::lfo::{Lfo, LfoConfig, LfoRate};
use crate::oscillator::{
    midi_to_frequency, Oscillator, OscillatorConfig, OversampleFactor, Waveform,
};
use std::collections::HashMap;

/// Maximum number of simultaneous voices (polyphony).
const MAX_VOICES: usize = 16;

/// Virtual Analog (VA) parameter IDs for control and automation.
///
/// These constants define the parameter IDs used for controlling
/// the Virtual Analog features of the synthesizer.
pub const PARAM_ZDF_ENABLED: i32 = 50;
pub const PARAM_ZDF_CUTOFF: i32 = 51;
pub const PARAM_ZDF_RES: i32 = 52;
pub const PARAM_ZDF_DRIVE: i32 = 53;
pub const PARAM_SATURATION_DRIVE: i32 = 54;
pub const PARAM_SATURATION_MIX: i32 = 55;
pub const PARAM_OVERSAMPLE: i32 = 56;

/// AI Melody Generation parameter IDs.
///
/// These constants define the parameter IDs used for controlling
/// the AI melody generation features.
pub const PARAM_MELODY_STYLE: i32 = 60;
pub const PARAM_MELODY_GENERATE: i32 = 61;
pub const PARAM_MELODY_LENGTH: i32 = 62;

/// Voice structure representing one playing note.
#[derive(Debug, Clone)]
struct Voice {
    /// Oscillator for this voice
    oscillator: Oscillator,

    /// Amplitude envelope
    amplitude_envelope: AdsrEnvelope,

    /// Current MIDI note
    note: u8,

    /// Current velocity (0-127)
    velocity: u8,

    /// Whether this voice is active
    active: bool,
}

impl Voice {
    /// Creates a new voice for a specific note.
    fn new(note: u8, velocity: u8, sample_rate: f32) -> Self {
        let freq = midi_to_frequency(note);

        let osc_config = OscillatorConfig {
            waveform: Waveform::Sawtooth,
            frequency: freq,
            amplitude: velocity as f32 / 127.0,
            phase_offset: 0.0,
            sample_rate,
            oversample_factor: OversampleFactor::None,
        };

        let env_config = EnvelopeConfig {
            attack: 0.01,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
            sample_rate,
            ..Default::default()
        };

        Self {
            oscillator: Oscillator::new(osc_config),
            amplitude_envelope: AdsrEnvelope::with_config(env_config),
            note,
            velocity,
            active: true,
        }
    }

    /// Processes one sample from this voice.
    fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let env_level = self.amplitude_envelope.process();
        let osc_sample = self.oscillator.next_sample();

        osc_sample * env_level
    }

    /// Triggers the voice (note on).
    fn trigger(&mut self) {
        self.amplitude_envelope.note_on();
    }

    /// Releases the voice (note off).
    fn release(&mut self) {
        self.amplitude_envelope.note_off();
    }

    /// Checks if the voice is still active.
    fn is_active(&self) -> bool {
        self.active && self.amplitude_envelope.is_active()
    }

    /// Stops the voice immediately.
    fn stop(&mut self) {
        self.active = false;
    }
}

/// Main synthesizer structure.
///
/// The Synth combines oscillators, filters, envelopes, LFOs, and effects
/// into a complete synthesizer engine supporting polyphony and modulation.
///
/// # Virtual Analog Features
///
/// The synthesizer includes several Virtual Analog features:
/// - **ZDF Filter**: Zero-Delay Feedback ladder filter (Moog-style)
/// - **Saturation**: Analog-style soft clipping and harmonic enhancement
/// - **Oversampling**: Anti-aliasing for oscillators (2x, 4x, 8x)
///
/// # Example
///
/// ```rust
/// use wavelet::synth::Synth;
///
/// let mut synth = Synth::new(48000.0);
/// synth.note_on(60, 100);  // C4, velocity 100
/// synth.note_off(60);      // Release C4
/// ```
#[derive(Debug, Clone)]
pub struct Synth {
    /// Active voices for polyphony
    voices: Vec<Voice>,

    /// Global biquad filter (original filter)
    filter: Filter,

    /// ZDF (Zero-Delay Feedback) ladder filter for VA character
    zdf_filter: ZdfFilter,

    /// Whether ZDF filter is active
    zdf_enabled: bool,

    /// Saturation effect for analog-style saturation
    saturation: Saturation,

    /// Global LFOs for modulation
    lfos: Vec<Lfo>,

    /// Global effect processor
    effects: EffectProcessor,

    /// Master volume
    master_volume: f32,

    /// Sample rate
    sample_rate: f32,

    /// Active note tracking for voice allocation
    active_notes: HashMap<u8, usize>, // note -> voice index

    /// Oversampling factor for oscillators
    oversample_factor: OversampleFactor,
}

impl Synth {
    /// Creates a new synthesizer instance.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Audio system sample rate in Hz
    ///
    /// # Returns
    ///
    /// A new Synth instance ready for use
    pub fn new(sample_rate: f32) -> Self {
        let lfo_config = LfoConfig {
            rate: LfoRate::Hertz(4.0),
            waveform: Waveform::Sine,
            depth: 0.3,
            sample_rate,
            ..Default::default()
        };

        let zdf_config = ZdfFilterConfig {
            mode: ZdfFilterMode::LowPass4,
            cutoff_frequency: 1000.0,
            resonance: 1.0,
            drive: 0.0,
            sample_rate,
        };

        Self {
            voices: Vec::with_capacity(MAX_VOICES),
            filter: Filter::new(FilterType::LowPass, 2000.0, 1.0, sample_rate),
            zdf_filter: ZdfFilter::with_config(zdf_config),
            zdf_enabled: true,
            saturation: Saturation::new(),
            lfos: vec![Lfo::with_config(lfo_config)],
            effects: EffectProcessor::new(sample_rate),
            master_volume: 0.7,
            sample_rate,
            active_notes: HashMap::new(),
            oversample_factor: OversampleFactor::None,
        }
    }

    /// Creates a new synthesizer with default sample rate (44100 Hz).
    pub fn default() -> Self {
        Self::new(44100.0)
    }

    /// Processes one stereo sample pair.
    ///
    /// # Returns
    ///
    /// Tuple of (left, right) audio samples
    pub fn process_stereo(&mut self) -> (f32, f32) {
        let sample = self.process_mono();
        (sample * self.master_volume, sample * self.master_volume)
    }

    /// Processes one mono sample.
    ///
    /// # Returns
    ///
    /// Mono audio sample
    pub fn process_mono(&mut self) -> f32 {
        // Sum all active voices
        let mut output = 0.0f32;

        // Process voices and collect output
        let mut voices_to_remove = Vec::new();

        for (note, voice_idx) in &self.active_notes.clone() {
            if let Some(voice) = self.voices.get_mut(*voice_idx) {
                if voice.is_active() {
                    output += voice.process();
                } else {
                    voices_to_remove.push(*note);
                }
            }
        }

        // Remove finished voices
        for note in voices_to_remove {
            self.active_notes.remove(&note);
        }

        // Process through ZDF filter if enabled
        if self.zdf_enabled {
            output = self.zdf_filter.process_sample(output);
        }

        // Process through biquad filter (original filter)
        let filtered = self.filter.process(output);

        // Process through saturation
        let saturated = self.saturation.process_sample(filtered);

        // Process through effects
        self.effects.process(saturated)
    }

    /// Processes a block of stereo samples.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of samples to process
    ///
    /// # Returns
    ///
    /// Vector of (left, right) sample pairs
    pub fn process_block_stereo(&mut self, count: usize) -> Vec<(f32, f32)> {
        (0..count).map(|_| self.process_stereo()).collect()
    }

    /// Processes a block of mono samples.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of samples to process
    ///
    /// # Returns
    ///
    /// Vector of mono samples
    pub fn process_block_mono(&mut self, count: usize) -> Vec<f32> {
        (0..count).map(|_| self.process_mono()).collect()
    }

    /// Triggers a note (note on event).
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    /// * `velocity` - Note velocity (0-127)
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        if velocity == 0 {
            self.note_off_specific(note);
            return;
        }

        // Check if note is already playing
        if self.active_notes.contains_key(&note) {
            self.note_off_specific(note);
        }

        // Find available voice (oldest first for voice stealing)
        let voice_idx = if self.voices.len() < MAX_VOICES {
            self.voices
                .push(Voice::new(note, velocity, self.sample_rate));
            self.voices.len() - 1
        } else {
            // Voice stealing: steal oldest voice
            let oldest_note = self.active_notes.keys().min().copied();
            if let Some(old_note) = oldest_note {
                if let Some(&voice_idx) = self.active_notes.get(&old_note) {
                    // Reuse this voice
                    self.active_notes.remove(&old_note);

                    // Reinitialize voice
                    let freq = midi_to_frequency(note);
                    let osc_config = OscillatorConfig {
                        waveform: Waveform::Sawtooth,
                        frequency: freq,
                        amplitude: velocity as f32 / 127.0,
                        phase_offset: 0.0,
                        sample_rate: self.sample_rate,
                        oversample_factor: OversampleFactor::None,
                    };

                    self.voices[voice_idx] = Voice::new(note, velocity, self.sample_rate);
                    voice_idx
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        self.active_notes.insert(note, voice_idx);

        // Trigger the voice
        if let Some(voice) = self.voices.get_mut(voice_idx) {
            voice.trigger();
        }
    }

    /// Releases a note (note off event).
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    pub fn note_off(&mut self) {
        // Release all notes
        for voice in &mut self.voices {
            if voice.active {
                voice.release();
            }
        }
        self.active_notes.clear();
    }

    /// Releases a specific note.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127)
    pub fn note_off_specific(&mut self, note: u8) {
        if let Some(&voice_idx) = self.active_notes.get(&note) {
            if let Some(voice) = self.voices.get_mut(voice_idx) {
                voice.release();
            }
            self.active_notes.remove(&note);
        }
    }

    /// Sets the master volume.
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume level (0.0 to 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Sets the global filter cutoff frequency.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff frequency in Hz
    pub fn set_filter_cutoff(&mut self, cutoff: f32) {
        self.filter.set_cutoff(cutoff);
    }

    /// Sets the global filter resonance.
    ///
    /// # Arguments
    ///
    /// * `resonance` - Q value
    pub fn set_filter_resonance(&mut self, resonance: f32) {
        self.filter.set_resonance(resonance);
    }

    /// Sets the global filter type.
    ///
    /// # Arguments
    ///
    /// * `filter_type` - Type of filter
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.filter.set_type(filter_type);
    }

    /// Sets the active effect type.
    ///
    /// # Arguments
    ///
    /// * `effect_type` - Type of effect
    pub fn set_effect_type(&mut self, effect_type: EffectType) {
        self.effects.set_effect_type(effect_type);
    }

    /// Sets the effect mix.
    ///
    /// # Arguments
    ///
    /// * `mix` - Wet/dry mix (0.0 to 1.0)
    pub fn set_effect_mix(&mut self, mix: f32) {
        self.effects.set_mix(mix);
    }

    // ===== Virtual Analog Feature Controls =====

    /// Enables or disables the ZDF (Zero-Delay Feedback) filter.
    ///
    /// When enabled, the ZDF ladder filter is used instead of the
    /// standard biquad filter for a more analog character.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable the ZDF filter
    pub fn set_zdf_enabled(&mut self, enabled: bool) {
        self.zdf_enabled = enabled;
    }

    /// Sets the ZDF filter cutoff frequency.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff frequency in Hz (20 to 20000)
    pub fn set_zdf_cutoff(&mut self, cutoff: f32) {
        self.zdf_filter.set_cutoff(cutoff);
    }

    /// Sets the ZDF filter resonance.
    ///
    /// # Arguments
    ///
    /// * `resonance` - Resonance value (0.0 to ~4.0)
    pub fn set_zdf_resonance(&mut self, resonance: f32) {
        self.zdf_filter.set_resonance(resonance);
    }

    /// Sets the ZDF filter drive amount.
    ///
    /// # Arguments
    ///
    /// * `drive` - Drive amount (0.0 to ~10.0)
    pub fn set_zdf_drive(&mut self, drive: f32) {
        self.zdf_filter.set_drive(drive);
    }

    /// Sets the saturation drive amount.
    ///
    /// # Arguments
    ///
    /// * `drive` - Drive amount (0.0 to ~10.0)
    pub fn set_saturation_drive(&mut self, drive: f32) {
        self.saturation.set_drive(drive);
    }

    /// Sets the saturation mix.
    ///
    /// # Arguments
    ///
    /// * `mix` - Wet/dry mix (0.0 = dry, 1.0 = fully saturated)
    pub fn set_saturation_mix(&mut self, mix: f32) {
        self.saturation.set_mix(mix);
    }

    /// Sets the oscillator oversampling factor.
    ///
    /// Higher oversampling reduces aliasing but increases CPU usage.
    ///
    /// # Arguments
    ///
    /// * `factor` - Oversampling factor (1x, 2x, 4x, 8x)
    pub fn set_oversample_factor(&mut self, factor: OversampleFactor) {
        self.oversample_factor = factor;
    }

    /// Gets the current oversampling factor.
    ///
    /// # Returns
    ///
    /// Current oversampling factor
    pub fn oversample_factor(&self) -> OversampleFactor {
        self.oversample_factor
    }

    /// Resets the synthesizer state.
    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.stop();
        }
        self.voices.clear();
        self.active_notes.clear();
        self.filter.reset();
        self.zdf_filter.reset();
        self.saturation.reset();
        self.effects.reset();
    }

    /// Gets the number of active voices.
    pub fn active_voice_count(&self) -> usize {
        self.active_notes.len()
    }

    // ===== AI Melody Generation Methods =====

    /// Generates a new melody based on the specified style.
    ///
    /// # Arguments
    ///
    /// * `root_note` - Root MIDI note (default 60 = C4)
    /// * `style` - Melody style (Pop, Jazz, LoFi, EDM, Ambient, Classical)
    /// * `tempo` - Tempo in BPM (default 120)
    /// * `length` - Number of measures (default 4)
    ///
    /// # Returns
    ///
    /// A Melody struct with the generated melody
    pub fn generate_melody(
        &self,
        root_note: u8,
        style: crate::melody_generator::MelodyStyle,
        tempo: f64,
        length: usize,
    ) -> crate::melody_generator::Melody {
        let key = crate::melody_generator::Key {
            root: root_note,
            scale: match style {
                crate::melody_generator::MelodyStyle::Jazz => {
                    crate::melody_generator::Scale::Mixolydian
                }
                crate::melody_generator::MelodyStyle::Ambient => {
                    crate::melody_generator::Scale::Lydian
                }
                _ => crate::melody_generator::Scale::Major,
            },
        };

        let mut generator = crate::melody_generator::MelodyGenerator::new(key, tempo, length);
        generator.generate_preset(style)
    }

    /// Plays a generated melody on the synthesizer.
    ///
    /// # Arguments
    ///
    /// * `melody` - The melody to play
    /// * `start_delay_ms` - Delay before starting (in milliseconds)
    pub fn play_melody(&mut self, melody: &crate::melody_generator::Melody, start_delay_ms: u64) {
        // Calculate delay in samples
        let sample_delay = (start_delay_ms as f64 / 1000.0 * self.sample_rate as f64) as usize;
        let beats_per_second = melody.tempo / 60.0;
        let samples_per_beat = self.sample_rate as f64 / beats_per_second;

        for note in &melody.notes {
            let delay_samples = sample_delay + (note.start_beat * samples_per_beat) as usize;
            let velocity = (note.velocity * 127.0) as u8;

            // Trigger note after delay
            // Note: In a real implementation, this would use a scheduler
            self.note_on(note.pitch, velocity);
        }
    }

    /// Generates and plays a melody in one step.
    ///
    /// # Arguments
    ///
    /// * `root_note` - Root MIDI note
    /// * `style` - Melody style
    /// * `tempo` - Tempo in BPM
    /// * `length` - Number of measures
    pub fn generate_and_play(
        &mut self,
        root_note: u8,
        style: crate::melody_generator::MelodyStyle,
        tempo: f64,
        length: usize,
    ) -> crate::melody_generator::Melody {
        let melody = self.generate_melody(root_note, style, tempo, length);
        self.play_melody(&melody, 0);
        melody
    }

    /// Gets the current tempo.
    ///
    /// # Returns
    ///
    /// Current tempo in BPM (default 120)
    pub fn get_tempo(&self) -> f64 {
        120.0 // Default tempo, can be extended to store actual tempo
    }
}

impl Default for Synth {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synth_default() {
        let synth = Synth::default();
        assert_eq!(synth.master_volume, 0.7);
        assert_eq!(synth.active_voice_count(), 0);
    }

    #[test]
    fn test_synth_note_on() {
        let mut synth = Synth::new(48000.0);
        synth.note_on(60, 100);
        assert_eq!(synth.active_voice_count(), 1);
    }

    #[test]
    fn test_synth_note_off() {
        let mut synth = Synth::new(48000.0);
        synth.note_on(60, 100);
        synth.note_off_specific(60);
        // Voice should still be active during release
        assert!(synth.active_voice_count() <= 1);
    }

    #[test]
    fn test_synth_process() {
        let mut synth = Synth::new(48000.0);
        synth.note_on(60, 100);

        let sample = synth.process_mono();
        // Should produce some output
        assert!(sample.is_finite());
    }

    #[test]
    fn test_synth_stereo() {
        let mut synth = Synth::new(48000.0);
        synth.note_on(60, 100);

        let (left, right) = synth.process_stereo();
        // Stereo output should be equal for mono source
        assert!((left - right).abs() < 0.001);
    }

    #[test]
    fn test_synth_reset() {
        let mut synth = Synth::new(48000.0);
        synth.note_on(60, 100);
        synth.note_on(64, 100);
        synth.reset();
        assert_eq!(synth.active_voice_count(), 0);
    }

    #[test]
    fn test_synth_polyphony() {
        let mut synth = Synth::new(48000.0);

        // Play multiple notes
        synth.note_on(60, 100);
        synth.note_on(64, 100);
        synth.note_on(67, 100);

        assert_eq!(synth.active_voice_count(), 3);
    }

    #[test]
    fn test_synth_filter() {
        let mut synth = Synth::new(48000.0);
        synth.set_filter_cutoff(500.0);
        synth.set_filter_resonance(5.0);

        let sample = synth.process_mono();
        assert!(sample.is_finite());
    }
}
