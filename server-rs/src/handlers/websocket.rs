use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::state::{AppState, SessionClients};

/// WebSocket handler with optional path parameters
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    let clients = state.websocket_clients.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, None, clients))
}

/// WebSocket handler with channel path parameter
pub async fn websocket_handler_with_channel(
    ws: WebSocketUpgrade,
    Path(channel): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let clients = state.websocket_clients.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, Some(channel), clients))
}

/// WebSocket handler with channel and client path parameters
pub async fn websocket_handler_with_client(
    ws: WebSocketUpgrade,
    Path((channel, _client)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Response {
    let clients = state.websocket_clients.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, Some(channel), clients))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, session_id_from_path: Option<String>, clients: SessionClients) {
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

    // Use shared session state from AppState
    let clients_for_recv = clients.clone();
    
    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        // Session ID from path or will be determined from first message
        let mut session_id: Option<String> = session_id_from_path.clone();
        
        // If we have session from path, register immediately
        if let Some(ref sid) = session_id {
            let mut clients_lock = clients_for_recv.write().await;
            clients_lock
                .entry(sid.clone())
                .or_insert_with(HashMap::new)
                .insert(client_id.clone(), tx.clone());
            
            tracing::info!(
                "Client {} joined session {} (from path). Total clients: {}",
                client_id,
                sid,
                clients_lock.get(sid).map(|c| c.len()).unwrap_or(0)
            );
        }

        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!("Message from {} in session {:?}: {}", client_id, session_id, text);
                    
                    // Broadcast message to all clients in the session
                    if let Some(ref sid) = session_id {
                        broadcast_to_session(
                            &clients_for_recv,
                            sid,
                            &client_id,
                            &text,
                        ).await;
                    } else {
                        tracing::warn!("Message from {} but no session established (this shouldn't happen)", client_id);
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
