use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

const DATA_FILE: &str = "./data/tasks.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String, // "todo", "in_progress", "review", "done"
    pub priority: String, // "low", "medium", "high", "urgent"
    pub category: String,
    pub assignee: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStore {
    pub tasks: HashMap<String, Task>,
}

impl TaskStore {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
}

fn load_tasks() -> TaskStore {
    if Path::new(DATA_FILE).exists() {
        match fs::read_to_string(DATA_FILE) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_else(|_| TaskStore::new())
            }
            Err(_) => TaskStore::new(),
        }
    } else {
        TaskStore::new()
    }
}

fn save_tasks(store: &TaskStore) -> Result<(), std::io::Error> {
    if let Some(parent) = Path::new(DATA_FILE).parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(store)?;
    fs::write(DATA_FILE, json)
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/index.html"))
}

async fn get_tasks(data: web::Data<Mutex<TaskStore>>) -> HttpResponse {
    let store = data.lock().unwrap();
    let tasks: Vec<Task> = store.tasks.values().cloned().collect();
    HttpResponse::Ok().json(tasks)
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: String,
    priority: String,
    category: String,
    assignee: String,
}

async fn create_task(
    data: web::Data<Mutex<TaskStore>>,
    req: web::Json<CreateTaskRequest>,
) -> HttpResponse {
    let mut store = data.lock().unwrap();
    
    let task = Task {
        id: Uuid::new_v4().to_string(),
        title: req.title.clone(),
        description: req.description.clone(),
        status: "todo".to_string(),
        priority: req.priority.clone(),
        category: req.category.clone(),
        assignee: req.assignee.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        comments: Vec::new(),
    };
    
    store.tasks.insert(task.id.clone(), task.clone());
    
    if let Err(e) = save_tasks(&store) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to save: {}", e)
        }));
    }
    
    HttpResponse::Ok().json(task)
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    category: Option<String>,
    assignee: Option<String>,
}

async fn update_task(
    data: web::Data<Mutex<TaskStore>>,
    task_id: web::Path<String>,
    req: web::Json<UpdateTaskRequest>,
) -> HttpResponse {
    let mut store = data.lock().unwrap();
    
    let task_updated = if let Some(task) = store.tasks.get_mut(&task_id.to_string()) {
        if let Some(title) = &req.title {
            task.title = title.clone();
        }
        if let Some(description) = &req.description {
            task.description = description.clone();
        }
        if let Some(status) = &req.status {
            task.status = status.clone();
        }
        if let Some(priority) = &req.priority {
            task.priority = priority.clone();
        }
        if let Some(category) = &req.category {
            task.category = category.clone();
        }
        if let Some(assignee) = &req.assignee {
            task.assignee = assignee.clone();
        }
        task.updated_at = Utc::now();
        true
    } else {
        false
    };
    
    drop(store); // 释放锁
    
    if !task_updated {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Task not found"
        }));
    }
    
    // 保存
    let store = data.lock().unwrap();
    if let Err(e) = save_tasks(&store) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to save: {}", e)
        }));
    }
    
    let task = store.tasks.get(&task_id.to_string()).unwrap().clone();
    HttpResponse::Ok().json(task)
}

async fn delete_task(
    data: web::Data<Mutex<TaskStore>>,
    task_id: web::Path<String>,
) -> HttpResponse {
    let mut store = data.lock().unwrap();
    
    if store.tasks.remove(&task_id.to_string()).is_some() {
        if let Err(e) = save_tasks(&store) {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to save: {}", e)
            }));
        }
        HttpResponse::Ok().json(serde_json::json!({ "success": true }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Task not found"
        }))
    }
}

#[derive(Deserialize)]
struct AddCommentRequest {
    content: String,
    author: String,
}

async fn add_comment(
    data: web::Data<Mutex<TaskStore>>,
    task_id: web::Path<String>,
    req: web::Json<AddCommentRequest>,
) -> HttpResponse {
    let mut store = data.lock().unwrap();
    
    let task_updated = if let Some(task) = store.tasks.get_mut(&task_id.to_string()) {
        let comment = Comment {
            id: Uuid::new_v4().to_string(),
            content: req.content.clone(),
            author: req.author.clone(),
            created_at: Utc::now(),
        };
        
        task.comments.push(comment);
        task.updated_at = Utc::now();
        true
    } else {
        false
    };
    
    drop(store);
    
    if !task_updated {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Task not found"
        }));
    }
    
    let store = data.lock().unwrap();
    if let Err(e) = save_tasks(&store) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to save: {}", e)
        }));
    }
    
    let task = store.tasks.get(&task_id.to_string()).unwrap().clone();
    HttpResponse::Ok().json(task)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let task_store = web::Data::new(Mutex::new(load_tasks()));
    
    println!("🚀 Task Center starting at http://localhost:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(task_store.clone())
            .route("/", web::get().to(index))
            .route("/api/tasks", web::get().to(get_tasks))
            .route("/api/tasks", web::post().to(create_task))
            .route("/api/tasks/{id}", web::put().to(update_task))
            .route("/api/tasks/{id}", web::delete().to(delete_task))
            .route("/api/tasks/{id}/comments", web::post().to(add_comment))
            .service(actix_files::Files::new("/static", "./static"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
