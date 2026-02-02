// Preset database operations
// WAVELET Backend - Database layer for preset CRUD operations

use sqlx::postgres::PgPool;
use uuid::Uuid;
use crate::models::preset::{
    Preset, PresetRating, PresetCategory,
    CreatePresetRequest, UpdatePresetRequest,
};

/// Repository for preset database operations
pub struct PresetRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> PresetRepository<'a> {
    /// Create new repository instance
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
    
    /// Create a new preset in the database
    /// 
    /// # Arguments
    /// * `user_id` - UUID of the preset owner
    /// * `data` - Preset creation request data
    /// * `storage_path` - Optional path to preset file in storage
    /// 
    /// # Returns
    /// Created Preset or error
    pub async fn create(
        &self,
        user_id: &Uuid,
        data: &CreatePresetRequest,
        storage_path: Option<&str>,
    ) -> Result<Preset, sqlx::Error> {
        let preset = sqlx::query_as!(
            Preset,
            r#"
            INSERT INTO presets (
                user_id, name, category, description, parameters, 
                is_public, storage_path, download_count, rating, rating_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0.00, 0)
            RETURNING *
            "#,
            user_id,
            data.name,
            data.category,
            data.description,
            data.parameters,
            data.is_public,
            storage_path
        )
        .fetch_one(self.pool)
        .await?;
        
        Ok(preset)
    }
    
    /// Find preset by ID
    /// 
    /// # Arguments
    /// * `id` - Preset UUID
    /// 
    /// # Returns
    /// Optional Preset (None if not found)
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<Preset>, sqlx::Error> {
        let preset = sqlx::query_as!(
            Preset,
            r#"
            SELECT id, user_id, name, category, description, parameters,
                   is_public, download_count, rating, rating_count,
                   created_at, updated_at
            FROM presets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(preset)
    }
    
    /// Find all presets by user ID
    /// 
    /// # Arguments
    /// * `user_id` - User UUID
    /// * `include_private` - Whether to include private presets
    /// 
    /// # Returns
    /// Vector of Presets
    pub async fn find_by_user(
        &self,
        user_id: &Uuid,
        include_private: bool,
    ) -> Result<Vec<Preset>, sqlx::Error> {
        let presets = if include_private {
            sqlx::query_as!(
                Preset,
                r#"
                SELECT id, user_id, name, category, description, parameters,
                       is_public, download_count, rating, rating_count,
                       created_at, updated_at
                FROM presets
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Preset,
                r#"
                SELECT id, user_id, name, category, description, parameters,
                       is_public, download_count, rating, rating_count,
                       created_at, updated_at
                FROM presets
                WHERE user_id = $1 AND is_public = true
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(self.pool)
            .await?
        };
        
        Ok(presets)
    }
    
    /// Search public presets with filters
    /// 
    /// # Arguments
    /// * `query` - Search text for name/description
    /// * `category` - Optional category filter
    /// * `sort_by` - Sort field: "newest", "popular", "rating", "downloads"
    /// * `limit` - Maximum number of results
    /// * `offset` - Offset for pagination
    /// 
    /// # Returns
    /// Vector of matching Presets
    pub async fn search_public(
        &self,
        query: Option<&str>,
        category: Option<&str>,
        sort_by: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Preset>, sqlx::Error> {
        // Build dynamic query based on filters
        let sort_clause = match sort_by {
            Some("popular") | Some("downloads") => "ORDER BY download_count DESC",
            Some("rating") => "ORDER BY rating DESC",
            Some("newest") | None => "ORDER BY created_at DESC",
            _ => "ORDER BY created_at DESC",
        };
        
        // Execute query based on available filters
        let presets = match (query, category) {
            (Some(q), Some(cat)) => {
                sqlx::query_as!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, category, description, parameters,
                           is_public, download_count, rating, rating_count,
                           created_at, updated_at
                    FROM presets
                    WHERE is_public = true
                    AND (name ILIKE $1 OR description ILIKE $1)
                    AND category = $2
                    {}
                    LIMIT $3 OFFSET $4
                    "#,
                    format!("%{}%", q),
                    cat,
                    sort_clause,
                    limit,
                    offset
                )
                .fetch_all(self.pool)
                .await?
            }
            (Some(q), None) => {
                sqlx::query_as!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, category, description, parameters,
                           is_public, download_count, rating, rating_count,
                           created_at, updated_at
                    FROM presets
                    WHERE is_public = true
                    AND (name ILIKE $1 OR description ILIKE $1)
                    {}
                    LIMIT $2 OFFSET $3
                    "#,
                    sort_clause,
                    format!("%{}%", q),
                    limit,
                    offset
                )
                .fetch_all(self.pool)
                .await?
            }
            (None, Some(cat)) => {
                sqlx::query_as!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, category, description, parameters,
                           is_public, download_count, rating, rating_count,
                           created_at, updated_at
                    FROM presets
                    WHERE is_public = true
                    AND category = $1
                    {}
                    LIMIT $2 OFFSET $3
                    "#,
                    sort_clause,
                    cat,
                    limit,
                    offset
                )
                .fetch_all(self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, category, description, parameters,
                           is_public, download_count, rating, rating_count,
                           created_at, updated_at
                    FROM presets
                    WHERE is_public = true
                    {}
                    LIMIT $1 OFFSET $2
                    "#,
                    sort_clause,
                    limit,
                    offset
                )
                .fetch_all(self.pool)
                .await?
            }
        };
        
        Ok(presets)
    }
    
    /// Get total count of public presets matching filters
    pub async fn count_public(
        &self,
        query: Option<&str>,
        category: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let count = match (query, category) {
            (Some(q), Some(cat)) => {
                sqlx::query!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM presets
                    WHERE is_public = true
                    AND (name ILIKE $1 OR description ILIKE $1)
                    AND category = $2
                    "#,
                    format!("%{}%", q),
                    cat
                )
                .fetch_one(self.pool)
                await?
            }
            (Some(q), None) => {
                sqlx::query!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM presets
                    WHERE is_public = true
                    AND (name ILIKE $1 OR description ILIKE $1)
                    "#,
                    format!("%{}%", q)
                )
                .fetch_one(self.pool)
                .await?
            }
            (None, Some(cat)) => {
                sqlx::query!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM presets
                    WHERE is_public = true
                    AND category = $1
                    "#,
                    cat
                )
                .fetch_one(self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM presets
                    WHERE is_public = true
                    "#,
                )
                .fetch_one(self.pool)
                .await?
            }
        };
        
        Ok(count.count.unwrap_or(0))
    }
    
    /// Increment download count for a preset
    /// 
    /// # Arguments
    /// * `id` - Preset UUID
    /// 
    /// # Returns
    /// Ok(()) on success
    pub async fn increment_download(&self, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE presets
            SET download_count = download_count + 1
            WHERE id = $1
            "#,
            id
        )
        .execute(self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Update rating for a preset (recalculates average)
    /// 
    /// # Arguments
    /// * `id` - Preset UUID
    /// 
    /// # Returns
    /// Ok(()) on success
    pub async fn update_rating(&self, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE presets p
            SET 
                rating = (
                    SELECT AVG(r.rating)::decimal(3,2)
                    FROM preset_ratings r
                    WHERE r.preset_id = p.id
                ),
                rating_count = (
                    SELECT COUNT(*)
                    FROM preset_ratings r
                    WHERE r.preset_id = p.id
                )
            WHERE p.id = $1
            "#,
            id
        )
        .execute(self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Update preset data
    /// 
    /// # Arguments
    /// * `id` - Preset UUID
    /// * `user_id` - User UUID (for ownership check)
    /// * `data` - Update request data
    /// 
    /// # Returns
    /// Updated Preset or error
    pub async fn update(
        &self,
        id: &Uuid,
        user_id: &Uuid,
        data: &UpdatePresetRequest,
    ) -> Result<Option<Preset>, sqlx::Error> {
        // Build dynamic update query
        let mut set_clauses = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::postgres::Postgres>>> = Vec::new();
        let mut param_count = 0;
        
        if let Some(name) = &data.name {
            param_count += 1;
            set_clauses.push(format!("name = ${}", param_count));
            params.push(Box::new(name) as Box<dyn sqlx::Encode<'_, _>>);
        }
        
        if let Some(category) = &data.category {
            param_count += 1;
            set_clauses.push(format!("category = ${}", param_count));
            params.push(Box::new(category) as Box<dyn sqlx::Encode<'_, _>>);
        }
        
        if let Some(description) = &data.description {
            param_count += 1;
            set_clauses.push(format!("description = ${}", param_count));
            params.push(Box::new(description) as Box<dyn sqlx::Encode<'_, _>>);
        }
        
        if let Some(parameters) = &data.parameters {
            param_count += 1;
            set_clauses.push(format!("parameters = ${}", param_count));
            params.push(Box::new(parameters) as Box<dyn sqlx::Encode<'_, _>>);
        }
        
        if let Some(is_public) = &data.is_public {
            param_count += 1;
            set_clauses.push(format!("is_public = ${}", param_count));
            params.push(Box::new(is_public) as Box<dyn sqlx::Encode<'_, _>>);
        }
        
        if set_clauses.is_empty() {
            // No fields to update
            return self.find_by_id(id).await;
        }
        
        // Add ID and user_id params
        param_count += 1;
        params.push(Box::new(id) as Box<dyn sqlx::Encode<'_, _>>);
        param_count += 1;
        params.push(Box::new(user_id) as Box<dyn sqlx::Encode<'_, _>>);
        
        let sql = format!(
            r#"
            UPDATE presets
            SET {}
            WHERE id = ${} AND user_id = ${}
            RETURNING *
            "#,
            set_clauses.join(", "),
            param_count - 1,
            param_count
        );
        
        let preset = sqlx::query_as::<_, Preset>(&sql)
            .fetch_optional(self.pool)
            .await?;
        
        Ok(preset)
    }
    
    /// Delete a preset (only owner can delete)
    /// 
    /// # Arguments
    /// * `id` - Preset UUID
    /// * `user_id` - User UUID (for ownership check)
    /// 
    /// # Returns
    /// true if deleted, false if not found
    pub async fn delete(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM presets
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .execute(self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    /// Get presets by IDs (for bulk fetch)
    pub async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Preset>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        
        let presets = sqlx::query_as!(
            Preset,
            r#"
            SELECT id, user_id, name, category, description, parameters,
                   is_public, download_count, rating, rating_count,
                   created_at, updated_at
            FROM presets
            WHERE id = ANY($1)
            "#,
            ids
        )
        .fetch_all(self.pool)
        .await?;
        
        Ok(presets)
    }
    
    /// Get featured presets
    pub async fn get_featured(&self, limit: i64) -> Result<Vec<Preset>, sqlx::Error> {
        let presets = sqlx::query_as!(
            Preset,
            r#"
            SELECT id, user_id, name, category, description, parameters,
                   is_public, download_count, rating, rating_count,
                   created_at, updated_at
            FROM presets
            WHERE is_public = true AND is_featured = true
            ORDER BY rating DESC, download_count DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(self.pool)
        .await?;
        
        Ok(presets)
    }
}

// ================== Ratings Operations ==================

impl<'a> PresetRepository<'a> {
    /// Add or update a rating for a preset
    /// 
    /// # Arguments
    /// * `preset_id` - Preset UUID
    /// * `user_id` - User UUID
    /// * `rating` - Rating value (1-5)
    /// * `comment` - Optional comment
    /// 
    /// # Returns
    /// Created/Updated PresetRating
    pub async fn upsert_rating(
        &self,
        preset_id: &Uuid,
        user_id: &Uuid,
        rating: i32,
        comment: Option<&str>,
    ) -> Result<PresetRating, sqlx::Error> {
        let rating_row = sqlx::query_as!(
            PresetRating,
            r#"
            INSERT INTO preset_ratings (preset_id, user_id, rating, comment)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (preset_id, user_id)
            DO UPDATE SET rating = EXCLUDED.rating, comment = COALESCE(EXCLUDED.comment, preset_ratings.comment)
            RETURNING *
            "#,
            preset_id,
            user_id,
            rating,
            comment
        )
        .fetch_one(self.pool)
        .await?;
        
        // Update preset average rating
        self.update_rating(preset_id).await?;
        
        Ok(rating_row)
    }
    
    /// Get user's rating for a preset
    /// 
    /// # Arguments
    /// * `preset_id` - Preset UUID
    /// * `user_id` - User UUID
    /// 
    /// # Returns
    /// Optional PresetRating
    pub async fn get_user_rating(
        &self,
        preset_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<PresetRating>, sqlx::Error> {
        let rating = sqlx::query_as!(
            PresetRating,
            r#"
            SELECT id, preset_id, user_id, rating, comment, created_at
            FROM preset_ratings
            WHERE preset_id = $1 AND user_id = $2
            "#,
            preset_id,
            user_id
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(rating)
    }
    
    /// Get average rating for a preset
    pub async fn get_average_rating(&self, preset_id: &Uuid) -> Result<Option<(f32, i32)>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT AVG(rating)::float as avg, COUNT(*) as count
            FROM preset_ratings
            WHERE preset_id = $1
            "#,
            preset_id
        )
        .fetch_one(self.pool)
        .await?;
        
        Ok(match (result.avg, result.count) {
            (Some(avg), Some(count)) => Some((avg as f32, count as i32)),
            _ => None,
        })
    }
    
    /// Check if user has rated a preset
    pub async fn has_user_rated(
        &self,
        preset_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT 1 FROM preset_ratings
            WHERE preset_id = $1 AND user_id = $2
            "#,
            preset_id,
            user_id
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(result.is_some())
    }
    
    /// Delete a rating
    pub async fn delete_rating(
        &self,
        preset_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM preset_ratings
            WHERE preset_id = $1 AND user_id = $2
            "#,
            preset_id,
            user_id
        )
        .execute(self.pool)
        .await?;
        
        // Update preset average rating
        self.update_rating(preset_id).await?;
        
        Ok(result.rows_affected() > 0)
    }
}

// ================== Author Information ==================

impl<'a> PresetRepository<'a> {
    /// Get author name for a user ID
    pub async fn get_author_name(&self, user_id: &Uuid) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT display_name FROM users WHERE id = $1 AND is_active = true
            "#,
            user_id
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(result.and_then(|r| r.display_name))
    }
    
    /// Get author username for a user ID
    pub async fn get_author_username(&self, user_id: &Uuid) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT username FROM users WHERE id = $1 AND is_active = true
            "#,
            user_id
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(result.map(|r| r.username))
    }
}
