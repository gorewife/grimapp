use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{
    error::{AppError, AppResult},
    models::{ApiKeyCreate, Game, PlayerStats, ScriptStats, StatsSummary},
    state::AppState,
    utils::validation,
};

// ============================================================================
// Public Read-Only Endpoints (protected by API key middleware)
// ============================================================================

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// List all games with pagination
pub async fn get_games(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<Vec<Game>>> {
    // Validate pagination
    if pagination.limit < 1 || pagination.limit > 100 {
        return Err(AppError::Validation("Limit must be between 1 and 100".to_string()));
    }
    if pagination.offset < 0 {
        return Err(AppError::Validation("Offset must be non-negative".to_string()));
    }

    let games = state.services.game.get_games(pagination.limit, pagination.offset).await?;
    Ok(Json(games))
}

/// Get a specific game by ID
pub async fn get_game_by_id(
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> AppResult<Json<Game>> {
    let game = state.services.game.get_game(game_id).await?
        .ok_or_else(|| AppError::NotFound("Game not found".to_string()))?;
    
    Ok(Json(game))
}

/// Get overall statistics summary
pub async fn get_stats_summary(
    State(state): State<AppState>,
) -> AppResult<Json<StatsSummary>> {
    let total_games = state.services.game.count_total_games().await?;
    let total_players = state.services.game.count_total_players().await?;
    let unique_players = state.services.game.count_unique_players().await?;
    let active_games = state.services.game.count_active_games().await?;

    let stats = StatsSummary {
        total_games,
        total_players,
        unique_players,
        active_games,
    };

    Ok(Json(stats))
}

/// Get player statistics by Discord ID
pub async fn get_player_stats(
    State(state): State<AppState>,
    Path(discord_id): Path<i64>,
) -> AppResult<Json<PlayerStats>> {
    let stats = state.services.game.get_player_stats(discord_id).await?
        .ok_or_else(|| AppError::NotFound("Player not found".to_string()))?;
    
    Ok(Json(stats))
}

/// Get script statistics by script name
pub async fn get_script_stats(
    State(state): State<AppState>,
    Path(script_name): Path<String>,
) -> AppResult<Json<ScriptStats>> {
    validation::validate_script_name(&script_name)?;
    
    let stats = state.services.game.get_script_stats(&script_name).await?
        .ok_or_else(|| AppError::NotFound("Script not found".to_string()))?;
    
    Ok(Json(stats))
}

// ============================================================================
// API Key Management (TODO: should require session token, not API key)
// ============================================================================

pub async fn list_api_keys(
    State(_state): State<AppState>,
) -> AppResult<Json<Vec<String>>> {
    // TODO: Verify session token instead of API key
    // For now, return empty list
    Ok(Json(vec![]))
}

pub async fn create_api_key(
    State(_state): State<AppState>,
    Json(payload): Json<ApiKeyCreate>,
) -> AppResult<StatusCode> {
    // Validate API key name
    validation::validate_api_key_name(&payload.name)?;
    
    // Validate rate limit if provided
    if let Some(rate_limit) = payload.rate_limit {
        validation::validate_rate_limit(rate_limit)?;
    }
    
    // TODO: Verify session token and implement
    Err(AppError::Internal("Not yet implemented".to_string()))
}

pub async fn update_api_key(
    State(_state): State<AppState>,
    Path(_key_id): Path<i32>,
) -> AppResult<StatusCode> {
    // TODO: Implement
    Err(AppError::Internal("Not yet implemented".to_string()))
}

pub async fn delete_api_key(
    State(_state): State<AppState>,
    Path(_key_id): Path<i32>,
) -> AppResult<StatusCode> {
    // TODO: Implement
    Err(AppError::Internal("Not yet implemented".to_string()))
}
