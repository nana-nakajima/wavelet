use super::*;

#[derive(Debug, Clone)]
pub struct AudioTrack {
    pub base: Track,
}

impl AudioTrack {
    pub fn new(id: u8, name: &str) -> Self {
        let mut base = Track::new(id, TrackType::Audio, name);
        base.base_width_filter = Some(BaseWidthFilter::new());
        AudioTrack { base }
    }
}

impl TrackBehavior for AudioTrack {
    fn process(&mut self, input: &[f32], output: &mut [f32], sample_rate: u32) {
        self.base.process(input, output, sample_rate);
    }

    fn trigger(&mut self, note: u8, velocity: f32) {
        self.base.trigger(note, velocity);
    }

    fn set_param(&mut self, param: &str, value: f32) {
        self.base.set_param(param, value);
    }

    fn get_param(&self, param: &str) -> f32 {
        self.base.get_param(param)
    }

    fn fx_slots(&self) -> usize {
        FX_SLOTS_AUDIO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_track_creation() {
        let track = AudioTrack::new(1, "AUDIO 1");
        assert_eq!(track.base.id, 1);
        assert_eq!(track.base.track_type, TrackType::Audio);
        assert_eq!(track.base.name, "AUDIO 1");
        assert!(track.base.base_width_filter.is_some());
        assert_eq!(track.base.insert_fx.len(), 2);
    }

    #[test]
    fn test_audio_track_mute() {
        let mut track = AudioTrack::new(1, "AUDIO 1");
        let input = vec![0.5; 1024];
        let mut output = vec![0.0; 1024];
        let sample_rate = 48000;

        track.set_param("mute", 127.0);
        track.process(&input, &mut output, sample_rate);
        assert!(output.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_audio_track_volume() {
        let mut track = AudioTrack::new(1, "AUDIO 1");
        track.set_param("volume", 127.0);
        let vol = track.base.get_param("volume");
        assert!((vol - 127.0).abs() < 0.1);
    }

    #[test]
    fn test_audio_track_pan() {
        let mut track = AudioTrack::new(1, "AUDIO 1");
        track.set_param("pan", 127.0);
        let pan = track.base.get_param("pan");
        assert!((pan - 126.0).abs() < 2.0); // Allow some tolerance
    }
}
