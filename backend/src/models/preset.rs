// Preset model definition
// WAVELET Backend - Preset management API

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::Row;

/// Preset category types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresetCategory {
    Lead,
    Bass,
    Pad,
    Fx,
    Keys,
    Drums,
    Sequencer,
    Other,
}

impl Default for PresetCategory {
    fn default() -> Self {
        PresetCategory::Other
    }
}

/// Main Preset model - represents a synth preset in the database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Preset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub preset_data: serde_json::Value,  // JSONB parameters
    pub is_public: bool,
    pub download_count: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Preset with author information (for API responses)
#[derive(Debug, Clone)]
pub struct PresetWithAuthor {
    pub preset: Preset,
    pub author_name: String,
    pub author_username: String,
}

/// Create preset request from API
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePresetRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: String,
    
    #[validate(length(min = 1, max = 50, message = "Category must be 1-50 characters"))]
    pub category: String,
    
    pub description: Option<String>,
    
    /// JSON object containing all preset parameters
    #[validate]
    pub preset_data: serde_json::Value,
    
    /// Whether preset is public or private
    pub is_public: bool,
}

/// Update preset request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePresetRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: Option<String>,
    
    #[validate(length(min = 1, max = 50, message = "Category must be 1-50 characters"))]
    pub category: Option<String>,
    
    pub description: Option<String>,
    
    #[validate]
    pub preset_data: Option<serde_json::Value>,
    
    pub is_public: Option<bool>,
}

/// Response structure for preset list endpoints
#[derive(Debug, Serialize)]
pub struct PresetResponse {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub author_id: Uuid,
    pub author_name: String,
    pub download_count: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for single preset detail
#[derive(Debug, Serialize)]
pub struct PresetDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub preset_data: serde_json::Value,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_username: String,
    pub download_count: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Search query parameters
#[derive(Debug, Deserialize, Default)]
pub struct SearchQuery {
    /// Search text for name/description
    pub q: Option<String>,
    
    /// Filter by category
    pub category: Option<String>,
    
    /// Sort order: "newest", "popular", "rating", "downloads"
    pub sort: Option<String>,
    
    /// Page number for pagination (1-indexed)
    #[serde(default = "default_page")]
    pub page: i64,
    
    /// Items per page
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

/// Paginated list response
#[derive(Debug, Serialize)]
pub struct PresetListResponse {
    pub presets: Vec<PresetResponse>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

/// Rating submission request
#[derive(Debug, Deserialize, Validate)]
pub struct RateRequest {
    #[validate(range(min = 1, max = 5, message = "Rating must be between 1 and 5"))]
    pub rating: i32,
    
    /// Optional comment for the rating
    pub comment: Option<String>,
}

/// User rating with details
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PresetRating {
    pub id: Uuid,
    pub preset_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PresetResponse {
    /// Convert from Preset with author info
    pub fn from_preset_with_author(preset: &Preset, author_name: &str) -> Self {
        Self {
            id: preset.id,
            name: preset.name.clone(),
            category: preset.category.clone(),
            description: preset.description.clone(),
            author_id: preset.user_id,
            author_name: author_name.to_string(),
            download_count: preset.download_count,
            rating: preset.rating,
            rating_count: preset.rating_count,
            is_public: preset.is_public,
            created_at: preset.created_at,
            updated_at: preset.updated_at,
        }
    }
}

impl PresetDetailResponse {
    /// Convert from Preset with author info
    pub fn from_preset_with_author(preset: &Preset, author_name: &str, author_username: &str) -> Self {
        Self {
            id: preset.id,
            name: preset.name.clone(),
            category: preset.category.clone(),
            description: preset.description.clone(),
            parameters: preset.preset_data.clone(),
            author_id: preset.user_id,
            author_name: author_name.to_string(),
            author_username: author_username.to_string(),
            download_count: preset.download_count,
            rating: preset.rating,
            rating_count: preset.rating_count,
            is_public: preset.is_public,
            created_at: preset.created_at,
            updated_at: preset.updated_at,
        }
    }
}
