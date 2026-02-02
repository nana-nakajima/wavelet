use crate::models::challenge::{Challenge, ChallengeSubmission, NewChallenge, NewSubmission};
use crate::services::challenge_service::ChallengeService;
use crate::middleware::jwt::JwtService;
use crate::services::user_service::AppState;
use actix_web::{web, Responder, HttpResponse, Result};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};

/// Challenge response structure
#[derive(Serialize)]
struct ChallengeResponse {
    id: i32,
    title: String,
    description: String,
    theme: String,
    start_date: chrono::DateTime<chrono::Utc>,
    end_date: chrono::DateTime<chrono::Utc>,
    status: String,
    participant_count: i32,
    created_by: i32,
}

/// Submission response structure
#[derive(Serialize)]
struct SubmissionResponse {
    id: i32,
    challenge_id: i32,
    user_id: i32,
    username: String,
    project_name: String,
    description: String,
    download_url: String,
    votes: i32,
    rank: i32,
    submitted_at: chrono::DateTime<chrono::Utc>,
}

/// Create a new challenge (admin only in real app, any user for demo)
async fn create_challenge(
    data: web::Json<NewChallenge>,
    state: web::Data<AppState>,
    jwt: web::Data<JwtService>,
) -> Result<impl Responder> {
    // Verify JWT
    let claims = match jwt.validate_token(&data.token) {
        Ok(claims) => claims,
        Err(_) => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Invalid or expired token"
        }))),
    };

    let service = ChallengeService::new(&state.db);
    let challenge = service.create_challenge(data.into_inner(), &claims.sub).await;

    match challenge {
        Ok(challenge) => Ok(HttpResponse::Created().json(ChallengeResponse {
            id: challenge.id,
            title: challenge.title,
            description: challenge.description,
            theme: challenge.theme,
            start_date: challenge.start_date,
            end_date: challenge.end_date,
            status: challenge.status,
            participant_count: 0,
            created_by: challenge.created_by,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error",
            "message": e.to_string()
        }))),
    }
}

/// Get active challenges
async fn get_challenges(state: web::Data<AppState>) -> Result<impl Responder> {
    let service = ChallengeService::new(&state.db);
    let challenges = service.get_active_challenges().await;

    Ok(HttpResponse::Ok().json(challenges))
}

/// Get challenge details with submissions
async fn get_challenge_detail(
    path: web::Path<i32>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let challenge_id = path.into_inner();
    let service = ChallengeService::new(&state.db);
    
    match service.get_challenge_with_submissions(challenge_id).await {
        Ok(Some(data)) => Ok(HttpResponse::Ok().json(data)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Not found",
            "message": "Challenge not found"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error",
            "message": e.to_string()
        }))),
    }
}

/// Submit a project to a challenge
async fn submit_project(
    data: web::Json<NewSubmission>,
    state: web::Data<AppState>,
    jwt: web::Data<JwtService>,
) -> Result<impl Responder> {
    // Verify JWT
    let claims = match jwt.validate_token(&data.token) {
        Ok(claims) => claims,
        Err(_) => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Invalid or expired token"
        }))),
    };

    let service = ChallengeService::new(&state.db);
    let submission = service.create_submission(data.into_inner(), &claims.sub).await;

    match submission {
        Ok(submission) => Ok(HttpResponse::Created().json(SubmissionResponse {
            id: submission.id,
            challenge_id: submission.challenge_id,
            user_id: submission.user_id,
            username: submission.username,
            project_name: submission.project_name,
            description: submission.description,
            download_url: submission.download_url,
            votes: submission.votes,
            rank: 0,
            submitted_at: submission.submitted_at,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error",
            "message": e.to_string()
        }))),
    }
}

/// Vote for a submission
async fn vote_submission(
    path: web::Path<(i32, i32)>,
    data: web::Json<VoteRequest>,
    state: web::Data<AppState>,
    jwt: web::Data<JwtService>,
) -> Result<impl Responder> {
    let (challenge_id, submission_id) = path.into_inner();
    
    // Verify JWT
    let claims = match jwt.validate_token(&data.token) {
        Ok(claims) => claims,
        Err(_) => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Invalid or expired token"
        }))),
    };

    let service = ChallengeService::new(&state.db);
    let result = service.vote_submission(submission_id, &claims.sub, data.vote).await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Vote recorded successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error",
            "message": e.to_string()
        }))),
    }
}

/// Get leaderboard for a challenge
async fn get_leaderboard(
    path: web::Path<i32>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let challenge_id = path.into_inner();
    let service = ChallengeService::new(&state.db);
    let leaderboard = service.get_leaderboard(challenge_id).await;

    Ok(HttpResponse::Ok().json(leaderboard))
}

/// Configure challenge routes
pub fn configure_challenge_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/api/challenges", web::get().to(get_challenges))
        .route("/api/challenges", web::post().to(create_challenge))
        .route("/api/challenges/{id}", web::get().to(get_challenge_detail))
        .route("/api/challenges/{id}/leaderboard", web::get().to(get_leaderboard))
        .route("/api/challenges/{challenge_id}/submissions", web::post().to(submit_project))
        .route("/api/challenges/{challenge_id}/submissions/{submission_id}/vote", web::post().to(vote_submission));
}

#[derive(Deserialize)]
struct VoteRequest {
    token: String,
    vote: bool, // true = upvote, false = downvote
}
