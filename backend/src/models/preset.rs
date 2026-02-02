// Preset model definition
// WAVELET Backend - Preset management API

use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::types::BigDecimal;
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
#[derive(Debug, Clone)]
pub struct Preset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub preset_data: serde_json::Value,
    pub thumbnail_url: Option<String>,
    pub is_public: bool,
    pub is_featured: bool,
    pub downloads_count: i32,
    pub likes_count: i32,
    pub rating: f64,  // Calculated from BigDecimal when needed
    pub rating_count: i32,
    pub storage_path: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl sqlx::FromRow<'_, PgRow> for Preset {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Preset {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            category: row.try_get("category")?,
            tags: row.try_get("tags")?,
            preset_data: row.try_get("preset_data")?,
            thumbnail_url: row.try_get("thumbnail_url")?,
            is_public: row.try_get("is_public")?,
            is_featured: row.try_get("is_featured")?,
            downloads_count: row.try_get("downloads_count")?,
            likes_count: row.try_get("likes_count")?,
            rating: row.try_get::<BigDecimal, _>("rating")?.to_string().parse().unwrap_or(0.0),
            rating_count: row.try_get("rating_count")?,
            storage_path: row.try_get("storage_path")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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
    
    /// JSON object containing all preset parameters
    pub preset_data: Option<serde_json::Value>,
    
    /// Whether preset is public or private
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
    pub downloads_count: i32,
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
    pub downloads_count: i32,
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

/// Feed query parameters
#[derive(Debug, Deserialize, Default)]
pub struct FeedQuery {
    /// Feed type: "latest", "popular", "featured", "following"
    #[serde(default = "default_feed_type")]
    pub feed_type: String,
    
    /// Filter by category
    pub category: Option<String>,
    
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: i64,
    
    /// Items per page
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_feed_type() -> String {
    "latest".to_string()
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
            downloads_count: preset.downloads_count,
            rating: preset.rating as f32,
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
            preset_data: preset.preset_data.clone(),
            author_id: preset.user_id,
            author_name: author_name.to_string(),
            author_username: author_username.to_string(),
            downloads_count: preset.downloads_count,
            rating: preset.rating as f32,
            rating_count: preset.rating_count,
            is_public: preset.is_public,
            created_at: preset.created_at,
            updated_at: preset.updated_at,
        }
    }
}
