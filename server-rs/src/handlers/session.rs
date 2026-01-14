use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::{error::AppResult, state::AppState};

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub discord_user_id: String,
}

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub token: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> AppResult<impl IntoResponse> {
    let discord_user_id: i64 = payload.discord_user_id.parse()
        .map_err(|_| crate::error::AppError::BadRequest("Invalid Discord user ID".to_string()))?;

    // Generate session token
    let token = uuid::Uuid::new_v4().to_string();

    // Create session (expires in 30 days)
    let session = state.services.session
        .create_session(&token, discord_user_id, 30 * 24 * 60 * 60)
        .await?;

    Ok((
        StatusCode::OK,
        Json(CreateSessionResponse {
            token: session.token,
            session_id: session.session_id,
        })
    ))
}
