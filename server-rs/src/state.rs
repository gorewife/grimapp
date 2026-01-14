use crate::{config::Config, database::Database, services::ServiceContainer};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use axum::extract::ws::Message;

/// Client sender type for WebSocket messages
type ClientSender = mpsc::UnboundedSender<Message>;

/// Session-based client registry
/// Maps session_id -> client_id -> sender
pub type SessionClients = Arc<RwLock<HashMap<String, HashMap<String, ClientSender>>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
    pub services: Arc<ServiceContainer>,
    pub websocket_clients: SessionClients,
}

impl AppState {
    pub fn new(config: Config, database: Database, services: ServiceContainer) -> Self {
        Self {
            config,
            database,
            services: Arc::new(services),
            websocket_clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
