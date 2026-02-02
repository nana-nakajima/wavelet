use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // User ID
    pub email: String,    // User email
    pub exp: i64,         // Expiration timestamp
    pub iat: i64,         // Issued at timestamp
    pub token_type: String,
}

/// JWT configuration
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
    pub algorithm: Algorithm,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            expiration_hours: 24 * 7, // 7 days
            algorithm: Algorithm::HS256,
        }
    }
}

/// JWT Service
#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    pub fn new(config: Option<JwtConfig>) -> Self {
        Self {
            config: config.unwrap_or_default()
        }
    }
    
    /// Generate a new JWT token
    pub fn generate_token(&self, user_id: &Uuid, email: &str) -> TokenResponse {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.expiration_hours);
        
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };
        
        let secret = self.config.secret.as_bytes();
        
        let token = encode(
            &Header::new(self.config.algorithm),
            &claims,
            &EncodingKey::from_secret(secret)
        ).expect("Failed to encode JWT token");
        
        TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.expiration_hours * 3600,
        }
    }
    
    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::Error> {
        let secret = self.config.secret.as_bytes();
        
        let validation = Validation::new(self.config.algorithm);
        
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret),
            &validation
        ).map(|data| data.claims)
    }
    
    /// Extract user ID from token
    pub fn get_user_id(&self, token: &str) -> Result<Uuid, jsonwebtoken::Error> {
        let claims = self.validate_token(token)?;
        Uuid::parse_str(&claims.sub).map_err(|_| jsonwebtoken::Error::InvalidToken)
    }
}

/// Token response structure
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// JWT Error types
#[derive(Debug)]
pub enum JwtError {
    InvalidToken,
    ExpiredToken,
    InvalidClaims,
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::InvalidToken => write!(f, "Invalid token"),
            JwtError::ExpiredToken => write!(f, "Token has expired"),
            JwtError::InvalidClaims => write!(f, "Invalid token claims"),
        }
    }
}

impl std::error::Error for JwtError {}
