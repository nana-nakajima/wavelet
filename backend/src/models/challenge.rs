use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Challenge model
#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub theme: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: String,
    pub participant_count: Option<i32>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Challenge with submissions
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeDetail {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub theme: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: String,
    pub participant_count: Option<i32>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub submissions: Vec<ChallengeSubmission>,
}

/// Challenge submission
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeSubmission {
    pub id: i32,
    pub challenge_id: i32,
    pub user_id: Uuid,
    pub username: String,
    pub project_name: String,
    pub description: String,
    pub download_url: String,
    pub votes: i32,
    pub rank: i32,
    pub submitted_at: DateTime<Utc>,
}

/// New challenge request
#[derive(Debug, Serialize, Deserialize)]
pub struct NewChallenge {
    pub title: String,
    pub description: String,
    pub theme: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub token: String,
}

/// New submission request
#[derive(Debug, Serialize, Deserialize)]
pub struct NewSubmission {
    pub challenge_id: i32,
    pub project_name: String,
    pub description: String,
    pub download_url: String,
    pub token: String,
}
