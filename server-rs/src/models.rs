use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// Session Models (Discord bot sessions)
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub guild_id: i64,
    pub category_id: i64,
    pub destination_channel_id: Option<i64>,
    pub grimoire_link: Option<String>,
    pub exception_channel_id: Option<i64>,
    pub announce_channel_id: Option<i64>,
    pub active_game_id: Option<i32>,
    pub created_at: f64,
    pub last_active: f64,
    pub storyteller_user_id: Option<i64>,
    pub session_code: Option<String>,
}

// ============================================================================
// Game Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Game {
    pub game_id: i32,
    pub guild_id: i64,
    pub script: String,
    pub custom_name: Option<String>,
    pub start_time: f64,
    pub end_time: Option<f64>,
    pub winner: Option<String>,
    pub player_count: Option<i32>,
    #[sqlx(json)]
    pub players: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub storyteller_id: Option<i64>,
    pub category_id: Option<i64>,
    pub storyteller_user_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GamePlayer {
    pub id: i32,
    pub game_id: i32,
    pub discord_id: Option<i64>,
    pub player_name: String,
    pub seat_number: i32,
    pub final_role_id: Option<String>,
    pub final_role_name: Option<String>,
    pub final_team: Option<String>,
    pub survived: Option<bool>,
    pub winning_team: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub starting_role_id: Option<String>,
    pub starting_role_name: Option<String>,
    pub starting_team: Option<String>,
}

// ============================================================================
// API Key Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: i32,
    pub key_hash: String,
    pub name: String,
    pub discord_user_id: Option<String>,
    pub rate_limit: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub last_used_at: Option<NaiveDateTime>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

// ============================================================================
// Web Session Models
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebSession {
    pub session_id: String,  // TEXT in database
    pub token: String,
    pub discord_user_id: Option<i64>,
    pub created_at: Option<i64>,  // BIGINT unix timestamp
    pub expires_at: i64,  // BIGINT unix timestamp
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyCreate {
    pub name: String,
    pub rate_limit: Option<i32>,
    pub notes: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: i32,
    pub name: String,
    pub key: String, // Only returned on creation
    pub rate_limit: i32,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

// ============================================================================
// Stats Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsSummary {
    pub total_games: i64,
    pub total_players: i64,
    pub unique_players: i64,
    pub active_games: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PlayerStats {
    pub discord_id: i64,
    pub player_name: String,
    pub games_played: i64,
    pub wins: i64,
    pub losses: i64,
    pub survival_rate: f64,
    pub favorite_role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ScriptStats {
    pub script_name: String,
    pub games_played: i64,
    pub good_wins: i64,
    pub evil_wins: i64,
    pub average_player_count: f64,
}

// ============================================================================
// WebSocket Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsMessage {
    Connect {
        session_id: String,
        discord_id: Option<String>,
        username: Option<String>,
    },
    GameState {
        players: Vec<WsPlayer>,
        night: i32,
        phase: String,
    },
    PlayerUpdate {
        player: WsPlayer,
    },
    Timer {
        duration: i32,
        remaining: i32,
    },
    Chat {
        from: String,
        message: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPlayer {
    pub id: String,
    pub name: String,
    pub character: Option<String>,
    pub is_dead: bool,
    pub is_storyteller: bool,
}
