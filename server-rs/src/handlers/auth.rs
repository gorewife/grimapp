use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Serialize};
use crate::{error::AppResult, state::AppState};

#[derive(Deserialize)]
pub struct DiscordOAuthQuery {
    redirect_uri: Option<String>,
}

#[derive(Deserialize)]
pub struct DiscordCallbackQuery {
    code: Option<String>,
    error: Option<String>,
    redirect_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: String,
    scope: String,
}

#[derive(Serialize, Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
    avatar: Option<String>,
}

pub async fn discord_oauth(
    State(state): State<AppState>,
    Query(query): Query<DiscordOAuthQuery>,
) -> impl IntoResponse {
    // Use provided redirect_uri or fall back to config
    let redirect_uri = query.redirect_uri
        .unwrap_or_else(|| state.config.discord_redirect_uri.clone());

    println!("🔐 Discord OAuth initiated with redirect_uri: {}", redirect_uri);

    let redirect_url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify",
        state.config.discord_client_id,
        urlencoding::encode(&redirect_uri)
    );

    println!("🔗 Redirecting to Discord: {}", redirect_url);

    Redirect::temporary(&redirect_url)
}

pub async fn discord_callback(
    State(state): State<AppState>,
    Query(query): Query<DiscordCallbackQuery>,
) -> AppResult<Redirect> {
    println!("🎯 Discord callback received!");
    println!("   - code present: {}", query.code.is_some());
    println!("   - error: {:?}", query.error);
    
    if let Some(error) = query.error {
        println!("❌ OAuth error: {}", error);
        // Redirect back to app with error
        let app_url = if cfg!(debug_assertions) {
            "http://localhost:1420"
        } else {
            "tauri://localhost"
        };
        let redirect_url = format!("{}/?discord_login=error&error={}", app_url, urlencoding::encode(&error));
        return Ok(Redirect::temporary(&redirect_url));
    }

    let code = query.code.ok_or_else(|| {
        crate::error::AppError::BadRequest("Missing authorization code".to_string())
    })?;

    // Use provided redirect_uri or fall back to config
    let redirect_uri = query.redirect_uri
        .unwrap_or_else(|| state.config.discord_redirect_uri.clone());

    println!("📝 Exchanging code for token...");
    
    let client = reqwest::Client::new();
    
    // Exchange code for token
    let token_response = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", state.config.discord_client_id.as_str()),
            ("client_secret", state.config.discord_client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Discord API error: {}", e)))?;
    
    println!("✅ Token response received: {}", token_response.status());

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(crate::error::AppError::Internal(format!("Discord token exchange failed: {}", error_text)));
    }

    let token_data: DiscordTokenResponse = token_response
        .json()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to parse Discord response: {}", e)))?;

    println!("👤 Fetching user info...");
    
    // Fetch user info
    let user_response = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", token_data.access_token))
        .send()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to fetch user info: {}", e)))?;

    if !user_response.status().is_success() {
        return Err(crate::error::AppError::Internal("Failed to fetch Discord user info".to_string()));
    }

    let discord_user: DiscordUser = user_response
        .json()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to parse user info: {}", e)))?;

    println!("✅ User info received: {} ({})", discord_user.username, discord_user.id);

    let discord_id: i64 = discord_user.id.parse()
        .map_err(|_| crate::error::AppError::Internal("Invalid Discord ID".to_string()))?;

    // Create session token
    let session_token = uuid::Uuid::new_v4().to_string();

    println!("💾 Creating session...");
    
    // Store session in database (expires in 30 days)
    let session = state.services.session
        .create_session(&session_token, discord_id, 30 * 24 * 60 * 60)
        .await?;

    println!("✅ Session created: {}", session.session_id);

    // Build redirect back to app with session data in URL fragment (for Tauri)
    let app_url = if cfg!(debug_assertions) {
        "http://localhost:5173" // Vite dev server (Tauri uses this)
    } else {
        "tauri://localhost" // Tauri production
    };

    let redirect_url = format!(
        "{}/?discord_login=success&user_id={}&username={}&avatar={}&token={}&session_id={}",
        app_url,
        discord_user.id,
        urlencoding::encode(&discord_user.username),
        discord_user.avatar.as_deref().unwrap_or(""),
        session_token,
        session.session_id
    );

    println!("🔄 Redirecting to app: {}", redirect_url);

    Ok(Redirect::temporary(&redirect_url))
}
