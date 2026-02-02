use crate::db::users::{UserRepository, RegisterRequest, LoginRequest, UserResponse};
use crate::middleware::jwt::JwtService;
use bcrypt::{hash, verify, DEFAULT_COST};
use actix_web::{web, HttpResponse, Result};
use validator::Validate;

/// User service - business logic
pub struct UserService<'a> {
    repo: UserRepository<'a>,
    jwt: JwtService,
}

impl<'a> UserService<'a> {
    pub fn new(repo: UserRepository<'a>, jwt: JwtService) -> Self {
        Self { repo, jwt }
    }
    
    /// Register a new user
    pub async fn register(&self, req: RegisterRequest) -> Result<HttpResponse> {
        // Validate input
        req.validate().map_err(|e| {
            actix_web::error::ErrorBadRequest(e.to_string())
        })?;
        
        // Check if email already exists
        if let Some(_) = self.repo.find_by_email(&req.email).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(e.to_string())
        })? {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Email already registered"
            })));
        }
        
        // Hash password
        let password_hash = hash(&req.password, DEFAULT_COST)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
        // Create user
        let user = self.repo.create(&req, &password_hash).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
        // Generate token
        let token = self.jwt.generate_token(&user.id, &user.email);
        let user_response = self.user_to_response(&user).await;
        
        Ok(HttpResponse::Created().json(serde_json::json!({
            "access_token": token.access_token,
            "token_type": "Bearer",
            "expires_in": token.expires_in,
            "user": user_response
        })))
    }
    
    /// Login user
    pub async fn login(&self, req: LoginRequest) -> Result<HttpResponse> {
        // Find user by email
        let user = match self.repo.find_by_email(&req.email).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))? 
        {
            Some(u) => u,
            None => {
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid email or password"
                })));
            }
        };
        
        // Verify password
        if !verify(&req.password, &user.password_hash)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))? 
        {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid email or password"
            })));
        }
        
        // Update last login
        self.repo.update_last_login(&user.id).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
        // Generate token
        let token = self.jwt.generate_token(&user.id, &user.email);
        let user_response = self.user_to_response(&user).await;
        
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "access_token": token.access_token,
            "token_type": "Bearer",
            "expires_in": token.expires_in,
            "user": user_response
        })))
    }
    
    /// Get user profile
    pub async fn get_profile(&self, user_id: &uuid::Uuid) -> Result<HttpResponse> {
        let profile = self.repo.get_profile(user_id).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        
        match profile {
            Some(p) => Ok(HttpResponse::Ok().json(p)),
            None => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            })))
        }
    }
    
    /// Convert User model to UserResponse
    async fn user_to_response(&self, user: &crate::db::users::User) -> UserResponse {
        // Get user stats
        let profile = self.repo.get_profile(&user.id).await.ok().flatten();
        
        profile.unwrap_or(UserResponse {
            id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            bio: user.bio.clone(),
            avatar_url: user.avatar_url.clone(),
            role: format!("{:?}", user.role),
            created_at: user.created_at.to_string(),
            presets_count: 0,
            followers_count: 0,
            following_count: 0,
        })
    }
}

///
pub async fn register(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let service = UserService::new(
        UserRepository::new(&data.db),
        data.jwt.clone()
    );
    service.register(req.into_inner()).await
}

pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let service = UserService::new(
        UserRepository::new(&data.db),
        data.jwt.clone()
    );
    service.login(req.into_inner()).await
}

pub async fn get_profile(
    data: web::Data<AppState>,
    user_id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse> {
    let service = UserService::new(
        UserRepository::new(&data.db),
        data.jwt.clone()
    );
    service.get_profile(&user_id.into_inner()).await
}

// AppState struct for dependency injection
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt: JwtService,
}
