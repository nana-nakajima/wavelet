mod db;
mod middleware;
mod services;

use actix_web::{web, App, HttpServer, Responder};
use middleware::jwt::JwtService;
use services::user_service::{AppState, register, login, get_profile};
use db::users::UserRepository;
use std::sync::Arc;

/// Health check endpoint
async fn health() -> impl Responder {
    web::Json(serde_json::json!({
        "status": "healthy",
        "service": "wavelet-backend",
        "version": "0.1.0"
    }))
}

/// Error response structure
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

/// Configure API routes
fn config_routes(cfg: &mut web::ServiceConfig) {
    // Health check
    cfg.route("/health", web::get().to(health));
    
    // Auth routes (public)
    cfg.route("/api/auth/register", web::post().to(register));
    cfg.route("/api/auth/login", web::post().to(login));
    
    // User routes (protected)
    cfg.route("/api/users/{id}", web::get().to(get_profile));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("Starting WAVELET Backend API...");
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://wavelet:wavelet@localhost:5432/wavelet".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    log::info!("Database connected successfully");
    
    // JWT service
    let jwt = JwtService::new(None);
    
    // App state
    let app_state = Arc::new(AppState {
        db: pool.clone(),
        jwt: jwt.clone(),
    });
    
    // HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(jwt.clone()))
            .configure(config_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web};

    #[actix_web::test]
    async fn test_health() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health))
        ).await;
        
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
