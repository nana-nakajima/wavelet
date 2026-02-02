// Follow system API handlers
// POST /api/users/{id}/follow - Follow a user
// DELETE /api/users/{id}/follow - Unfollow a user
// GET /api/users/{id}/followers - Get followers
// GET /api/users/{id}/following - Get following

use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::follows::FollowRepository;
use crate::middleware::jwt::JwtService;

/// Follow response
#[derive(Serialize)]
struct FollowResponse {
    success: bool,
    message: String,
}

/// Pagination query params
#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 { 20 }

/// Follow a user
pub async fn follow_user(
    req: HttpRequest,
    path: web::Path<Uuid>,
    jwt: web::Data<JwtService>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    // Extract JWT token
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().json(FollowResponse {
            success: false,
            message: "Missing authorization header".to_string(),
        }),
    };

    let token = auth_header.trim_start_matches("Bearer ");
    let claims = match jwt.validate_token(token) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().json(FollowResponse {
            success: false,
            message: "Invalid token".to_string(),
        }),
    };

    let follower_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(FollowResponse {
            success: false,
            message: "Invalid user ID in token".to_string(),
        }),
    };

    let following_id = path.into_inner();

    // Can't follow yourself
    if follower_id == following_id {
        return HttpResponse::BadRequest().json(FollowResponse {
            success: false,
            message: "Cannot follow yourself".to_string(),
        });
    }

    let repo = FollowRepository::new(pool.get_ref().clone());
    match repo.follow(follower_id, following_id).await {
        Ok(_) => HttpResponse::Ok().json(FollowResponse {
            success: true,
            message: "Successfully followed user".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(FollowResponse {
            success: false,
            message: format!("Failed to follow: {}", e),
        }),
    }
}

/// Unfollow a user
pub async fn unfollow_user(
    req: HttpRequest,
    path: web::Path<Uuid>,
    jwt: web::Data<JwtService>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    // Extract JWT token
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().json(FollowResponse {
            success: false,
            message: "Missing authorization header".to_string(),
        }),
    };

    let token = auth_header.trim_start_matches("Bearer ");
    let claims = match jwt.validate_token(token) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().json(FollowResponse {
            success: false,
            message: "Invalid token".to_string(),
        }),
    };

    let follower_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(FollowResponse {
            success: false,
            message: "Invalid user ID in token".to_string(),
        }),
    };

    let following_id = path.into_inner();

    let repo = FollowRepository::new(pool.get_ref().clone());
    match repo.unfollow(follower_id, following_id).await {
        Ok(deleted) => {
            if deleted {
                HttpResponse::Ok().json(FollowResponse {
                    success: true,
                    message: "Successfully unfollowed user".to_string(),
                })
            } else {
                HttpResponse::Ok().json(FollowResponse {
                    success: true,
                    message: "Was not following this user".to_string(),
                })
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(FollowResponse {
            success: false,
            message: format!("Failed to unfollow: {}", e),
        }),
    }
}

/// Get user's followers
pub async fn get_followers(
    path: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    let user_id = path.into_inner();
    let repo = FollowRepository::new(pool.get_ref().clone());

    match repo.get_followers(user_id, query.limit, query.offset).await {
        Ok(followers) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": followers,
            "pagination": {
                "limit": query.limit,
                "offset": query.offset
            }
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to get followers: {}", e)
        })),
    }
}

/// Get users that this user is following
pub async fn get_following(
    path: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    let user_id = path.into_inner();
    let repo = FollowRepository::new(pool.get_ref().clone());

    match repo.get_following(user_id, query.limit, query.offset).await {
        Ok(following) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": following,
            "pagination": {
                "limit": query.limit,
                "offset": query.offset
            }
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to get following: {}", e)
        })),
    }
}

/// Check if current user is following another user
pub async fn check_following(
    req: HttpRequest,
    path: web::Path<Uuid>,
    jwt: web::Data<JwtService>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    // Extract JWT token
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Missing authorization header"
        })),
    };

    let token = auth_header.trim_start_matches("Bearer ");
    let claims = match jwt.validate_token(token) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Invalid token"
        })),
    };

    let follower_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "Invalid user ID in token"
        })),
    };

    let following_id = path.into_inner();
    let repo = FollowRepository::new(pool.get_ref().clone());

    match repo.is_following(follower_id, following_id).await {
        Ok(is_following) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "is_following": is_following
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to check follow status: {}", e)
        })),
    }
}

/// Configure follow routes
pub fn configure_follow_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/users/{id}")
            .route("/follow", web::post().to(follow_user))
            .route("/follow", web::delete().to(unfollow_user))
            .route("/follow/check", web::get().to(check_following))
            .route("/followers", web::get().to(get_followers))
            .route("/following", web::get().to(get_following))
    );
}
