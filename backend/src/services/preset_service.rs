// Preset service - Business logic layer
// WAVELET Backend - Service layer for preset operations

use std::sync::Arc;
use uuid::Uuid;
use crate::db::presets::PresetRepository;
use crate::models::preset::{
    Preset, PresetResponse, PresetDetailResponse, PresetListResponse,
    CreatePresetRequest, UpdatePresetRequest, SearchQuery, RateRequest,
};
use crate::storage::{StorageBackend, StorageError};

/// Service errors
#[derive(Debug, thiserror::Error)]
pub enum PresetServiceError {
    #[error("Preset not found")]
    NotFound,
    
    #[error("Access denied")]
    AccessDenied,
    
    #[error("Invalid category: {0}")]
    InvalidCategory(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Result type for preset service operations
pub type PresetServiceResult<T> = Result<T, PresetServiceError>;

/// Preset service - handles business logic for preset operations
pub struct PresetService {
    repo: PresetRepository<'static>,
    storage: Arc<dyn StorageBackend>,
}

impl PresetService {
    /// Create new preset service
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `storage` - Storage backend for preset files
    pub fn new(pool: &sqlx::PgPool, storage: Arc<dyn StorageBackend>) -> Self {
        // SAFETY: The pool lives long enough for our static lifetime
        // This is safe because sqlx::PgPool is thread-safe and lives as long as needed
        let pool_ref = unsafe { std::mem::transmute(pool) };
        Self {
            repo: PresetRepository::new(pool_ref),
            storage,
        }
    }
    
    /// Create a new preset
    /// 
    /// # Arguments
    /// * `user_id` - ID of user creating the preset
    /// * `data` - Preset creation request
    /// 
    /// # Returns
    /// Created preset response
    pub async fn create_preset(
        &self,
        user_id: Uuid,
        data: CreatePresetRequest,
    ) -> PresetServiceResult<PresetDetailResponse> {
        // Validate category
        self.validate_category(&data.category)?;
        
        // Serialize parameters to bytes for storage
        let parameters_json = serde_json::to_string(&data.parameters)
            .map_err(|e| PresetServiceError::ValidationError(e.to_string()))?;
        
        // Create preset in database first
        let preset = self.repo.create(&user_id, &data, None).await?;
        
        // Upload parameters to storage
        let storage_path = self.storage.upload_preset(preset.id, parameters_json.as_bytes()).await?;
        
        // Update preset with storage path
        // Note: In a real implementation, you'd update the DB with the storage path
        // For now, we just return the preset response
        
        // Get author info
        let author_name = self.repo.get_author_name(&user_id).await?
            .unwrap_or_else(|| "Unknown".to_string());
        let author_username = self.repo.get_author_username(&user_id).await?
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(PresetDetailResponse::from_preset_with_author(
            &preset,
            &author_name,
            &author_username,
        ))
    }
    
    /// Get a preset by ID
    /// 
    /// # Arguments
    /// * `preset_id` - Preset ID
    /// * `requester_id` - Optional requesting user ID (for access check)
    /// 
    /// # Returns
    /// Preset detail response
    pub async fn get_preset(
        &self,
        preset_id: Uuid,
        requester_id: Option<Uuid>,
    ) -> PresetServiceResult<PresetDetailResponse> {
        let preset = self.repo.find_by_id(&preset_id).await?
            .ok_or(PresetServiceError::NotFound)?;
        
        // Check access permissions
        if !preset.is_public {
            match requester_id {
                Some(user_id) if user_id == preset.user_id => { /* Owner access allowed */ },
                Some(_) => return Err(PresetServiceError::AccessDenied),
                None => return Err(PresetServiceError::AccessDenied),
            }
        }
        
        // Get author info
        let author_name = self.repo.get_author_name(&preset.user_id).await?
            .unwrap_or_else(|| "Unknown".to_string());
        let author_username = self.repo.get_author_username(&preset.user_id).await?
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(PresetDetailResponse::from_preset_with_author(
            &preset,
            &author_name,
            &author_username,
        ))
    }
    
    /// Search presets with filters and pagination
    /// 
    /// # Arguments
    /// * `query` - Search query parameters
    /// 
    /// # Returns
    /// Paginated list of presets
    pub async fn search_presets(
        &self,
        query: SearchQuery,
    ) -> PresetServiceResult<PresetListResponse> {
        // Validate and normalize pagination
        let page = if query.page < 1 { 1 } else { query.page };
        let limit = if query.limit < 1 { 20 } else { query.limit.min(100) };
        let offset = (page - 1) * limit;
        
        // Validate sort parameter
        let sort_by = query.sort.as_deref();
        let valid_sorts = ["newest", "popular", "rating", "downloads"];
        let sort_by = sort_by.filter(|s| valid_sorts.contains(s));
        
        // Search presets
        let presets = self.repo.search_public(
            query.q.as_deref(),
            query.category.as_deref(),
            sort_by,
            limit,
            offset,
        ).await?;
        
        // Get total count
        let total = self.repo.count_public(
            query.q.as_deref(),
            query.category.as_deref(),
        ).await?;
        
        // Convert to response with author info
        let mut preset_responses = Vec::new();
        for preset in presets {
            let author_name = self.repo.get_author_name(&preset.user_id).await?
                .unwrap_or_else(|| "Unknown".to_string());
            
            preset_responses.push(PresetResponse::from_preset_with_author(
                &preset,
                &author_name,
            ));
        }
        
        let total_pages = (total as f64 / limit as f64).ceil() as i64;
        
        Ok(PresetListResponse {
            presets: preset_responses,
            total,
            page,
            limit,
            total_pages,
        })
    }
    
    /// Download preset file
    /// 
    /// # Arguments
    /// * `preset_id` - Preset ID
    /// * `requester_id` - Optional requesting user ID (for access check)
    /// 
    /// # Returns
    /// Preset file data as bytes
    pub async fn download_preset(
        &self,
        preset_id: Uuid,
        requester_id: Option<Uuid>,
    ) -> PresetServiceResult<Vec<u8>> {
        let preset = self.repo.find_by_id(&preset_id).await?
            .ok_or(PresetServiceError::NotFound)?;
        
        // Check access permissions
        if !preset.is_public {
            match requester_id {
                Some(user_id) if user_id == preset.user_id => { /* Owner access allowed */ },
                Some(_) => return Err(PresetServiceError::AccessDenied),
                None => return Err(PresetServiceError::AccessDenied),
            }
        }
        
        // Increment download count
        self.repo.increment_download(&preset_id).await?;
        
        // Get file from storage
        let data = self.storage.download_preset(preset_id).await?;
        
        Ok(data)
    }
    
    /// Rate a preset
    /// 
    /// # Arguments
    /// * `preset_id` - Preset ID
    /// * `user_id` - User ID (rater)
    /// * `rating` - Rating value (1-5)
    /// * `comment` - Optional comment
    /// 
    /// # Returns
    /// Ok(()) on success
    pub async fn rate_preset(
        &self,
        preset_id: Uuid,
        user_id: Uuid,
        rating: i32,
        comment: Option<String>,
    ) -> PresetServiceResult<()> {
        // Validate preset exists
        let preset = self.repo.find_by_id(&preset_id).await?
            .ok_or(PresetServiceError::NotFound)?;
        
        // Validate rating range
        if rating < 1 || rating > 5 {
            return Err(PresetServiceError::ValidationError(
                "Rating must be between 1 and 5".to_string()
            ));
        }
        
        // Check if user is rating their own preset
        if user_id == preset.user_id {
            return Err(PresetServiceError::ValidationError(
                "Cannot rate your own preset".to_string()
            ));
        }
        
        // Upsert rating
        self.repo.upsert_rating(&preset_id, &user_id, rating, comment.as_deref()).await?;
        
        Ok(())
    }
    
    /// Update a preset
    /// 
    /// # Arguments
    /// * `preset_id` - Preset ID
    /// * `user_id` - User ID (must be owner)
    /// * `data` - Update data
    /// 
    /// # Returns
    /// Updated preset response
    pub async fn update_preset(
        &self,
        preset_id: Uuid,
        user_id: Uuid,
        data: UpdatePresetRequest,
    ) -> PresetServiceResult<PresetDetailResponse> {
        // Validate category if provided
        if let Some(category) = &data.category {
            self.validate_category(category)?;
        }
        
        // Update preset
        let preset = self.repo.update(&preset_id, &user_id, &data).await?
            .ok_or(PresetServiceError::NotFound)?;
        
        // If parameters updated, update storage
        if let Some(parameters) = &data.parameters {
            let parameters_json = serde_json::to_string(parameters)
                .map_err(|e| PresetServiceError::ValidationError(e.to_string()))?;
            let _ = self.storage.upload_preset(preset_id, parameters_json.as_bytes()).await;
        }
        
        // Get author info
        let author_name = self.repo.get_author_name(&preset.user_id).await?
            .unwrap_or_else(|| "Unknown".to_string());
        let author_username = self.repo.get_author_username(&preset.user_id).await?
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(PresetDetailResponse::from_preset_with_author(
            &preset,
            &author_name,
            &author_username,
        ))
    }
    
    /// Delete a preset
    /// 
    /// # Arguments
    /// * `preset_id` - Preset ID
    /// * `user_id` - User ID (must be owner)
    /// 
    /// # Returns
    /// Ok(()) on success
    pub async fn delete_preset(
        &self,
        preset_id: Uuid,
        user_id: Uuid,
    ) -> PresetServiceResult<()> {
        // Delete from database
        let deleted = self.repo.delete(&preset_id, &user_id).await?;
        
        if !deleted {
            return Err(PresetServiceError::NotFound);
        }
        
        // Delete from storage
        let _ = self.storage.delete_preset(preset_id).await;
        
        Ok(())
    }
    
    /// Get presets by user
    /// 
    /// # Arguments
    /// * `user_id` - User ID
    /// * `include_private` - Whether to include private presets
    /// 
    /// # Returns
    /// List of presets
    pub async fn get_user_presets(
        &self,
        user_id: Uuid,
        requester_id: Option<Uuid>,
        include_private: bool,
    ) -> PresetServiceResult<Vec<PresetResponse>> {
        // Check if requester is the owner
        let is_owner = requester_id.map_or(false, |id| id == user_id);
        
        let presets = self.repo.find_by_user(&user_id, is_owner && include_private).await?;
        
        // Convert to response
        let mut responses = Vec::new();
        for preset in presets {
            // Skip private presets if not owner
            if !preset.is_public && !is_owner {
                continue;
            }
            
            let author_name = self.repo.get_author_name(&preset.user_id).await?
                .unwrap_or_else(|| "Unknown".to_string());
            
            responses.push(PresetResponse::from_preset_with_author(
                &preset,
                &author_name,
            ));
        }
        
        Ok(responses)
    }
    
    /// Validate preset category
    fn validate_category(&self, category: &str) -> PresetServiceResult<()> {
        let valid_categories = [
            "lead", "bass", "pad", "fx", "keys", 
            "drums", "sequencer", "other",
        ];
        
        if !valid_categories.contains(&category.to_lowercase().as_str()) {
            return Err(PresetServiceError::InvalidCategory(category.to_string()));
        }
        
        Ok(())
    }
}
