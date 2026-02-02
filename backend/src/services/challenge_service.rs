use sqlx::PgPool;
use uuid::Uuid;
use serde::Serialize;
use crate::models::challenge::{Challenge, ChallengeDetail, ChallengeSubmission, NewChallenge, NewSubmission};
use chrono::{DateTime, Utc};

/// Challenge data model
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct ChallengeModel {
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

/// Submission data model
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct SubmissionModel {
    pub id: i32,
    pub challenge_id: i32,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub project_name: String,
    pub description: Option<String>,
    pub download_url: String,
    pub votes: Option<i32>,
    pub submitted_at: DateTime<Utc>,
}

/// Challenge service
pub struct ChallengeService<'a> {
    pool: &'a PgPool,
}

impl<'a> ChallengeService<'a> {
    /// Create a new challenge service
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new challenge
    pub async fn create_challenge(&self, data: NewChallenge, creator_id: &Uuid) -> Result<Challenge, sqlx::Error> {
        // Insert the challenge
        sqlx::query!(
            r#"
            INSERT INTO challenges (title, description, theme, start_date, end_date, status, created_by)
            VALUES ($1, $2, $3, $4, $5, 'active', $6)
            "#,
            data.title, data.description, data.theme, data.start_date, data.end_date, creator_id
        )
        .execute(self.pool)
        .await?;

        // Get the created challenge
        let challenge = sqlx::query_as!(
            ChallengeModel,
            r#"
            SELECT id, title, description, theme, start_date, end_date, status, 
                   0 as participant_count, created_by, created_at
            FROM challenges 
            WHERE created_by = $1 AND title = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            creator_id, data.title
        )
        .fetch_one(self.pool)
        .await?;

        Ok(Challenge {
            id: challenge.id,
            title: challenge.title,
            description: challenge.description,
            theme: challenge.theme,
            start_date: challenge.start_date,
            end_date: challenge.end_date,
            status: challenge.status,
            participant_count: challenge.participant_count,
            created_by: challenge.created_by,
            created_at: challenge.created_at,
        })
    }

    /// Get all active challenges
    pub async fn get_active_challenges(&self) -> Result<Vec<Challenge>, sqlx::Error> {
        let challenges = sqlx::query_as!(
            ChallengeModel,
            r#"
            SELECT c.id, c.title, c.description, c.theme, c.start_date, c.end_date, 
                   c.status, 
                   (SELECT COUNT(*) FROM challenge_submissions WHERE challenge_id = c.id) as participant_count, 
                   c.created_by, c.created_at
            FROM challenges c
            WHERE c.status IN ('active', 'upcoming')
            ORDER BY c.created_at DESC
            "#
        )
        .fetch_all(self.pool)
        .await?;

        Ok(challenges.into_iter().map(|c| Challenge {
            id: c.id,
            title: c.title,
            description: c.description,
            theme: c.theme,
            start_date: c.start_date,
            end_date: c.end_date,
            status: c.status,
            participant_count: c.participant_count,
            created_by: c.created_by,
            created_at: c.created_at,
        }).collect())
    }

    /// Get challenge with submissions
    pub async fn get_challenge_with_submissions(&self, challenge_id: i32) -> Result<Option<ChallengeDetail>, sqlx::Error> {
        let challenge = sqlx::query_as!(
            ChallengeModel,
            r#"
            SELECT id, title, description, theme, start_date, end_date, status, 
                   0 as participant_count, created_by, created_at
            FROM challenges WHERE id = $1
            "#,
            challenge_id
        )
        .fetch_optional(self.pool)
        .await?;

        match challenge {
            Some(c) => {
                let submissions = self.get_submissions(challenge_id).await?;
                Ok(Some(ChallengeDetail {
                    id: c.id,
                    title: c.title,
                    description: c.description,
                    theme: c.theme,
                    start_date: c.start_date,
                    end_date: c.end_date,
                    status: c.status,
                    participant_count: Some(submissions.len() as i32),
                    created_by: c.created_by,
                    created_at: c.created_at,
                    submissions,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get submissions for a challenge
    pub async fn get_submissions(&self, challenge_id: i32) -> Result<Vec<ChallengeSubmission>, sqlx::Error> {
        let submissions = sqlx::query_as!(
            SubmissionModel,
            r#"
            SELECT s.id, s.challenge_id, s.user_id, u.username, 
                   s.project_name, s.description, s.download_url, 
                   COALESCE(s.votes, 0) as votes, s.submitted_at
            FROM challenge_submissions s
            JOIN users u ON s.user_id = u.id
            WHERE s.challenge_id = $1
            ORDER BY s.votes DESC, s.submitted_at DESC
            "#,
            challenge_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(submissions.into_iter().enumerate().map(|(i, s)| ChallengeSubmission {
            id: s.id,
            challenge_id: s.challenge_id,
            user_id: s.user_id,
            username: s.username.unwrap_or_default(),
            project_name: s.project_name,
            description: s.description.unwrap_or_default(),
            download_url: s.download_url,
            votes: s.votes.unwrap_or(0),
            rank: (i + 1) as i32,
            submitted_at: s.submitted_at,
        }).collect())
    }

    /// Create a submission
    pub async fn create_submission(&self, data: NewSubmission, user_id: &Uuid) -> Result<ChallengeSubmission, sqlx::Error> {
        let submission = sqlx::query_as!(
            SubmissionModel,
            r#"
            INSERT INTO challenge_submissions (challenge_id, user_id, project_name, description, download_url, votes)
            VALUES ($1, $2, $3, $4, $5, 0)
            RETURNING id, challenge_id, user_id, '' as username, project_name, description, download_url, votes, submitted_at
            "#,
            data.challenge_id, user_id, data.project_name, data.description, data.download_url
        )
        .fetch_one(self.pool)
        .await?;

        // Get username
        let user = sqlx::query!(r#"SELECT username FROM users WHERE id = $1"#, user_id)
            .fetch_one(self.pool)
            .await?;

        Ok(ChallengeSubmission {
            id: submission.id,
            challenge_id: submission.challenge_id,
            user_id: submission.user_id,
            username: user.username.unwrap_or_default(),
            project_name: submission.project_name,
            description: submission.description.unwrap_or_default(),
            download_url: submission.download_url,
            votes: submission.votes.unwrap_or(0),
            rank: 0,
            submitted_at: submission.submitted_at,
        })
    }

    /// Vote for a submission
    pub async fn vote_submission(&self, submission_id: i32, user_id: &Uuid, vote: bool) -> Result<(), sqlx::Error> {
        // Check if user already voted
        let existing = sqlx::query!(
            r#"SELECT id FROM challenge_votes WHERE submission_id = $1 AND user_id = $2"#,
            submission_id, user_id
        )
        .fetch_optional(self.pool)
        .await?;

        if existing.is_some() {
            // Update existing vote
            sqlx::query!(
                r#"UPDATE challenge_votes SET vote = $1 WHERE submission_id = $2 AND user_id = $3"#,
                vote, submission_id, user_id
            )
            .execute(self.pool)
            .await?;
        } else {
            // Create new vote
            sqlx::query!(
                r#"INSERT INTO challenge_votes (submission_id, user_id, vote) VALUES ($1, $2, $3)"#,
                submission_id, user_id, vote
            )
            .execute(self.pool)
            .await?;
        }

        // Update vote count
        sqlx::query!(
            r#"
            UPDATE challenge_submissions s
            SET votes = (
                SELECT COUNT(*) FROM challenge_votes WHERE submission_id = s.id AND vote = true
            ) - (
                SELECT COUNT(*) FROM challenge_votes WHERE submission_id = s.id AND vote = false
            )
            WHERE id = $1
            "#,
            submission_id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get leaderboard for a challenge
    pub async fn get_leaderboard(&self, challenge_id: i32) -> Result<Vec<ChallengeSubmission>, sqlx::Error> {
        self.get_submissions(challenge_id).await
    }
}
