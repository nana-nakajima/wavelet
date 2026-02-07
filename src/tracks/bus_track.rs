use super::*;

#[derive(Debug, Clone)]
pub struct BusTrack {
    pub base: Track,
}

impl BusTrack {
    pub fn new(id: u8, name: &str) -> Self {
        let mut base = Track::new(id, TrackType::Bus, name);
        base.base_width_filter = Some(BaseWidthFilter::new());
        BusTrack { base }
    }
}

impl TrackBehavior for BusTrack {
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
        FX_SLOTS_BUS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_track_creation() {
        let track = BusTrack::new(9, "BUS 1");
        assert_eq!(track.base.id, 9);
        assert_eq!(track.base.track_type, TrackType::Bus);
        assert_eq!(track.base.name, "BUS 1");
        assert!(track.base.base_width_filter.is_some());
        assert_eq!(track.base.insert_fx.len(), 2);
    }

    #[test]
    fn test_bus_track_mute() {
        let mut track = BusTrack::new(9, "BUS 1");
        let input = vec![0.5; 1024];
        let mut output = vec![0.0; 1024];
        let sample_rate = 48000;

        track.set_param("mute", 127.0);
        track.process(&input, &mut output, sample_rate);
        assert!(output.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_bus_track_send_levels() {
        let mut track = BusTrack::new(9, "BUS 1");
        assert_eq!(track.base.send_levels.len(), 3);

        track.base.send_levels[0] = 0.5;
        track.base.send_levels[1] = 0.75;
        track.base.send_levels[2] = 1.0;

        assert_eq!(track.base.send_levels[0], 0.5);
        assert_eq!(track.base.send_levels[1], 0.75);
        assert_eq!(track.base.send_levels[2], 1.0);
    }
}
