use crate::{database::Database, error::AppResult, models::WebSession};

#[allow(dead_code)]
pub struct SessionService {
    db: Database,
}

#[allow(dead_code)]
impl SessionService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn get_session_by_token(&self, token: &str) -> AppResult<Option<WebSession>> {
        let session = sqlx::query_as::<_, WebSession>(
            "SELECT session_id, token, discord_user_id, created_at, expires_at 
             FROM web_sessions 
             WHERE token = $1 AND expires_at > EXTRACT(epoch FROM now())"
        )
        .bind(token)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(session)
    }

    pub async fn create_session(&self, token: &str, discord_user_id: i64, expires_in_seconds: i64) -> AppResult<WebSession> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = current_time + expires_in_seconds;

        let session = sqlx::query_as::<_, WebSession>(
            "INSERT INTO web_sessions (session_id, token, discord_user_id, created_at, expires_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING session_id, token, discord_user_id, created_at, expires_at"
        )
        .bind(&session_id)
        .bind(token)
        .bind(discord_user_id)
        .bind(current_time)
        .bind(expires_at)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(session)
    }

    pub async fn cleanup_expired_sessions(&self) -> AppResult<u64> {
        let result = sqlx::query(
            "DELETE FROM web_sessions WHERE expires_at < EXTRACT(epoch FROM now())"
        )
        .execute(&self.db.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
