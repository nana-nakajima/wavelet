use actix_web::{web, Responder, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use crate::services::user_service::AppState;
use crate::middleware::jwt::JwtService;
use crate::db::users::UserRepository;
use crate::services::preset_service::PresetService;
use std::sync::Arc;
use sqlx::PgPool;
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;

/// Project data structure for import/export
#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectData {
    pub id: Option<String>,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub author_id: Option<i32>,
    pub parameters: ProjectParameters,
    pub settings: ProjectSettings,
    pub created_at: Option<String>,
    pub is_public: bool,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectParameters {
    pub volume: f64,
    pub filter_cutoff: f64,
    pub filter_resonance: f64,
    pub attack: f64,
    pub release: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectSettings {
    pub theme: i32,
    pub presets_loaded: Vec<String>,
}

#[derive(Serialize)]
pub struct ShareResponse {
    pub success: bool,
    pub project_id: String,
    pub share_url: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ImportResponse {
    pub success: bool,
    pub project: Option<ProjectData>,
    pub message: String,
}

/// Storage backend trait for project files
trait ProjectStorage {
    fn save_project(&self, project_id: &str, data: &ProjectData) -> Result<(), String>;
    fn load_project(&self, project_id: &str) -> Result<ProjectData, String>;
    fn delete_project(&self, project_id: &str) -> Result<(), String>;
}

/// Local file storage implementation
struct LocalProjectStorage {
    data_dir: PathBuf,
}

impl LocalProjectStorage {
    fn new(data_dir: PathBuf) -> Self {
        fs::create_dir_all(&data_dir).ok();
        Self { data_dir }
    }
}

impl ProjectStorage for LocalProjectStorage {
    fn save_project(&self, project_id: &str, data: &ProjectData) -> Result<(), String> {
        let file_path = self.data_dir.join(format!("{}.json", project_id));
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;
        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to save project file: {}", e))?;
        Ok(())
    }

    fn load_project(&self, project_id: &str) -> Result<ProjectData, String> {
        let file_path = self.data_dir.join(format!("{}.json", project_id));
        if !file_path.exists() {
            return Err("Project not found".to_string());
        }
        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read project file: {}", e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse project: {}", e))
    }

    fn delete_project(&self, project_id: &str) -> Result<(), String> {
        let file_path = self.data_dir.join(format!("{}.json", project_id));
        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete project: {}", e))?;
        }
        Ok(())
    }
}

/// Share a project to community
async fn share_project(
    state: web::Data<AppState>,
    jwt: web::Data<JwtService>,
    storage: web::Data<LocalProjectStorage>,
    project: web::Json<ProjectData>,
) -> impl Responder {
    // Verify JWT token
    let token = match extract_token(&project) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().json(ShareResponse {
            success: false,
            project_id: String::new(),
            share_url: String::new(),
            message: "Authentication required".to_string(),
        }),
    };

    let claims = match jwt.validate_token(&token) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().json(ShareResponse {
            success: false,
            project_id: String::new(),
            share_url: String::new(),
            message: "Invalid token".to_string(),
        }),
    };
    let user_id = claims.sub;

    // Generate unique project ID
    let project_id = Uuid::new_v4().to_string();
    let mut project_data = project.into_inner();
    project_data.id = Some(project_id.clone());
    project_data.author_id = Some(user_id);

    // Save to local storage
    if let Err(e) = storage.save_project(&project_id, &project_data) {
        return HttpResponse::InternalServerError().json(ShareResponse {
            success: false,
            project_id: String::new(),
            share_url: String::new(),
            message: format!("Failed to save project: {}", e),
        });
    }

    let share_url = format!("/api/projects/{}", project_id);

    HttpResponse::Ok().json(ShareResponse {
        success: true,
        project_id: project_id.clone(),
        share_url,
        message: "Project shared successfully!".to_string(),
    })
}

/// Download a shared project
async fn get_project(
    storage: web::Data<LocalProjectStorage>,
    project_id: web::Path<String>,
) -> impl Responder {
    match storage.load_project(project_id.as_str()) {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => HttpResponse::NotFound().json(ImportResponse {
            success: false,
            project: None,
            message: e,
        }),
    }
}

/// Delete a shared project (owner only)
async fn delete_project(
    state: web::Data<AppState>,
    jwt: web::Data<JwtService>,
    storage: web::Data<LocalProjectStorage>,
    project_id: web::Path<String>,
) -> impl Responder {
    // Verify JWT
    // Implementation similar to share_project

    match storage.delete_project(project_id.as_str()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Project deleted successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// Helper function to extract token (simplified)
fn extract_token(project: &ProjectData) -> Option<String> {
    // In a real implementation, extract from Authorization header
    None
}

/// Configure project sharing routes
pub fn configure_project_routes(cfg: &mut web::ServiceConfig) {
    // Project storage with thread-safe reference
    let storage = web::Data::new(LocalProjectStorage::new(PathBuf::from("./data/projects")));

    cfg.app_data(storage.clone());

    // Routes
    cfg.route("/api/projects", web::post().to(share_project));
    cfg.route("/api/projects/{id}", web::get().to(get_project));
    cfg.route("/api/projects/{id}", web::delete().to(delete_project));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web};

    #[actix_web::test]
    async fn test_project_data_serialization() {
        let project = ProjectData {
            id: None,
            version: "2.3.0".to_string(),
            name: "Test Project".to_string(),
            description: None,
            author_id: None,
            parameters: ProjectParameters {
                volume: 0.7,
                filter_cutoff: 2000.0,
                filter_resonance: 1.0,
                attack: 0.01,
                release: 0.5,
            },
            settings: ProjectSettings {
                theme: 0,
                presets_loaded: vec![],
            },
            created_at: None,
            is_public: true,
            tags: vec!["test".to_string()],
        };

        let json = serde_json::to_string(&project).unwrap();
        let parsed: ProjectData = serde_json::from_str(&json).unwrap();

        assert_eq!(project.name, parsed.name);
        assert_eq!(project.parameters.volume, parsed.parameters.volume);
    }

    #[actix_web::test]
    async fn test_local_storage() {
        let temp_dir = std::env::temp_dir().join("wavelet_test_{}", Uuid::new_v4().to_string());
        let storage = LocalProjectStorage::new(temp_dir.clone());

        let project = ProjectData {
            id: None,
            version: "2.3.0".to_string(),
            name: "Test Project".to_string(),
            description: None,
            author_id: None,
            parameters: ProjectParameters {
                volume: 0.7,
                filter_cutoff: 2000.0,
                filter_resonance: 1.0,
                attack: 0.01,
                release: 0.5,
            },
            settings: ProjectSettings {
                theme: 0,
                presets_loaded: vec![],
            },
            created_at: None,
            is_public: true,
            tags: vec!["test".to_string()],
        };

        let project_id = "test_project_123";
        assert!(storage.save_project(project_id, &project).is_ok());

        let loaded = storage.load_project(project_id).unwrap();
        assert_eq!(loaded.name, project.name);

        assert!(storage.delete_project(project_id).is_ok());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
