use super::*;

#[derive(Debug, Clone)]
pub struct SendTrack {
    pub base: Track,
}

impl SendTrack {
    pub fn new(id: u8, name: &str) -> Self {
        let mut base = Track::new(id, TrackType::Send, name);
        base.base_width_filter = None;
        SendTrack { base }
    }
}

impl TrackBehavior for SendTrack {
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
        FX_SLOTS_SEND
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_track_creation() {
        let track = SendTrack::new(13, "SEND 1");
        assert_eq!(track.base.id, 13);
        assert_eq!(track.base.track_type, TrackType::Send);
        assert_eq!(track.base.name, "SEND 1");
        assert!(track.base.base_width_filter.is_none());
        assert_eq!(track.base.insert_fx.len(), 1);
    }

    #[test]
    fn test_send_track_mute() {
        let mut track = SendTrack::new(13, "SEND 1");
        let input = vec![0.5; 1024];
        let mut output = vec![0.0; 1024];
        let sample_rate = 48000;

        track.set_param("mute", 127.0);
        track.process(&input, &mut output, sample_rate);
        assert!(output.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_send_track_volume() {
        let mut track = SendTrack::new(13, "SEND 1");
        track.set_param("volume", 127.0);
        let vol = track.base.get_param("volume");
        assert!((vol - 127.0).abs() < 0.1);
    }
}
