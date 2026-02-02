// Follow system database operations
// Allows users to follow/unfollow creators

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Follow relationship between users
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Follow {
    pub id: Uuid,
    pub follower_id: Uuid,
    pub following_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
}

/// Public user info for follow lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub preset_count: i64,
    pub follower_count: i64,
}

/// Follow repository for database operations
pub struct FollowRepository {
    pool: sqlx::PgPool,
}

impl FollowRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Follow a user
    pub async fn follow(&self, follower_id: Uuid, following_id: Uuid) -> Result<Follow, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            Follow,
            r#"
            INSERT INTO user_follows (id, follower_id, following_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (follower_id, following_id) DO UPDATE SET id = user_follows.id
            RETURNING id, follower_id, following_id, created_at
            "#,
            id,
            follower_id,
            following_id
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Unfollow a user
    pub async fn unfollow(&self, follower_id: Uuid, following_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_follows
            WHERE follower_id = $1 AND following_id = $2
            "#,
            follower_id,
            following_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if user is following another user
    pub async fn is_following(&self, follower_id: Uuid, following_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_follows
                WHERE follower_id = $1 AND following_id = $2
            ) as "exists!"
            "#,
            follower_id,
            following_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Get list of users that a user is following
    pub async fn get_following(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserSummary>, sqlx::Error> {
        sqlx::query_as!(
            UserSummary,
            r#"
            SELECT 
                u.id,
                u.username,
                u.display_name,
                (SELECT COUNT(*) FROM presets WHERE user_id = u.id) as "preset_count!",
                (SELECT COUNT(*) FROM user_follows WHERE following_id = u.id) as "follower_count!"
            FROM users u
            INNER JOIN user_follows f ON f.following_id = u.id
            WHERE f.follower_id = $1
            ORDER BY f.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get list of users following a user
    pub async fn get_followers(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserSummary>, sqlx::Error> {
        sqlx::query_as!(
            UserSummary,
            r#"
            SELECT 
                u.id,
                u.username,
                u.display_name,
                (SELECT COUNT(*) FROM presets WHERE user_id = u.id) as "preset_count!",
                (SELECT COUNT(*) FROM user_follows WHERE following_id = u.id) as "follower_count!"
            FROM users u
            INNER JOIN user_follows f ON f.follower_id = u.id
            WHERE f.following_id = $1
            ORDER BY f.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get follower count for a user
    pub async fn get_follower_count(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!" FROM user_follows WHERE following_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Get following count for a user
    pub async fn get_following_count(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!" FROM user_follows WHERE follower_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
    }
}
