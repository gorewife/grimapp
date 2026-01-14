use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use crate::{models::WsMessage, state::AppState};

/// Client sender type for WebSocket messages
type ClientSender = mpsc::UnboundedSender<Message>;

/// Session-based client registry
/// Maps session_id -> client_id -> sender
type SessionClients = Arc<RwLock<HashMap<String, HashMap<String, ClientSender>>>>;

/// WebSocket handler - upgrades HTTP connection to WebSocket
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> Response {
    ws.on_upgrade(handle_socket)
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4().to_string();

    tracing::info!("WebSocket client connected: {}", client_id);

    // Create a channel for this client
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Spawn task to send messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Initialize session state
    let clients: SessionClients = Arc::new(RwLock::new(HashMap::new()));
    let clients_for_recv = clients.clone();
    
    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        let mut session_id: Option<String> = None;

        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!("Received message from {}: {}", client_id, text);

                    // Parse message
                    match serde_json::from_str::<WsMessage>(&text) {
                        Ok(ws_msg) => {
                            match ws_msg {
                                WsMessage::Connect {
                                    session_id: sid,
                                    discord_id: _,
                                    username: _,
                                } => {
                                    // Store client in session
                                    session_id = Some(sid.clone());
                                    let mut clients_lock = clients_for_recv.write().await;
                                    clients_lock
                                        .entry(sid.clone())
                                        .or_insert_with(HashMap::new)
                                        .insert(client_id.clone(), tx.clone());
                                    
                                    tracing::info!(
                                        "Client {} joined session {}. Total clients: {}",
                                        client_id,
                                        sid,
                                        clients_lock.get(&sid).map(|c| c.len()).unwrap_or(0)
                                    );
                                }
                                WsMessage::GameState { players, night, phase } => {
                                    if let Some(sid) = &session_id {
                                        tracing::debug!(
                                            "Game state update for session {}: {} players, night {}, phase {}",
                                            sid,
                                            players.len(),
                                            night,
                                            phase
                                        );
                                        
                                        // Broadcast to all clients in this session
                                        broadcast_to_session(
                                            &clients_for_recv,
                                            sid,
                                            &client_id,
                                            &text,
                                        ).await;
                                    }
                                }
                                WsMessage::PlayerUpdate { .. } => {
                                    if let Some(sid) = &session_id {
                                        broadcast_to_session(
                                            &clients_for_recv,
                                            sid,
                                            &client_id,
                                            &text,
                                        ).await;
                                    }
                                }
                                WsMessage::Timer { .. } => {
                                    if let Some(sid) = &session_id {
                                        broadcast_to_session(
                                            &clients_for_recv,
                                            sid,
                                            &client_id,
                                            &text,
                                        ).await;
                                    }
                                }
                                WsMessage::Chat { .. } => {
                                    if let Some(sid) = &session_id {
                                        broadcast_to_session(
                                            &clients_for_recv,
                                            sid,
                                            &client_id,
                                            &text,
                                        ).await;
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse message from {}: {}", client_id, e);
                        }
                    }
                }
                Message::Close(_) => {
                    tracing::info!("Client {} disconnected", client_id);
                    break;
                }
                Message::Ping(data) => {
                    // Respond with pong
                    if tx.send(Message::Pong(data)).is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }

        // Cleanup: remove client from session
        if let Some(sid) = session_id {
            let mut clients_lock = clients_for_recv.write().await;
            if let Some(session_clients) = clients_lock.get_mut(&sid) {
                session_clients.remove(&client_id);
                if session_clients.is_empty() {
                    clients_lock.remove(&sid);
                    tracing::info!("Session {} is now empty, removed", sid);
                }
            }
        }

        client_id
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        client_id = (&mut recv_task) => {
            send_task.abort();
            if let Ok(cid) = client_id {
                tracing::info!("Client {} connection closed", cid);
            }
        }
    }
}

/// Broadcast a message to all clients in a session except the sender
async fn broadcast_to_session(
    clients: &SessionClients,
    session_id: &str,
    sender_id: &str,
    message: &str,
) {
    let clients_lock = clients.read().await;
    
    if let Some(session_clients) = clients_lock.get(session_id) {
        let mut failed_clients = Vec::new();
        
        for (client_id, tx) in session_clients.iter() {
            // Don't send back to sender
            if client_id == sender_id {
                continue;
            }
            
            if let Err(_) = tx.send(Message::Text(message.to_string())) {
                failed_clients.push(client_id.clone());
            }
        }
        
        if !failed_clients.is_empty() {
            tracing::warn!(
                "Failed to broadcast to {} clients in session {}",
                failed_clients.len(),
                session_id
            );
        }
    }
}
