// Preset database operations
// WAVELET Backend - Database layer for preset CRUD operations

use sqlx::postgres::PgPool;
use uuid::Uuid;
use thiserror::Error;
use crate::models::preset::{
    Preset, PresetRating,
    CreatePresetRequest, UpdatePresetRequest,
};

/// Preset repository errors
#[derive(Debug, Error)]
pub enum PresetRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Preset not found")]
    NotFound,
}

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
        let preset = sqlx::query_as_unchecked!(
            Preset,
            r#"
            INSERT INTO presets (
                user_id, name, category, description, preset_data, 
                is_public, storage_path, downloads_count, rating, rating_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0.00::float, 0)
            RETURNING id, user_id, name, description, category, tags, preset_data,
                      thumbnail_url, is_public, is_featured, downloads_count, likes_count,
                      rating::float as rating, rating_count, storage_path, created_at, updated_at
            "#,
            user_id,
            data.name,
            data.category,
            data.description,
            data.preset_data,
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
        let preset = sqlx::query_as_unchecked!(
            Preset,
            r#"
            SELECT id, user_id, name, description, category, tags, preset_data,
                   thumbnail_url, is_public, is_featured, downloads_count, likes_count,
                   rating::float as rating, rating_count, storage_path, created_at, updated_at
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
            sqlx::query_as_unchecked!(
                Preset,
                r#"
                SELECT id, user_id, name, COALESCE(description, '') as description, category, 
             tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url, 
             is_public, is_featured, downloads_count, likes_count,
             COALESCE(rating, 0)::float as rating, rating_count, 
             COALESCE(storage_path, '') as storage_path, created_at, updated_at
                FROM presets
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as_unchecked!(
                Preset,
                r#"
                SELECT id, user_id, name, COALESCE(description, '') as description, category, 
             tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url, 
             is_public, is_featured, downloads_count, likes_count,
             COALESCE(rating, 0)::float as rating, rating_count, 
             COALESCE(storage_path, '') as storage_path, created_at, updated_at
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
            Some("popular") | Some("downloads") => "ORDER BY downloads_count DESC",
            Some("rating") => "ORDER BY rating DESC",
            Some("newest") | None => "ORDER BY created_at DESC",
            _ => "ORDER BY created_at DESC",
        };
        
        // Build WHERE clause with parameters
        let mut sql_conditions: Vec<String> = Vec::new();
        let mut sql_params: Vec<Box<dyn sqlx::Encode<'_, sqlx::postgres::Postgres> + Send + Sync>> = Vec::new();
        let mut param_idx = 0;
        
        // Always public
        sql_conditions.push("is_public = true".to_string());
        
        if let Some(q) = query {
            param_idx += 1;
            sql_conditions.push(format!("(name ILIKE ${} OR description ILIKE ${})", param_idx, param_idx));
            sql_params.push(Box::new(format!("%{}%", q)));
        }
        
        if let Some(cat) = category {
            param_idx += 1;
            sql_conditions.push(format!("category = ${}", param_idx));
            sql_params.push(Box::new(cat));
        }
        
        let where_clause = sql_conditions.join(" AND ");
        
        // Build final query - use separate queries based on parameters
        let presets: Vec<Preset> = match (query, category) {
            (None, None) => {
                sqlx::query_as_unchecked!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                           tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                           is_public, is_featured, downloads_count, likes_count,
                           COALESCE(rating, 0)::float as rating, rating_count,
                           COALESCE(storage_path, '') as storage_path, created_at, updated_at
                    FROM presets
                    WHERE is_public = true
                    ORDER BY created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    limit,
                    offset
                )
                .fetch_all(self.pool)
                .await?
            }
            (Some(q), None) => {
                sqlx::query_as_unchecked!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                           tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                           is_public, is_featured, downloads_count, likes_count,
                           COALESCE(rating, 0)::float as rating, rating_count,
                           COALESCE(storage_path, '') as storage_path, created_at, updated_at
                    FROM presets
                    WHERE is_public = true AND (name ILIKE $1 OR description ILIKE $1)
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    format!("%{}%", q),
                    limit,
                    offset
                )
                .fetch_all(self.pool)
                .await?
            }
            (None, Some(cat)) => {
                sqlx::query_as_unchecked!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                           tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                           is_public, is_featured, downloads_count, likes_count,
                           COALESCE(rating, 0)::float as rating, rating_count,
                           COALESCE(storage_path, '') as storage_path, created_at, updated_at
                    FROM presets
                    WHERE is_public = true AND category = $1
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    cat,
                    limit,
                    offset
                )
                .fetch_all(self.pool)
                .await?
            }
            (Some(q), Some(cat)) => {
                sqlx::query_as_unchecked!(
                    Preset,
                    r#"
                    SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                           tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                           is_public, is_featured, downloads_count, likes_count,
                           COALESCE(rating, 0)::float as rating, rating_count,
                           COALESCE(storage_path, '') as storage_path, created_at, updated_at
                    FROM presets
                    WHERE is_public = true AND (name ILIKE $1 OR description ILIKE $1) AND category = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    format!("%{}%", q),
                    cat,
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
                let row = sqlx::query!(
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
                .await?;
                row.count.unwrap_or(0)
            }
            (Some(q), None) => {
                let row = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM presets
                    WHERE is_public = true
                    AND (name ILIKE $1 OR description ILIKE $1)
                    "#,
                    format!("%{}%", q)
                )
                .fetch_one(self.pool)
                .await?;
                row.count.unwrap_or(0)
            }
            (None, Some(cat)) => {
                let row = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM presets
                    WHERE is_public = true
                    AND category = $1
                    "#,
                    cat
                )
                .fetch_one(self.pool)
                .await?;
                row.count.unwrap_or(0)
            }
            (None, None) => {
                let row = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM presets
                    WHERE is_public = true
                    "#,
                )
                .fetch_one(self.pool)
                .await?;
                row.count.unwrap_or(0)
            }
        };
        
        Ok(count)
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
            SET downloads_count = downloads_count + 1
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
        
        if let Some(preset_data) = &data.preset_data {
            param_count += 1;
            set_clauses.push(format!("preset_data = ${}", param_count));
            params.push(Box::new(preset_data) as Box<dyn sqlx::Encode<'_, _>>);
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
        
        let presets = sqlx::query_as_unchecked!(
            Preset,
            r#"
            SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                   tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                   is_public, is_featured, downloads_count, likes_count,
                   COALESCE(rating, 0)::float as rating, rating_count,
                   COALESCE(storage_path, '') as storage_path, created_at, updated_at
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
        let presets = sqlx::query_as_unchecked!(
            Preset,
            r#"
            SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                   tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                   is_public, is_featured, downloads_count, likes_count,
                   COALESCE(rating, 0)::float as rating, rating_count,
                   COALESCE(storage_path, '') as storage_path, created_at, updated_at
            FROM presets
            WHERE is_public = true AND is_featured = true
            ORDER BY rating DESC, downloads_count DESC
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
        let rating_row = sqlx::query_as_unchecked!(
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
        let rating = sqlx::query_as_unchecked!(
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
            SELECT 1 as exists FROM preset_ratings
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
    
    /// Get feed presets based on feed type
    /// 
    /// # Arguments
    /// * `feed_type` - Type of feed: "latest", "popular", "featured", "following"
    /// * `category` - Optional category filter
    /// * `user_id` - For "following" feed, get presets from followed users
    /// * `limit` - Maximum number of results
    /// * `offset` - Offset for pagination
    /// 
    /// # Returns
    /// Vector of Presets for the feed
    pub async fn get_feed(
        &self,
        feed_type: &str,
        category: Option<&str>,
        user_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Preset>, sqlx::Error> {
        match feed_type {
            "popular" => {
                // Popular presets: most downloads
                match category {
                    Some(cat) => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                               tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                               is_public, is_featured, downloads_count, likes_count,
                               COALESCE(rating, 0)::float as rating, rating_count,
                               COALESCE(storage_path, '') as storage_path, created_at, updated_at
                        FROM presets
                        WHERE is_public = true AND category = $1
                        ORDER BY downloads_count DESC, rating DESC
                        LIMIT $2 OFFSET $3
                        "#,
                        cat, limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                    None => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                               tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                               is_public, is_featured, downloads_count, likes_count,
                               COALESCE(rating, 0)::float as rating, rating_count,
                               COALESCE(storage_path, '') as storage_path, created_at, updated_at
                        FROM presets
                        WHERE is_public = true
                        ORDER BY downloads_count DESC, rating DESC
                        LIMIT $1 OFFSET $2
                        "#,
                        limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                }
            }
            "featured" => {
                // Featured presets (curated by admin)
                match category {
                    Some(cat) => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                               tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                               is_public, is_featured, downloads_count, likes_count,
                               COALESCE(rating, 0)::float as rating, rating_count,
                               COALESCE(storage_path, '') as storage_path, created_at, updated_at
                        FROM presets
                        WHERE is_public = true AND is_featured = true AND category = $1
                        ORDER BY rating DESC, downloads_count DESC
                        LIMIT $2 OFFSET $3
                        "#,
                        cat, limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                    None => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                               tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                               is_public, is_featured, downloads_count, likes_count,
                               COALESCE(rating, 0)::float as rating, rating_count,
                               COALESCE(storage_path, '') as storage_path, created_at, updated_at
                        FROM presets
                        WHERE is_public = true AND is_featured = true
                        ORDER BY rating DESC, downloads_count DESC
                        LIMIT $1 OFFSET $2
                        "#,
                        limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                }
            }
            "following" if user_id.is_some() => {
                // Following feed: presets from users this person follows
                let user_id = user_id.unwrap();
                match category {
                    Some(cat) => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT p.id, p.user_id, p.name, COALESCE(p.description, '') as description, p.category, 
                               p.tags, p.preset_data, COALESCE(p.thumbnail_url, '') as thumbnail_url,
                               p.is_public, p.is_featured, p.downloads_count, p.likes_count,
                               COALESCE(p.rating, 0)::float as rating, p.rating_count,
                               COALESCE(p.storage_path, '') as storage_path, p.created_at, p.updated_at
                        FROM presets p
                        INNER JOIN user_follows f ON p.user_id = f.following_id
                        WHERE p.is_public = true AND f.follower_id = $1 AND p.category = $2
                        ORDER BY p.created_at DESC
                        LIMIT $3 OFFSET $4
                        "#,
                        user_id, cat, limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                    None => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT p.id, p.user_id, p.name, COALESCE(p.description, '') as description, p.category, 
                               p.tags, p.preset_data, COALESCE(p.thumbnail_url, '') as thumbnail_url,
                               p.is_public, p.is_featured, p.downloads_count, p.likes_count,
                               COALESCE(p.rating, 0)::float as rating, p.rating_count,
                               COALESCE(p.storage_path, '') as storage_path, p.created_at, p.updated_at
                        FROM presets p
                        INNER JOIN user_follows f ON p.user_id = f.following_id
                        WHERE p.is_public = true AND f.follower_id = $1
                        ORDER BY p.created_at DESC
                        LIMIT $2 OFFSET $3
                        "#,
                        user_id, limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                }
            }
            _ => {
                // Default: latest presets (newest first)
                match category {
                    Some(cat) => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                               tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                               is_public, is_featured, downloads_count, likes_count,
                               COALESCE(rating, 0)::float as rating, rating_count,
                               COALESCE(storage_path, '') as storage_path, created_at, updated_at
                        FROM presets
                        WHERE is_public = true AND category = $1
                        ORDER BY created_at DESC
                        LIMIT $2 OFFSET $3
                        "#,
                        cat, limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                    None => sqlx::query_as_unchecked!(
                        Preset,
                        r#"
                        SELECT id, user_id, name, COALESCE(description, '') as description, category, 
                               tags, preset_data, COALESCE(thumbnail_url, '') as thumbnail_url,
                               is_public, is_featured, downloads_count, likes_count,
                               COALESCE(rating, 0)::float as rating, rating_count,
                               COALESCE(storage_path, '') as storage_path, created_at, updated_at
                        FROM presets
                        WHERE is_public = true
                        ORDER BY created_at DESC
                        LIMIT $1 OFFSET $2
                        "#,
                        limit, offset
                    )
                    .fetch_all(self.pool)
                    .await,
                }
            }
        }
    }
    
    /// Count presets in feed
    pub async fn count_feed(
        &self,
        feed_type: &str,
        category: Option<&str>,
        user_id: Option<Uuid>,
    ) -> Result<i64, sqlx::Error> {
        match feed_type {
            "following" if user_id.is_some() => {
                let user_id = user_id.unwrap();
                match category {
                    Some(cat) => {
                        let row = sqlx::query!(
                            r#"
                            SELECT COUNT(*) as count
                            FROM presets p
                            INNER JOIN user_follows f ON p.user_id = f.following_id
                            WHERE p.is_public = true AND f.follower_id = $1 AND p.category = $2
                            "#,
                            user_id, cat
                        )
                        .fetch_one(self.pool)
                        .await?;
                        Ok(row.count.unwrap_or(0))
                    }
                    None => {
                        let row = sqlx::query!(
                            r#"
                            SELECT COUNT(*) as count
                            FROM presets p
                            INNER JOIN user_follows f ON p.user_id = f.following_id
                            WHERE p.is_public = true AND f.follower_id = $1
                            "#,
                            user_id
                        )
                        .fetch_one(self.pool)
                        .await?;
                        Ok(row.count.unwrap_or(0))
                    }
                }
            }
            _ => {
                // For other feed types, use the same count logic
                self.count_public(None, category).await
            }
        }
    }
}
