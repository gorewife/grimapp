//! Authentication middleware for API key verification

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};

use crate::{error::AppError, models::ApiKey, state::AppState};

/// Extension type to store authenticated API key in request
#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthenticatedApiKey(pub ApiKey);

/// Middleware to verify API key from X-API-Key header
pub async fn verify_api_key(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing X-API-Key header".to_string()))?;

    // Hash the key
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    // Look up key in database
    let api_key_record = sqlx::query_as::<_, ApiKey>(
        "SELECT id, key_hash, name, discord_user_id, rate_limit, created_at, last_used_at, is_active, notes 
         FROM api_keys 
         WHERE key_hash = $1 AND is_active = true"
    )
    .bind(&key_hash)
    .fetch_optional(&state.database.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()))?;

    // Check rate limit
    let rate_limit = api_key_record.rate_limit.unwrap_or(100) as u32;
    let allowed = state
        .services
        .rate_limit
        .check_rate_limit(
            &api_key_record.id.to_string(),
            rate_limit,
            state.config.rate_limit_window_ms,
        )
        .await;

    if !allowed {
        return Err(AppError::RateLimitExceeded);
    }

    // Update last_used_at (fire and forget)
    let pool = state.database.pool.clone();
    let key_id = api_key_record.id;
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE api_keys SET last_used_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(key_id)
            .execute(&pool)
            .await;
    });

    // Store authenticated key in request extensions
    request.extensions_mut().insert(AuthenticatedApiKey(api_key_record));

    Ok(next.run(request).await)
}
