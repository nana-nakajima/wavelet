// Preset HTTP handlers
// WAVELET Backend - HTTP request handlers for preset API

use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;
use crate::services::preset_service::{PresetService, PresetServiceError};
use crate::models::preset::{
    CreatePresetRequest, UpdatePresetRequest, SearchQuery, RateRequest,
};
use crate::middleware::auth::AuthUser;
use std::sync::Arc;

/// Type alias for PresetService wrapped in Arc
type PresetServiceState = web::Data<Arc<PresetService>>;

/// Create a new preset - POST /api/presets
#[post("/presets")]
async fn create_preset(
    state: PresetServiceState,
    user: AuthUser,
    data: web::Json<CreatePresetRequest>,
) -> Result<HttpResponse> {
    let response = state.create_preset(user.user_id, data.into_inner()).await;
    
    match response {
        Ok(preset) => Ok(HttpResponse::Created().json(preset)),
        Err(PresetServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Validation error",
                "message": msg
            })))
        }
        Err(e) => {
            log::error!("Error creating preset: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Get a preset by ID - GET /api/presets/{id}
#[get("/presets/{id}")]
async fn get_preset(
    state: PresetServiceState,
    user: Option<AuthUser>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let preset_id = path.into_inner();
    let requester_id = user.map(|u| u.user_id);
    
    let response = state.get_preset(preset_id, requester_id).await;
    
    match response {
        Ok(preset) => Ok(HttpResponse::Ok().json(preset)),
        Err(PresetServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Preset not found"
            })))
        }
        Err(PresetServiceError::AccessDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied",
                "message": "This preset is private"
            })))
        }
        Err(e) => {
            log::error!("Error getting preset: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Search presets - GET /api/presets
#[get("/presets")]
async fn search_presets(
    state: PresetServiceState,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse> {
    let response = state.search_presets(query.into_inner()).await;
    
    match response {
        Ok(presets) => Ok(HttpResponse::Ok().json(presets)),
        Err(e) => {
            log::error!("Error searching presets: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Download preset file - GET /api/presets/{id}/download
#[get("/presets/{id}/download")]
async fn download_preset(
    state: PresetServiceState,
    user: Option<AuthUser>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let preset_id = path.into_inner();
    let requester_id = user.map(|u| u.user_id);
    
    let response = state.download_preset(preset_id, requester_id).await;
    
    match response {
        Ok(data) => {
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(data))
        }
        Err(PresetServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Preset not found"
            })))
        }
        Err(PresetServiceError::AccessDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied",
                "message": "This preset is private"
            })))
        }
        Err(e) => {
            log::error!("Error downloading preset: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Rate a preset - POST /api/presets/{id}/rate
#[post("/presets/{id}/rate")]
async fn rate_preset(
    state: PresetServiceState,
    user: AuthUser,
    path: web::Path<Uuid>,
    data: web::Json<RateRequest>,
) -> Result<HttpResponse> {
    let preset_id = path.into_inner();
    
    let response = state.rate_preset(
        preset_id,
        user.user_id,
        data.rating,
        data.comment.clone(),
    ).await;
    
    match response {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Rating submitted successfully"
        }))),
        Err(PresetServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Preset not found"
            })))
        }
        Err(PresetServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Validation error",
                "message": msg
            })))
        }
        Err(e) => {
            log::error!("Error rating preset: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Update a preset - PUT /api/presets/{id}
#[put("/presets/{id}")]
async fn update_preset(
    state: PresetServiceState,
    user: AuthUser,
    path: web::Path<Uuid>,
    data: web::Json<UpdatePresetRequest>,
) -> Result<HttpResponse> {
    let preset_id = path.into_inner();
    
    let response = state.update_preset(preset_id, user.user_id, data.into_inner()).await;
    
    match response {
        Ok(preset) => Ok(HttpResponse::Ok().json(preset)),
        Err(PresetServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Preset not found"
            })))
        }
        Err(PresetServiceError::AccessDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied",
                "message": "You can only update your own presets"
            })))
        }
        Err(PresetServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Validation error",
                "message": msg
            })))
        }
        Err(e) => {
            log::error!("Error updating preset: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Delete a preset - DELETE /api/presets/{id}
#[delete("/presets/{id}")]
async fn delete_preset(
    state: PresetServiceState,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let preset_id = path.into_inner();
    
    let response = state.delete_preset(preset_id, user.user_id).await;
    
    match response {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Preset deleted successfully"
        }))),
        Err(PresetServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Preset not found"
            })))
        }
        Err(PresetServiceError::AccessDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied",
                "message": "You can only delete your own presets"
            })))
        }
        Err(e) => {
            log::error!("Error deleting preset: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Get presets by user - GET /api/users/{user_id}/presets
#[get("/users/{user_id}/presets")]
async fn get_user_presets(
    state: PresetServiceState,
    path: web::Path<Uuid>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    let response = state.search_presets(query.into_inner()).await;
    
    match response {
        Ok(presets) => Ok(HttpResponse::Ok().json(presets)),
        Err(e) => {
            log::error!("Error getting user presets: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Configure preset routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_preset)
       .service(get_preset)
       .service(search_presets)
       .service(download_preset)
       .service(rate_preset)
       .service(update_preset)
       .service(delete_preset)
       .service(get_user_presets);
}
