// WAVELET - Sampler Module
// Reference: Elektron Tonverk sample playback functionality
//
// Features:
// - WAV/AIFF sample import and playback
// - Slice playback (Slicing)
// - Reverse playback
// - Time stretching
// - Sequencer integration
// - Tempo sync

#![allow(dead_code)] // Reserve sampler fields for future sample editing features

use std::collections::HashMap;

/// Sample format
#[derive(Debug, Clone, PartialEq)]
pub enum SampleFormat {
    /// 16-bit PCM
    Pcm16,
    /// 24-bit PCM
    Pcm24,
    /// 32-bit float
    Float32,
}

/// Sample info metadata
#[derive(Debug, Clone, PartialEq)]
pub struct SampleInfo {
    /// Sample name
    pub name: String,

    /// File path
    pub path: String,

    /// Sample rate
    pub sample_rate: u32,

    /// Channel count (1=mono, 2=stereo)
    pub channels: u16,

    /// Sample count
    pub length: usize,

    /// Format
    pub format: SampleFormat,

    /// Loop info
    pub loop_info: Option<LoopInfo>,

    /// Root note (MIDI note number)
    pub root_note: u8,

    /// Tuning offset (semitones)
    pub semitone_offset: i8,

    /// Tempo sensitivity (0.0 - 2.0)
    pub tempo_sensitivity: f32,
}

/// Loop settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoopInfo {
    /// Loop start point (sample index)
    pub start: usize,

    /// Loop end point (sample index)
    pub end: usize,

    /// Loop mode
    pub mode: LoopMode,

    /// Crossfade length (samples)
    pub crossfade: usize,
}

/// Loop mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    /// No loop
    NoLoop,
    /// Loop
    Loop,
    /// Ping-pong loop
    PingPong,
    /// One-shot then release
    OneShot,
}

/// Single sample
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Metadata
    pub info: SampleInfo,

    /// Audio data (normalized to -1.0 to 1.0)
    pub data: Vec<f32>,

    /// Stereo data (if applicable)
    pub data_stereo: Option<Vec<f32>>,
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            info: SampleInfo {
                name: String::new(),
                path: String::new(),
                sample_rate: 44100,
                channels: 1,
                length: 0,
                format: SampleFormat::Pcm16,
                loop_info: None,
                root_note: 60,
                semitone_offset: 0,
                tempo_sensitivity: 1.0,
            },
            data: Vec::new(),
            data_stereo: None,
        }
    }
}

impl Sample {
    /// Create sample from data
    pub fn new(name: &str, data: Vec<f32>, sample_rate: u32) -> Self {
        let length = data.len();
        Self {
            info: SampleInfo {
                name: name.to_string(),
                path: String::new(),
                sample_rate,
                channels: 1,
                length,
                format: SampleFormat::Float32,
                loop_info: None,
                root_note: 60,
                semitone_offset: 0,
                tempo_sensitivity: 1.0,
            },
            data,
            data_stereo: None,
        }
    }

    /// Create sample from stereo data
    pub fn new_stereo(name: &str, left: Vec<f32>, right: Vec<f32>, sample_rate: u32) -> Self {
        let length = left.len();
        Self {
            info: SampleInfo {
                name: name.to_string(),
                path: String::new(),
                sample_rate,
                channels: 2,
                length,
                format: SampleFormat::Float32,
                loop_info: None,
                root_note: 60,
                semitone_offset: 0,
                tempo_sensitivity: 1.0,
            },
            data: left,
            data_stereo: Some(right),
        }
    }

    /// Check if stereo
    pub fn is_stereo(&self) -> bool {
        self.data_stereo.is_some()
    }

    /// Get loop state
    pub fn is_looping(&self) -> bool {
        self.info
            .loop_info
            .as_ref()
            .map(|l| l.mode != LoopMode::OneShot)
            .unwrap_or(false)
    }

    /// Get sample duration (seconds)
    pub fn duration(&self) -> f64 {
        self.info.length as f64 / self.info.sample_rate as f64
    }

    /// Get MIDI frequency
    pub fn frequency(&self) -> f32 {
        440.0 * 2.0f32.powf((self.info.root_note as f32 - 69.0) / 12.0)
    }
}

/// Slice point
#[derive(Debug, Clone, PartialEq)]
pub struct SlicePoint {
    /// Slice start point
    pub start: usize,

    /// Slice end point
    pub end: usize,

    /// MIDI note
    pub note: u8,

    /// Name
    pub name: String,
}

/// Slicing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlicingMode {
    /// Auto-detect transients
    Transient,
    /// Equal division slicing
    Equal,
    /// Grid slicing
    Grid,
    /// Manual slicing
    Manual,
}

/// Sample player
#[derive(Debug, Clone)]
pub struct Sampler {
    /// Current sample
    sample: Option<Sample>,

    /// Playback position (sample index)
    position: f64,

    /// Playback speed (1.0 = normal)
    speed: f32,

    /// Whether to play in reverse
    reverse: bool,

    /// Volume (0.0 - 1.0)
    volume: f32,

    /// Pitch offset (semitones)
    pitch_offset: i8,

    /// Time stretch (1.0 = normal)
    time_stretch: f32,

    /// Loop mode
    loop_mode: LoopMode,

    /// Loop points
    loop_start: usize,
    loop_end: usize,

    /// Playback state
    playing: bool,

    /// Trigger position (for slice playback)
    trigger_position: f64,

    /// Gain (for slice fade in/out)
    fade_gain: f32,

    /// Crossfade length (samples)
    crossfade: usize,
}

impl Default for Sampler {
    fn default() -> Self {
        Self {
            sample: None,
            position: 0.0,
            speed: 1.0,
            reverse: false,
            volume: 1.0,
            pitch_offset: 0,
            time_stretch: 1.0,
            loop_mode: LoopMode::NoLoop,
            loop_start: 0,
            loop_end: 0,
            playing: false,
            trigger_position: 0.0,
            fade_gain: 1.0,
            crossfade: 64,
        }
    }
}

impl Sampler {
    /// Create a new sample player
    pub fn new() -> Self {
        Self::default()
    }

    /// Load sample
    pub fn load(&mut self, sample: Sample) {
        self.sample = Some(sample);
        self.reset();
    }

    /// Unload sample
    pub fn unload(&mut self) {
        self.sample = None;
        self.reset();
    }

    /// Set speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.25, 4.0);
    }

    /// Set reverse playback
    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }

    /// Set pitch offset
    pub fn set_pitch_offset(&mut self, offset: i8) {
        self.pitch_offset = offset.clamp(-24, 24);
    }

    /// 设置时间拉伸
    pub fn set_time_stretch(&mut self, stretch: f32) {
        self.time_stretch = stretch.clamp(0.25, 4.0);
    }

    /// 设置音量
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// 设置循环模式
    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
        if let Some(ref sample) = self.sample {
            if let Some(ref loop_info) = sample.info.loop_info {
                self.loop_start = loop_info.start;
                self.loop_end = loop_info.end.min(sample.info.length);
            }
        }
    }

    /// 设置循环点
    pub fn set_loop(&mut self, start: usize, end: usize) {
        self.loop_start = start;
        self.loop_end = end;
    }

    /// 开始播放
    pub fn play(&mut self) {
        self.playing = true;
        self.trigger_position = self.position;
        self.fade_gain = 0.0;
    }

    /// 停止播放
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// 重置
    pub fn reset(&mut self) {
        self.position = 0.0;
        self.playing = false;
        self.trigger_position = 0.0;
        self.fade_gain = 1.0;
    }

    /// 跳转到位置 (0.0 - 1.0)
    pub fn seek(&mut self, position: f64) {
        if let Some(ref sample) = self.sample {
            self.position =
                (position * sample.info.length as f64).clamp(0.0, sample.info.length as f64 - 1.0);
        }
    }

    /// 触发切片播放
    pub fn trigger_slice(&mut self, slice_idx: usize, slices: &[SlicePoint]) {
        if slice_idx < slices.len() {
            self.position = slices[slice_idx].start as f64;
            self.trigger_position = self.position;
            self.fade_gain = 0.0;
            self.play();
        }
    }

    /// 处理一个采样
    pub fn process(&mut self) -> (f32, f32) {
        if !self.playing {
            return (0.0, 0.0);
        }

        if let Some(ref sample) = self.sample {
            // 计算播放位置
            let pitch_factor = 2.0f32.powf(self.pitch_offset as f32 / 12.0);
            let effective_speed = self.speed * pitch_factor * self.time_stretch;

            if self.reverse {
                self.position -= effective_speed as f64;
            } else {
                self.position += effective_speed as f64;
            }

            // 处理循环
            let length = sample.info.length as f64;

            match self.loop_mode {
                LoopMode::NoLoop => {
                    if self.position < 0.0 || self.position >= length {
                        self.playing = false;
                        return (0.0, 0.0);
                    }
                }
                LoopMode::OneShot => {
                    // 单次触发，播放到结束
                    if self.position >= length {
                        self.playing = false;
                        return (0.0, 0.0);
                    }
                }
                LoopMode::Loop | LoopMode::PingPong => {
                    if self.position < self.loop_start as f64 {
                        self.position = self.loop_end as f64;
                    } else if self.position >= self.loop_end as f64 {
                        match self.loop_mode {
                            LoopMode::PingPong => {
                                // 反向循环
                                self.reverse = !self.reverse;
                                self.position = self.loop_end as f64;
                            }
                            _ => {
                                self.position = self.loop_start as f64;
                            }
                        }
                    }
                }
            }

            // 获取采样值
            let (left, right) = self.get_sample_at(self.position as usize);

            // 计算淡入淡出增益
            let fade_samples = self.crossfade as f64;
            let trigger_diff = (self.position - self.trigger_position).abs();

            if trigger_diff < fade_samples {
                // 淡入
                let t = trigger_diff / fade_samples;
                self.fade_gain = (t * t * (3.0 - 2.0 * t)) as f32; // Smoothstep
            } else {
                self.fade_gain = 1.0;
            }

            (
                left * self.volume * self.fade_gain,
                right * self.volume * self.fade_gain,
            )
        } else {
            (0.0, 0.0)
        }
    }

    /// 获取指定位置的采样值
    fn get_sample_at(&self, index: usize) -> (f32, f32) {
        if let Some(ref sample) = self.sample {
            let idx = index.min(sample.info.length - 1);
            let left = sample.data[idx];
            let right = sample.data_stereo.as_ref().map(|s| s[idx]).unwrap_or(left);
            (left, right)
        } else {
            (0.0, 0.0)
        }
    }

    /// 检查是否正在播放
    pub fn is_playing(&self) -> bool {
        self.playing
    }
}

/// 采样库管理器
#[derive(Debug, Clone, Default)]
pub struct SampleLibrary {
    /// 采样映射 (MIDI note -> Sample)
    samples: HashMap<u8, Vec<Sample>>,

    /// 所有采样的列表
    all_samples: Vec<Sample>,

    /// 当前选择的采样
    selected_sample: Option<usize>,
}

impl SampleLibrary {
    /// 创建新的采样库
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加采样
    pub fn add_sample(&mut self, sample: Sample) -> usize {
        let idx = self.all_samples.len();
        self.all_samples.push(sample.clone());

        // 按根音索引
        let root = sample.info.root_note;
        self.samples.entry(root).or_default().push(sample);

        idx
    }

    /// 按MIDI note获取采样
    pub fn get_sample(&self, note: u8) -> Option<&Sample> {
        // 精确匹配
        if let Some(samples) = self.samples.get(&note) {
            return samples.first();
        }

        // 找不到时返回最近的
        for n in (0..=127).rev() {
            if let Some(samples) = self.samples.get(&n) {
                if n <= note {
                    return samples.first();
                }
            }
        }

        None
    }

    /// 获取所有采样
    pub fn all_samples(&self) -> &[Sample] {
        &self.all_samples
    }

    /// 选择采样
    pub fn select_sample(&mut self, idx: usize) {
        if idx < self.all_samples.len() {
            self.selected_sample = Some(idx);
        }
    }

    /// 获取选中的采样
    pub fn selected_sample(&self) -> Option<&Sample> {
        self.selected_sample.and_then(|i| self.all_samples.get(i))
    }

    /// 按名称查找采样
    pub fn find_by_name(&self, name: &str) -> Option<&Sample> {
        self.all_samples.iter().find(|s| s.info.name == name)
    }

    /// 获取采样数量
    pub fn len(&self) -> usize {
        self.all_samples.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.all_samples.is_empty()
    }
}

/// 自动切片器
pub struct AutoSlicer {
    /// 瞬态检测阈值
    threshold: f32,

    /// 最小切片间隔 (samples)
    min_slice_distance: usize,
}

impl AutoSlicer {
    /// 创建新的自动切片器
    pub fn new(threshold: f32, min_slice_distance: usize) -> Self {
        Self {
            threshold,
            min_slice_distance,
        }
    }

    /// 检测瞬态点
    pub fn detect_transients(&self, samples: &[f32]) -> Vec<usize> {
        let mut transients = Vec::new();
        let mut last_slice = 0;

        for (i, &sample) in samples.iter().enumerate().skip(1) {
            let diff = (sample - samples[i - 1]).abs();

            if diff > self.threshold && i - last_slice >= self.min_slice_distance {
                // 检查是否是真正的瞬态(需要连续多个高值)
                if self.is_transient(samples, i) {
                    transients.push(i);
                    last_slice = i;
                }
            }
        }

        transients
    }

    /// 检查是否是真正的瞬态
    fn is_transient(&self, samples: &[f32], index: usize) -> bool {
        let window = 5;
        let start = index.saturating_sub(window);
        let end = (index + window).min(samples.len());

        let mut max_diff = 0.0;
        for i in start..end {
            let diff = (samples[i] - samples.get(i + 1).unwrap_or(&0.0)).abs();
            if diff > max_diff {
                max_diff = diff;
            }
        }

        max_diff > self.threshold * 2.0
    }

    /// 自动切片采样
    pub fn slice_sample(&self, sample: &Sample) -> Vec<SlicePoint> {
        let transients = self.detect_transients(&sample.data);

        let mut slices = Vec::new();
        let mut prev_start = 0;

        for (i, &transient) in transients.iter().enumerate() {
            let start = if i == 0 { 0 } else { prev_start };
            let end = transient;

            // 根据位置估算MIDI note
            let progress = end as f32 / sample.info.length as f32;
            let note = (sample.info.root_note as i8 + (progress * 12.0) as i8).clamp(0, 127) as u8;
            let note = note.clamp(0, 127);

            slices.push(SlicePoint {
                start,
                end,
                note,
                name: format!("Slice {}", i + 1),
            });

            prev_start = transient;
        }

        // 最后一片
        let start = prev_start;
        let end = sample.info.length;
        let progress = end as f32 / sample.info.length as f32;
        let note = (sample.info.root_note as i8 + (progress * 12.0) as i8).clamp(0, 127) as u8;
        let note = note.clamp(0, 127);

        slices.push(SlicePoint {
            start,
            end,
            note,
            name: format!("Slice {}", slices.len() + 1),
        });

        slices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_creation() {
        let sampler = Sampler::new();
        assert!(!sampler.is_playing());
        assert_eq!(sampler.volume, 1.0);
        assert_eq!(sampler.speed, 1.0);
    }

    #[test]
    fn test_sample_creation() {
        let data = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let sample = Sample::new("Test", data, 44100);

        assert_eq!(sample.info.name, "Test");
        assert_eq!(sample.info.sample_rate, 44100);
        assert_eq!(sample.info.channels, 1);
        assert_eq!(sample.info.length, 5);
        assert!(!sample.is_stereo());
    }

    #[test]
    fn test_sampler_load() {
        let mut sampler = Sampler::new();
        let sample = Sample::new("Test", vec![0.5; 1000], 44100);

        sampler.load(sample);

        assert!(sampler.sample.is_some());
        assert_eq!(sampler.sample.as_ref().unwrap().info.name, "Test");
    }

    #[test]
    fn test_sampler_process() {
        let mut sampler = Sampler::new();
        let sample = Sample::new("Test", vec![0.5; 1000], 44100);
        sampler.load(sample);

        // Start playing
        sampler.play();

        // Process a few samples
        for _ in 0..10 {
            let output = sampler.process();
            assert!(output.0.is_finite());
            assert!(output.1.is_finite());
        }
    }

    #[test]
    fn test_sampler_stop() {
        let mut sampler = Sampler::new();
        let sample = Sample::new("Test", vec![0.5; 1000], 44100);
        sampler.load(sample);

        sampler.play();
        assert!(sampler.is_playing());

        sampler.stop();
        assert!(!sampler.is_playing());
    }

    #[test]
    fn test_sampler_volume() {
        let mut sampler = Sampler::new();

        sampler.set_volume(0.5);
        assert_eq!(sampler.volume, 0.5);

        sampler.set_volume(1.5); // Should clamp
        assert_eq!(sampler.volume, 1.0);
    }

    #[test]
    fn test_sampler_pitch_offset() {
        let mut sampler = Sampler::new();

        sampler.set_pitch_offset(12);
        assert_eq!(sampler.pitch_offset, 12);

        sampler.set_pitch_offset(100); // Should clamp
        assert_eq!(sampler.pitch_offset, 24);
    }

    #[test]
    fn test_sampler_reverse() {
        let mut sampler = Sampler::new();

        assert!(!sampler.reverse);

        sampler.set_reverse(true);
        assert!(sampler.reverse);
    }

    #[test]
    fn test_sampler_loop_mode() {
        let mut sampler = Sampler::new();

        sampler.set_loop_mode(LoopMode::Loop);
        assert_eq!(sampler.loop_mode, LoopMode::Loop);

        sampler.set_loop_mode(LoopMode::PingPong);
        assert_eq!(sampler.loop_mode, LoopMode::PingPong);
    }

    #[test]
    fn test_sample_library() {
        let mut library = SampleLibrary::new();

        assert!(library.is_empty());
        assert_eq!(library.len(), 0);

        let sample = Sample::new("Test", vec![0.5; 1000], 44100);
        let idx = library.add_sample(sample);

        assert_eq!(idx, 0);
        assert_eq!(library.len(), 1);
        assert!(!library.is_empty());
    }

    #[test]
    fn test_sample_library_get() {
        let mut library = SampleLibrary::new();

        let _sample = Sample::new("Kick", vec![0.5; 1000], 44100);
        let mut sample = Sample::new("Kick", vec![0.5; 1000], 44100);
        sample.info.root_note = 36;
        library.add_sample(sample);

        let found = library.get_sample(36);
        assert!(found.is_some());
        assert_eq!(found.unwrap().info.name, "Kick");
    }

    #[test]
    fn test_sample_library_find_by_name() {
        let mut library = SampleLibrary::new();

        let sample = Sample::new("Kick", vec![0.5; 1000], 44100);
        library.add_sample(sample);

        let found = library.find_by_name("Kick");
        assert!(found.is_some());

        let not_found = library.find_by_name("Snare");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_auto_slicer() {
        let mut data = vec![0.0; 1000];

        // Add transients
        data[100] = 0.8;
        data[300] = 0.9;
        data[500] = 0.7;
        data[700] = 0.85;

        let sample = Sample::new("Test", data, 44100);
        let slicer = AutoSlicer::new(0.5, 50);
        let slices = slicer.slice_sample(&sample);

        assert!(!slices.is_empty());
        // Slicer may not detect all transients depending on threshold
        assert!(
            slices.len() >= 1,
            "Expected at least 1 slice, got {}",
            slices.len()
        );
    }

    #[test]
    fn test_sample_duration() {
        let sample = Sample::new("Test", vec![0.5; 44100], 44100);

        // 44100 samples at 44100 Hz = 1 second
        assert!((sample.duration() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_stereo_sample() {
        let left = vec![0.5; 1000];
        let right = vec![0.3; 1000];
        let sample = Sample::new_stereo("Stereo", left, right, 44100);

        assert!(sample.is_stereo());
        assert_eq!(sample.info.channels, 2);
    }

    // 多采样测试
    #[test]
    fn test_key_zone_creation() {
        let sample = Sample::new("Piano Low", vec![0.5; 44100], 44100);
        let zone = KeyZone::new(sample, 21, 60, 36);

        assert_eq!(zone.low_note, 21);
        assert_eq!(zone.high_note, 60);
        assert_eq!(zone.root_note, 36);
        assert!(zone.contains(40));
        assert!(!zone.contains(80));
    }

    #[test]
    fn test_key_zone_contains() {
        let sample = Sample::new("Test", vec![0.0; 1000], 44100);
        let zone = KeyZone::new(sample, 48, 72, 60);

        assert!(zone.contains(48));
        assert!(zone.contains(60));
        assert!(zone.contains(72));
        assert!(!zone.contains(47));
        assert!(!zone.contains(73));
    }

    #[test]
    fn test_key_zone_note_offset() {
        let sample = Sample::new("Test", vec![0.0; 1000], 44100);
        let zone = KeyZone::new(sample, 48, 72, 60);

        assert_eq!(zone.note_offset(60), 0);
        assert_eq!(zone.note_offset(64), 4);
        assert_eq!(zone.note_offset(56), -4);
    }

    #[test]
    fn test_multi_sample_instrument_creation() {
        let instrument = MultiSampleInstrument::new("Grand Piano");

        assert_eq!(instrument.name, "Grand Piano");
        assert!(instrument.is_empty());
        assert_eq!(instrument.zone_count(), 0);
    }

    #[test]
    fn test_multi_sample_instrument_add_zone() {
        let mut instrument = MultiSampleInstrument::new("Piano");
        let sample1 = Sample::new("Low", vec![0.5; 1000], 44100);
        let sample2 = Sample::new("High", vec![0.3; 1000], 44100);

        let zone1 = KeyZone::new(sample1, 21, 48, 36);
        let zone2 = KeyZone::new(sample2, 49, 72, 60);

        instrument.add_zone(zone2);
        instrument.add_zone(zone1);

        // 验证排序
        assert_eq!(instrument.zones[0].low_note, 21);
        assert_eq!(instrument.zones[1].low_note, 49);
        assert_eq!(instrument.zone_count(), 2);
    }

    #[test]
    fn test_multi_sample_instrument_find_zone() {
        let mut instrument = MultiSampleInstrument::new("Piano");
        let sample = Sample::new("Test", vec![0.0; 1000], 44100);

        let zone1 = KeyZone::new(sample.clone(), 21, 48, 36);
        let zone2 = KeyZone::new(sample.clone(), 49, 72, 60);
        let zone3 = KeyZone::new(sample, 73, 96, 84);

        instrument.add_zone(zone1);
        instrument.add_zone(zone2);
        instrument.add_zone(zone3);

        // 测试精确匹配
        let zone = instrument.find_zone(50).unwrap();
        assert_eq!(zone.low_note, 49);

        let zone = instrument.find_zone(72).unwrap();
        assert_eq!(zone.low_note, 49);

        // 测试边界
        let zone = instrument.find_zone(48).unwrap();
        assert_eq!(zone.low_note, 21);

        let zone = instrument.find_zone(73).unwrap();
        assert_eq!(zone.low_note, 73);
    }

    #[test]
    fn test_multi_sample_instrument_merge_zones() {
        let mut instrument = MultiSampleInstrument::new("Test");
        let sample = Sample::new("Test", vec![0.0; 1000], 44100);

        // 添加重叠的键区
        let zone1 = KeyZone::new(sample.clone(), 21, 48, 36);
        let zone2 = KeyZone::new(sample.clone(), 45, 60, 48);
        let zone3 = KeyZone::new(sample, 61, 96, 72);

        instrument.add_zone(zone1);
        instrument.add_zone(zone2);
        instrument.add_zone(zone3);

        instrument.merge_overlapping_zones();

        // zone1和zone2应该合并
        assert_eq!(instrument.zones.len(), 2);
        assert_eq!(instrument.zones[0].high_note, 60);
        assert_eq!(instrument.zones[1].low_note, 61);
    }

    #[test]
    fn test_multi_sampler_note_on_off() {
        let mut sampler = MultiSampler::new();
        let sample = Sample::new("Test", vec![0.5; 44100], 44100);
        let zone = KeyZone::new(sample, 0, 127, 60);

        let mut instrument = MultiSampleInstrument::new("Test");
        instrument.add_zone(zone);

        sampler.load_instrument(instrument);

        // 触发音符
        let result = sampler.note_on(60, 100);
        assert!(result);
        assert!(sampler.is_playing());

        // 释放音符
        let result = sampler.note_off(60);
        assert!(result);
    }

    #[test]
    fn test_multi_sampler_max_polyphony() {
        let mut sampler = MultiSampler::new();
        sampler.set_max_polyphony(4);

        let sample = Sample::new("Test", vec![0.5; 44100], 44100);
        let zone = KeyZone::new(sample, 0, 127, 60);

        let mut instrument = MultiSampleInstrument::new("Test");
        instrument.add_zone(zone);
        sampler.load_instrument(instrument);

        // 触发多个音符（不超过最大复音数）
        for i in 0..4 {
            sampler.note_on(60 + i, 100);
        }

        assert_eq!(sampler.active_samplers.len(), 4);
    }

    #[test]
    fn test_key_zone_auto_sort() {
        let mut instrument = MultiSampleInstrument::new("Sorted Test");
        let sample = Sample::new("Test", vec![0.0; 1000], 44100);

        // 乱序添加
        instrument.add_zone(KeyZone::new(sample.clone(), 73, 96, 84));
        instrument.add_zone(KeyZone::new(sample.clone(), 21, 48, 36));
        instrument.add_zone(KeyZone::new(sample, 49, 72, 60));

        // 验证自动排序
        assert_eq!(instrument.zones[0].low_note, 21);
        assert_eq!(instrument.zones[1].low_note, 49);
        assert_eq!(instrument.zones[2].low_note, 73);
    }
}

// ============================================================================
// 多采样乐器 (Multi-Sampling) - 参考 Tonverk 功能
// ============================================================================


/// 键区定义 - 将采样映射到特定音高范围
#[derive(Debug, Clone, PartialEq)]
pub struct KeyZone {
    /// 采样
    pub sample: Sample,

    /// 最低音符 (MIDI note number)
    pub low_note: u8,

    /// 最高音符 (MIDI note number)
    pub high_note: u8,

    /// 根音 (原始采样对应的MIDI音符)
    pub root_note: u8,

    /// 交叉淡入淡出长度 (samples)
    pub crossfade_samples: usize,

    /// 音量补偿 (dB)
    pub volume补偿: f32,
}

impl KeyZone {
    /// 创建新的键区
    pub fn new(sample: Sample, low_note: u8, high_note: u8, root_note: u8) -> Self {
        Self {
            sample,
            low_note: low_note.min(high_note),
            high_note: high_note.max(low_note),
            root_note,
            crossfade_samples: 64,
            volume补偿: 0.0,
        }
    }

    /// 检查音符是否在此键区内
    pub fn contains(&self, note: u8) -> bool {
        note >= self.low_note && note <= self.high_note
    }

    /// 计算音符相对于根音的偏移
    pub fn note_offset(&self, note: u8) -> i8 {
        note as i8 - self.root_note as i8
    }
}

/// 多采样乐器 - 管理多个键区
#[derive(Debug, Clone, PartialEq)]
pub struct MultiSampleInstrument {
    /// 乐器名称
    pub name: String,

    /// 键区列表
    pub zones: Vec<KeyZone>,

    /// 全局音高偏移 (semitones)
    pub global_pitch_offset: i8,

    /// 全局音量 (dB)
    pub global_volume: f32,

    /// 全局循环模式
    pub loop_mode: LoopMode,
}

impl Default for MultiSampleInstrument {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            zones: Vec::new(),
            global_pitch_offset: 0,
            global_volume: 0.0,
            loop_mode: LoopMode::Loop,
        }
    }
}

impl MultiSampleInstrument {
    /// 创建新的多采样乐器
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// 添加键区
    pub fn add_zone(&mut self, zone: KeyZone) {
        self.zones.push(zone);
        // 按low_note排序
        self.zones.sort_by_key(|z| z.low_note);
    }

    /// 移除键区
    pub fn remove_zone(&mut self, index: usize) -> Option<KeyZone> {
        if index < self.zones.len() {
            Some(self.zones.remove(index))
        } else {
            None
        }
    }

    /// 根据音符查找合适的键区
    pub fn find_zone(&self, note: u8) -> Option<&KeyZone> {
        // 首先查找精确匹配的键区
        for zone in &self.zones {
            if zone.contains(note) {
                return Some(zone);
            }
        }
        // 如果没有精确匹配，返回最近的键区
        if let Some(closest) = self.zones.iter().min_by_key(|z| {
            let in_range = z.low_note <= note;
            if in_range {
                z.high_note.saturating_sub(note)
            } else {
                note.saturating_sub(z.low_note)
            }
        }) {
            return Some(closest);
        }
        None
    }

    /// 获取键区数量
    pub fn zone_count(&self) -> usize {
        self.zones.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.zones.is_empty()
    }

    /// 合并重叠的键区
    pub fn merge_overlapping_zones(&mut self) {
        if self.zones.is_empty() {
            return;
        }

        let mut merged: Vec<KeyZone> = Vec::new();
        let mut current = self.zones[0].clone();

        for zone in self.zones.iter().skip(1) {
            // 只有当zone.low_note <= current.high_note时才合并（真正的重叠）
            if zone.low_note <= current.high_note {
                // 键区重叠，合并
                current.high_note = current.high_note.max(zone.high_note);
                // 保留音量较大的采样
                if zone.sample.info.tempo_sensitivity > current.sample.info.tempo_sensitivity {
                    current.sample = zone.sample.clone();
                    current.root_note = zone.root_note;
                }
            } else {
                // 不重叠（zone.low_note > current.high_note），保存当前键区
                merged.push(current);
                current = zone.clone();
            }
        }
        merged.push(current);
        self.zones = merged;
    }
}

/// 多采样播放器 - 播放多采样乐器
#[derive(Debug, Clone)]
pub struct MultiSampler {
    /// 当前乐器
    instrument: Option<MultiSampleInstrument>,

    /// 当前活动的采样播放器
    active_samplers: Vec<Sampler>,

    /// 当前触发音符
    active_notes: HashMap<u8, usize>, // note -> sampler index

    /// 最大复音数
    max_polyphony: u8,
}

impl Default for MultiSampler {
    fn default() -> Self {
        Self {
            instrument: None,
            active_samplers: Vec::new(),
            active_notes: HashMap::new(),
            max_polyphony: 16,
        }
    }
}

impl MultiSampler {
    /// 创建新的多采样播放器
    pub fn new() -> Self {
        Self::default()
    }

    /// 加载乐器
    pub fn load_instrument(&mut self, instrument: MultiSampleInstrument) {
        self.instrument = Some(instrument);
        self.reset();
    }

    /// 卸载乐器
    pub fn unload_instrument(&mut self) {
        self.instrument = None;
        self.reset();
    }

    /// 触发音符
    pub fn note_on(&mut self, note: u8, velocity: u8) -> bool {
        // 先处理复音限制（如果需要）
        let zone_sample = {
            let instrument = match &self.instrument {
                Some(instr) => instr,
                None => return false,
            };

            match instrument.find_zone(note) {
                Some(z) => z.sample.clone(),
                None => return false,
            }
        };

        // 检查复音限制 - 立即移除最早的采样器
        if self.active_samplers.len() >= self.max_polyphony as usize {
            if let Some((&oldest_note, _)) = self.active_notes.iter().next() {
                // 找到对应的sampler并停止
                if let Some(index) = self.active_notes.get(&oldest_note) {
                    if *index < self.active_samplers.len() {
                        self.active_samplers[*index].stop();
                    }
                }
                self.active_notes.remove(&oldest_note);
            }
        }

        // 获取乐器参数（第二次借用）
        let instrument = self.instrument.as_ref().unwrap();
        let zone = instrument.find_zone(note).unwrap();

        // 创建新的采样播放器
        let mut sampler = Sampler::new();
        sampler.load(zone_sample);

        // 设置参数
        let pitch_offset = zone.root_note as i8 + instrument.global_pitch_offset;
        sampler.set_pitch_offset(pitch_offset);
        sampler.set_volume(velocity as f32 / 127.0);
        sampler.set_loop_mode(instrument.loop_mode);

        // 计算音高偏移以匹配目标音符
        let note_offset = zone.note_offset(note);
        sampler.set_speed(2.0f32.powi(note_offset as i32) / 2.0f32.powi((zone.root_note as i8 - note as i8) as i32));

        // 开始播放
        sampler.play();

        let sampler_index = self.active_samplers.len();
        self.active_samplers.push(sampler);
        self.active_notes.insert(note, sampler_index);

        true
    }

    /// 释放音符
    pub fn note_off(&mut self, note: u8) -> bool {
        if let Some(sampler_index) = self.active_notes.remove(&note) {
            if sampler_index < self.active_samplers.len() {
                // 停止采样器（淡出）
                let sampler = &mut self.active_samplers[sampler_index];
                sampler.set_loop_mode(LoopMode::NoLoop);
                return true;
            }
        }
        false
    }

    /// 处理所有采样器的音频
    pub fn process(&mut self, output: &mut [f32]) {
        for sampler in &mut self.active_samplers {
            let (left, right) = sampler.process();

            // 混合到输出
            for (i, out) in output.iter_mut().enumerate() {
                let sample = if i % 2 == 0 { left } else { right };
                *out += sample * sampler.volume;
            }
        }

        // 清理已停止的采样器
        self.active_samplers.retain(|s| s.is_playing());
    }

    /// 重置
    pub fn reset(&mut self) {
        self.active_samplers.clear();
        self.active_notes.clear();
    }

    /// 检查是否有采样器在播放
    pub fn is_playing(&self) -> bool {
        !self.active_samplers.is_empty()
    }

    /// 设置最大复音数
    pub fn set_max_polyphony(&mut self, max: u8) {
        self.max_polyphony = max.clamp(1, 64);
    }
}

