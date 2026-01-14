# Grimlive Rust Server

Blood on the Clocktower WebSocket server written in Rust.

## Prerequisites

- Rust 1.75+ (install from https://rustup.rs/)
- PostgreSQL access via SSH tunnel to `botserver`
- Environment variables configured in `.env`

## Database Setup

This server shares the PostgreSQL database with the Discord bot (grimkeeper).
**You must have an SSH tunnel running to connect to the database.**

### Start the SSH Tunnel

In a separate terminal, run:

```bash
./tunnel.sh
```

Or manually:

```bash
ssh -N -L 5432:localhost:5432 botserver
```

Leave this running while the server is active.

## Running the Server

### Development Mode

```bash
cargo run
```

### Production Mode

```bash
cargo build --release
./target/release/grimlive-server
```

## Configuration

Copy `.env.example` to `.env` and configure:

- `DATABASE_URL` - PostgreSQL connection (via localhost:5432 tunnel)
- `DISCORD_CLIENT_ID` - Discord OAuth client ID
- `DISCORD_CLIENT_SECRET` - Discord OAuth secret
- `PORT` - Server port (default: 8001)

## API Endpoints

### Public Endpoints

- `GET /health` - Health check
- `GET /auth/discord` - Discord OAuth redirect
- `GET /auth/discord/callback` - Discord OAuth callback
- `GET /` - WebSocket connection endpoint

### Protected Endpoints (require X-API-Key header)

- `GET /api/v1/games` - List games with pagination
- `GET /api/v1/games/:id` - Get game details
- `GET /api/v1/stats/summary` - Overall statistics
- `GET /api/v1/players/:discord_id/stats` - Player statistics
- `GET /api/v1/scripts/:script_name/stats` - Script statistics

## WebSocket Protocol

Connect to `ws://localhost:8001/` and send JSON messages:

### Connect to Session

```json
{
  "type": "connect",
  "sessionId": "session-code",
  "discordId": "123456789",
  "username": "Player"
}
```

### Game State Update

```json
{
  "type": "gameState",
  "players": [...],
  "night": 1,
  "phase": "night"
}
```

Messages are automatically broadcast to all clients in the same session.

## Development

### Check Code

```bash
cargo check
```

### Run Tests

```bash
cargo test
```

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy
```

## Architecture

```
src/
├── config.rs       - Configuration management
├── database.rs     - Database connection pool
├── error.rs        - Error types and handling
├── main.rs         - Server setup and routing
├── models.rs       - Database models
├── state.rs        - Application state
├── handlers/       - HTTP/WebSocket handlers
│   ├── api/       - REST API endpoints
│   ├── auth.rs    - Discord OAuth
│   ├── health.rs  - Health check
│   └── websocket.rs - WebSocket handler
├── middleware/     - Middleware (auth, etc.)
├── services/       - Business logic
│   ├── game.rs    - Game queries
│   ├── session.rs - Session management
│   └── rate_limit.rs - Rate limiting
└── utils/          - Utilities
    └── validation.rs - Input validation
```

## Troubleshooting

### "relation does not exist" error

Make sure the SSH tunnel is running and connected to botserver.

### "connection refused" error

1. Check that the SSH tunnel is active: `ps aux | grep "ssh.*5432"`
2. Verify you can connect manually: `psql -h localhost -U grimkeeper -d grimkeeper_db`

### Migration errors

The database schema is managed by the Discord bot. This server's migrations only add indexes.
If you get migration errors, try:

```bash
# On botserver
sudo -u postgres psql -d grimkeeper_db -c "DELETE FROM _sqlx_migrations WHERE version = 20240101000001;"
```

Then restart the server.
