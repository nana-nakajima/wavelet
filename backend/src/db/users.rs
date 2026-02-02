use sqlx::postgres::PgPool;
use chrono::NaiveDateTime;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use actix_web::error::InternalError;
use std::fmt;

/// User roles
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum UserRole {
    User,
    Creator,
    Admin,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

/// User model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_login_at: Option<NaiveDateTime>,
    pub is_active: bool,
}

/// User registration request
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 32, message = "Username must be 3-32 characters"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, max = 128, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// User login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// User response (without sensitive data)
#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: String,
    pub presets_count: i64,
    pub followers_count: i64,
    pub following_count: i64,
}

/// Auth token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

/// Database operations for users
pub struct UserRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
    
    /// Create a new user
    pub async fn create(&self, req: &RegisterRequest, password_hash: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, role, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, true)
            RETURNING *
            "#,
            Uuid::new_v4(),
            req.username,
            req.email,
            password_hash,
            req.username,  // display_name defaults to username
            UserRole::User as UserRole
        )
        .fetch_one(self.pool)
        .await?;
        
        Ok(user)
    }
    
    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT * FROM users WHERE email = $1 AND is_active = true"#,
            email
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(user)
    }
    
    /// Find user by ID
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT * FROM users WHERE id = $1 AND is_active = true"#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(user)
    }
    
    /// Update last login time
    pub async fn update_last_login(&self, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1"#,
            id
        )
        .execute(self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Get user profile with stats
    pub async fn get_profile(&self, id: &Uuid) -> Result<Option<UserResponse>, sqlx::Error> {
        let profile = sqlx::query_as!(
            UserResponse,
            r#"
            SELECT 
                u.id::text,
                u.username,
                u.email,
                u.display_name,
                u.bio,
                u.avatar_url,
                u.role::text as role,
                u.created_at::text,
                COALESCE(p.count, 0)::int8 as presets_count,
                COALESCE(f.followers, 0)::int8 as followers_count,
                COALESCE(f2.following, 0)::int8 as following_count
            FROM users u
            LEFT JOIN (
                SELECT user_id, COUNT(*) as count FROM presets WHERE is_public = true GROUP BY user_id
            ) p ON u.id = p.user_id
            LEFT JOIN (
                SELECT following_id, COUNT(*) as followers FROM follows GROUP BY following_id
            ) f ON u.id = f.following_id
            LEFT JOIN (
                SELECT follower_id, COUNT(*) as following FROM follows GROUP BY follower_id
            ) f2 ON u.id = f2.follower_id
            WHERE u.id = $1 AND u.is_active = true
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        
        Ok(profile)
    }
    
    /// Search users by username
    pub async fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<UserResponse>, sqlx::Error> {
        let users = sqlx::query_as!(
            UserResponse,
            r#"
            SELECT 
                u.id::text,
                u.username,
                u.email,
                u.display_name,
                u.bio,
                u.avatar_url,
                u.role::text as role,
                u.created_at::text,
                COALESCE(p.count, 0)::int8 as presets_count,
                0 as followers_count,
                0 as following_count
            FROM users u
            LEFT JOIN (
                SELECT user_id, COUNT(*) as count FROM presets WHERE is_public = true GROUP BY user_id
            ) p ON u.id = p.user_id
            WHERE u.username ILIKE $1 OR u.display_name ILIKE $1
            ORDER BY u.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            format!("%{}%", query),
            limit,
            offset
        )
        .fetch_all(self.pool)
        .await?;
        
        Ok(users)
    }
}
