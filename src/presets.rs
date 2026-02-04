//!
//! Preset Manager - 预设管理和分类系统
//!
//! # 功能
//! - 预设加载和保存
//! - 预设分类系统
//! - 预设搜索功能
//! - 预设收藏夹
//! - 预设预览
//!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

/// 预设分类
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PresetCategory {
    /// 基础预设
    #[default]
    Basic,
    /// 贝斯音色
    Bass,
    /// 主音/Lead
    Lead,
    /// 铺垫/Pad
    Pad,
    /// 键盘音色
    Keys,
    /// 弦乐
    Strings,
    /// 钟声
    Bell,
    /// 特殊效果
    Effect,
    /// 鼓组
    Drums,
    /// 环境音
    Ambient,
    /// 其它
    Other,
}

impl PresetCategory {
    /// 获取分类名称
    pub fn name(&self) -> &str {
        match self {
            PresetCategory::Basic => "Basic",
            PresetCategory::Bass => "Bass",
            PresetCategory::Lead => "Lead",
            PresetCategory::Pad => "Pad",
            PresetCategory::Keys => "Keys",
            PresetCategory::Strings => "Strings",
            PresetCategory::Bell => "Bell",
            PresetCategory::Effect => "Effect",
            PresetCategory::Drums => "Drums",
            PresetCategory::Ambient => "Ambient",
            PresetCategory::Other => "Other",
        }
    }

    /// 获取所有分类
    pub fn all_categories() -> Vec<Self> {
        vec![
            PresetCategory::Basic,
            PresetCategory::Bass,
            PresetCategory::Lead,
            PresetCategory::Pad,
            PresetCategory::Keys,
            PresetCategory::Strings,
            PresetCategory::Bell,
            PresetCategory::Effect,
            PresetCategory::Drums,
            PresetCategory::Ambient,
            PresetCategory::Other,
        ]
    }
}

/// 预设参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetParameters {
    /// 主音量
    pub volume: f32,
    /// 滤波器截止频率
    pub filter_cutoff: f32,
    /// 滤波器共振
    pub filter_resonance: f32,
    /// 起音时间 (Attack)
    pub attack: f32,
    /// 释音时间 (Release)
    pub release: f32,
    /// 波形类型
    pub waveform: String,
    /// 额外参数
    #[serde(default)]
    pub extra: HashMap<String, f32>,
}

impl Default for PresetParameters {
    fn default() -> Self {
        Self {
            volume: 0.7,
            filter_cutoff: 2000.0,
            filter_resonance: 1.0,
            attack: 0.01,
            release: 0.5,
            waveform: "sawtooth".to_string(),
            extra: HashMap::new(),
        }
    }
}

/// 单个预设
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    /// 预设名称
    pub name: String,
    /// 预设分类
    pub category: PresetCategory,
    /// 描述
    pub description: String,
    /// 参数
    pub parameters: PresetParameters,
    /// 标签 (用于搜索)
    #[serde(default)]
    pub tags: Vec<String>,
    /// 是否已收藏
    #[serde(default)]
    pub is_favorite: bool,
    /// 使用次数
    #[serde(default)]
    pub usage_count: u32,
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            category: PresetCategory::Basic,
            description: String::new(),
            parameters: PresetParameters::default(),
            tags: Vec::new(),
            is_favorite: false,
            usage_count: 0,
        }
    }
}

/// 预设集合
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetCollection {
    /// 所有预设
    pub presets: Vec<Preset>,
    /// 收藏的预设索引
    pub favorites: HashSet<usize>,
    /// 预设总数
    pub total_count: usize,
}

impl Default for PresetCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl PresetCollection {
    /// 创建新的预设集合
    pub fn new() -> Self {
        Self {
            presets: Vec::new(),
            favorites: HashSet::new(),
            total_count: 0,
        }
    }

    /// 添加预设
    pub fn add_preset(&mut self, preset: Preset) -> usize {
        let index = self.presets.len();
        self.presets.push(preset);
        self.total_count = self.presets.len();
        index
    }

    /// 获取预设 by 索引
    pub fn get_preset(&self, index: usize) -> Option<&Preset> {
        self.presets.get(index)
    }

    /// 获取预设 by 名称
    pub fn get_preset_by_name(&self, name: &str) -> Option<&Preset> {
        self.presets.iter().find(|p| p.name == name)
    }

    /// 获取所有分类的预设
    pub fn get_presets_by_category(&self, category: &PresetCategory) -> Vec<&Preset> {
        self.presets
            .iter()
            .filter(|p| p.category == *category)
            .collect()
    }

    /// 获取所有收藏的预设
    pub fn get_favorites(&self) -> Vec<&Preset> {
        self.favorites
            .iter()
            .filter_map(|&idx| self.presets.get(idx))
            .collect()
    }

    /// 搜索预设
    pub fn search(&self, query: &str) -> Vec<&Preset> {
        let query_lower = query.to_lowercase();
        self.presets
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// 切换收藏状态
    pub fn toggle_favorite(&mut self, index: usize) -> bool {
        if self.favorites.contains(&index) {
            self.favorites.remove(&index);
            false
        } else {
            self.favorites.insert(index);
            true
        }
    }

    /// 增加使用次数
    pub fn increment_usage(&mut self, index: usize) {
        if let Some(preset) = self.presets.get_mut(index) {
            preset.usage_count += 1;
        }
    }

    /// 获取最常用的预设
    pub fn get_most_used(&self, limit: usize) -> Vec<&Preset> {
        let mut indices: Vec<usize> = (0..self.presets.len()).collect();
        indices.sort_by(|&a, &b| {
            self.presets[b]
                .usage_count
                .cmp(&self.presets[a].usage_count)
        });
        indices
            .into_iter()
            .take(limit)
            .filter_map(|idx| self.presets.get(idx))
            .collect()
    }

    /// 获取预设数量
    pub fn count(&self) -> usize {
        self.presets.len()
    }

    /// 清空所有预设
    pub fn clear(&mut self) {
        self.presets.clear();
        self.favorites.clear();
        self.total_count = 0;
    }
}

/// 预设管理器
#[derive(Debug, Clone, PartialEq)]
pub struct PresetManager {
    /// 当前选中的预设索引
    current_preset: Option<usize>,
    /// 预设集合
    collection: PresetCollection,
    /// 预设文件路径
    preset_path: String,
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PresetManager {
    /// 创建新的预设管理器
    pub fn new() -> Self {
        Self {
            current_preset: None,
            collection: PresetCollection::new(),
            preset_path: String::new(),
        }
    }

    /// 设置预设文件路径
    pub fn set_preset_path(&mut self, path: &str) {
        self.preset_path = path.to_string();
    }

    /// 加载预设集合
    pub fn load_presets(&mut self) -> Result<(), String> {
        if self.preset_path.is_empty() {
            return Err("Preset path not set".to_string());
        }

        // TODO: 从JSON文件加载预设
        // 使用 serde_json 读取文件
        Ok(())
    }

    /// 保存预设集合
    pub fn save_presets(&self) -> Result<(), String> {
        if self.preset_path.is_empty() {
            return Err("Preset path not set".to_string());
        }

        // TODO: 保存到JSON文件
        Ok(())
    }

    /// 选择预设
    pub fn select_preset(&mut self, index: usize) -> bool {
        if index < self.collection.count() {
            self.current_preset = Some(index);
            self.collection.increment_usage(index);
            true
        } else {
            false
        }
    }

    /// 获取当前选中的预设
    pub fn get_current_preset(&self) -> Option<&Preset> {
        self.current_preset
            .and_then(|idx| self.collection.get_preset(idx))
    }

    /// 获取当前预设索引
    pub fn get_current_index(&self) -> Option<usize> {
        self.current_preset
    }

    /// 添加新预设
    pub fn add_preset(&mut self, preset: Preset) -> usize {
        let index = self.collection.add_preset(preset);
        self.current_preset = Some(index);
        index
    }

    /// 删除预设
    pub fn delete_preset(&mut self, index: usize) -> Option<Preset> {
        if index < self.collection.count() {
            self.collection.favorites.remove(&index);
            let preset = self.collection.presets.remove(index);
            self.collection.total_count = self.collection.presets.len();

            // 更新收藏索引
            self.collection.favorites = self
                .collection
                .favorites
                .iter()
                .map(|&idx| if idx > index { idx - 1 } else { idx })
                .collect();

            // 如果删除的是当前预设，清除选择
            if self.current_preset == Some(index) {
                self.current_preset = None;
            }

            Some(preset)
        } else {
            None
        }
    }

    /// 获取所有预设
    pub fn all_presets(&self) -> &[Preset] {
        &self.collection.presets
    }

    /// 按分类获取预设
    pub fn presets_by_category(&self, category: &PresetCategory) -> Vec<&Preset> {
        self.collection.get_presets_by_category(category)
    }

    /// 获取收藏
    pub fn favorites(&self) -> Vec<&Preset> {
        self.collection.get_favorites()
    }

    /// 搜索预设
    pub fn search(&self, query: &str) -> Vec<&Preset> {
        self.collection.search(query)
    }

    /// 切换收藏
    pub fn toggle_favorite(&mut self, index: usize) -> bool {
        self.collection.toggle_favorite(index)
    }

    /// 获取预设总数
    pub fn count(&self) -> usize {
        self.collection.count()
    }

    /// 获取分类统计
    pub fn category_stats(&self) -> HashMap<PresetCategory, usize> {
        let mut stats = HashMap::new();
        for preset in &self.collection.presets {
            *stats.entry(preset.category.clone()).or_insert(0) += 1;
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_creation() {
        let preset = Preset {
            name: "Test Preset".to_string(),
            category: PresetCategory::Bass,
            description: "A test preset".to_string(),
            parameters: PresetParameters::default(),
            tags: vec!["test".to_string()],
            is_favorite: false,
            usage_count: 0,
        };

        assert_eq!(preset.name, "Test Preset");
        assert_eq!(preset.category, PresetCategory::Bass);
    }

    #[test]
    fn test_preset_collection() {
        let mut collection = PresetCollection::new();

        let preset1 = Preset::default();
        let preset2 = Preset::default();

        let idx1 = collection.add_preset(preset1);
        let idx2 = collection.add_preset(preset2);

        assert_eq!(collection.count(), 2);
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
    }

    #[test]
    fn test_preset_search() {
        let mut collection = PresetCollection::new();

        let preset1 = Preset {
            name: "Deep Bass".to_string(),
            description: "A deep bass sound".to_string(),
            ..Preset::default()
        };
        let preset2 = Preset {
            name: "Bright Lead".to_string(),
            description: "A bright lead".to_string(),
            ..Preset::default()
        };

        collection.add_preset(preset1);
        collection.add_preset(preset2);

        let results = collection.search("bass");
        assert_eq!(results.len(), 1);

        let results = collection.search("lead");
        assert_eq!(results.len(), 1);

        let results = collection.search("bright");
        assert_eq!(results.len(), 1);

        let results = collection.search("deep");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_preset_favorites() {
        let mut collection = PresetCollection::new();

        collection.add_preset(Preset::default());
        collection.add_preset(Preset::default());

        // 第一次添加收藏，返回 true（之前没有）
        assert!(collection.toggle_favorite(0));
        assert_eq!(collection.get_favorites().len(), 1);

        // 再次添加，应该切换为未收藏，返回 false（之前有）
        assert!(!collection.toggle_favorite(0));
        assert_eq!(collection.get_favorites().len(), 0);

        // 再次添加
        assert!(collection.toggle_favorite(0));
        assert_eq!(collection.get_favorites().len(), 1);
    }

    #[test]
    fn test_preset_manager() {
        let mut manager = PresetManager::new();

        let preset = Preset {
            name: "Test".to_string(),
            ..Preset::default()
        };

        let index = manager.add_preset(preset);
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.get_current_index(), Some(index));
    }
}
