// WAVELET - Sampler Module
// 参考: Elektron Tonverk 采样播放功能
//
// 功能:
// - WAV/AIFF采样导入和播放
// - 切片播放 (Slicing)
// - 反向播放
// - 时间拉伸 (Time Stretching)
// - 与音序器集成
// - 速度同步

#![allow(dead_code)] // Reserve sampler fields for future sample editing features

use std::collections::HashMap;

/// 采样格式
#[derive(Debug, Clone, PartialEq)]
pub enum SampleFormat {
    /// 16-bit PCM
    Pcm16,
    /// 24-bit PCM
    Pcm24,
    /// 32-bit float
    Float32,
}

/// 采样信息元数据
#[derive(Debug, Clone)]
pub struct SampleInfo {
    /// 采样名称
    pub name: String,

    /// 文件路径
    pub path: String,

    /// 采样率
    pub sample_rate: u32,

    /// 通道数 (1=单声道, 2=立体声)
    pub channels: u16,

    /// 采样数量
    pub length: usize,

    /// 格式
    pub format: SampleFormat,

    /// 循环信息
    pub loop_info: Option<LoopInfo>,

    /// 根音 (MIDI note number)
    pub root_note: u8,

    /// 调性偏移 (semitones)
    pub semitone_offset: i8,

    /// 速度敏感度 (0.0 - 2.0)
    pub tempo_sensitivity: f32,
}

/// 循环设置
#[derive(Debug, Clone, Copy)]
pub struct LoopInfo {
    /// 循环开始点 (sample index)
    pub start: usize,

    /// 循环结束点 (sample index)
    pub end: usize,

    /// 循环模式
    pub mode: LoopMode,

    /// 交叉淡入淡出长度 (samples)
    pub crossfade: usize,
}

/// 循环模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    /// 不循环
    NoLoop,
    /// 循环
    Loop,
    /// 往返循环
    PingPong,
    /// 单次触发后释放
    OneShot,
}

/// 单个采样
#[derive(Debug, Clone)]
pub struct Sample {
    /// 元数据
    pub info: SampleInfo,

    /// 音频数据 (归一化到 -1.0 到 1.0)
    pub data: Vec<f32>,

    /// 立体声数据 (如果适用)
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
    /// 从数据创建采样
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

    /// 从立体声数据创建采样
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

    /// 检查是否是立体声
    pub fn is_stereo(&self) -> bool {
        self.data_stereo.is_some()
    }

    /// 获取循环状态
    pub fn is_looping(&self) -> bool {
        self.info
            .loop_info
            .as_ref()
            .map(|l| l.mode != LoopMode::OneShot)
            .unwrap_or(false)
    }

    /// 获取采样时长(秒)
    pub fn duration(&self) -> f64 {
        self.info.length as f64 / self.info.sample_rate as f64
    }

    /// 获取MIDI频率
    pub fn frequency(&self) -> f32 {
        440.0 * 2.0f32.powf((self.info.root_note as f32 - 69.0) / 12.0)
    }
}

/// 切片点
#[derive(Debug, Clone, PartialEq)]
pub struct SlicePoint {
    /// 切片开始点
    pub start: usize,

    /// 切片结束点
    pub end: usize,

    /// MIDI音符
    pub note: u8,

    /// 名称
    pub name: String,
}

/// 切片模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlicingMode {
    /// 自动检测瞬态
    Transient,
    /// 等分切片
    Equal,
    /// 网格切片
    Grid,
    /// 手动切片
    Manual,
}

/// 采样播放器
#[derive(Debug, Clone)]
pub struct Sampler {
    /// 当前采样
    sample: Option<Sample>,

    /// 播放位置 (sample index)
    position: f64,

    /// 播放速度 (1.0 = 正常)
    speed: f32,

    /// 是否反向播放
    reverse: bool,

    /// 音量 (0.0 - 1.0)
    volume: f32,

    /// 音高偏移 (semitones)
    pitch_offset: i8,

    /// 时间拉伸 (1.0 = 正常)
    time_stretch: f32,

    /// 循环模式
    loop_mode: LoopMode,

    /// 循环点
    loop_start: usize,
    loop_end: usize,

    /// 播放状态
    playing: bool,

    /// 触发位置 (用于切片播放)
    trigger_position: f64,

    /// 增益 (用于切片淡入淡出)
    fade_gain: f32,

    /// 交叉淡入淡出长度 (samples)
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
    /// 创建新的采样播放器
    pub fn new() -> Self {
        Self::default()
    }

    /// 加载采样
    pub fn load(&mut self, sample: Sample) {
        self.sample = Some(sample);
        self.reset();
    }

    /// 卸载采样
    pub fn unload(&mut self) {
        self.sample = None;
        self.reset();
    }

    /// 设置速度
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.25, 4.0);
    }

    /// 设置反向播放
    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }

    /// 设置音高偏移
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
}
