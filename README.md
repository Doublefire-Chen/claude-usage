# claude-usage

Monitor your Claude usage over time.

## Tech Stack

- **Backend**: Rust (Axum)
- **Frontend**: Vite + React + TypeScript
- **Database**: PostgreSQL
- **Charts**: Recharts

## Architecture

```
Local agent (polls Claude API) → PostgreSQL → Backend API → React frontend
```

The **agent** runs as a daemon on a machine where Claude Code is active. It reads the OAuth token from Claude Code's credentials (macOS Keychain or `~/.claude/.credentials.json` on Linux) and periodically calls the usage endpoint, storing snapshots in PostgreSQL.

The **server** serves the usage data via a REST API. The **frontend** displays current usage gauges and historical charts.

## Auth Flow

SSH key-based authentication using the system's OpenSSH server:

1. User opens the website and sees an SSH command: `ssh claude-auth@host auth=<token>`
2. User runs the command in their terminal
3. OpenSSH verifies the user's public key against `authorized_keys`
4. The forced command (`auth-handler`) calls the backend to mark the challenge as authenticated
5. The browser receives a session cookie and shows the dashboard

All authenticated users share the same read-only view of usage data.

## Prerequisites

- Rust (stable)
- Node.js (18+)
- PostgreSQL

## Setup

### 1. Clone Repository

```bash
cd ~
git clone https://github.com/Doublefire-Chen/claude-usage.git
cd claude-usage
```

### 2. Database

```bash
psql
```
```
CREATE USER claude_usage WITH PASSWORD 'strong-password';
CREATE DATABASE claude_usage OWNER claude_usage;
GRANT ALL PRIVILEGES ON DATABASE claude_usage TO claude_usage;
```

Tables are auto-created on first startup by the backend.

### 3. Install frontend dependencies

```bash
cd frontend && npm install
```

### 4. SSH auth (production)

Create a dedicated system user for SSH authentication:

```bash
sudo useradd -r -s /usr/sbin/nologin -m claude-auth
sudo mkdir -p ~claude-auth/.ssh
sudo chmod 700 ~claude-auth/.ssh
```

Build the auth-handler binary:

```bash
cd backend
cargo build --release -p auth-handler
```

Add authorized SSH public keys (admin-managed only):

```bash
# For each user, add a line to authorized_keys:
echo 'command="/path/to/target/release/auth-handler",no-port-forwarding,no-X11-forwarding,no-agent-forwarding,no-pty ssh-ed25519 AAAA... user@host' \
  | sudo tee -a ~claude-auth/.ssh/authorized_keys

sudo chown -R claude-auth: ~claude-auth/.ssh
sudo chmod 600 ~claude-auth/.ssh/authorized_keys
```

## Running

### Development

```bash
# Terminal 1: Backend API server
cd backend && DATABASE_URL=postgres://localhost/claude_usage cargo run -p server

# Terminal 2: Usage collection agent
cd backend && DATABASE_URL=postgres://localhost/claude_usage cargo run -p agent

# Terminal 3: Frontend dev server (proxies /api to localhost:3000)
cd frontend && npm run dev
```

### Environment Variables

**Backend & Agent** (`backend/.env`):

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | (required) | PostgreSQL connection string |
| `LISTEN_ADDR` | `0.0.0.0:3000` | Server listen address |
| `SSH_HOST` | `localhost` | SSH host shown in login command |
| `SSH_PORT` | `22` | SSH port shown in login command |
| `POLL_INTERVAL_SECS` | `300` | Agent polling interval (seconds) |
| `RUST_LOG` | `info` | Log level |
| `AUTH_SERVER_URL` | `http://localhost:3000` | Backend URL for auth-handler |

**Frontend** (`frontend/.env`):

| Variable | Default | Description |
|---|---|---|
| `VITE_API_BASE_URL` | `/api` | Backend API base URL |

### Production Build & Deploy

```bash
# Build backend (produces two binaries: server and agent)
cd backend
cargo build --release
sudo mkdir -p /opt/claude-usage
sudo cp target/release/server /opt/claude-usage/
sudo cp target/release/agent /opt/claude-usage/
sudo cp target/release/auth-handler /opt/claude-usage/

# Configure backend & agent .env
sudo cp .env.example /opt/claude-usage/.env
sudo vim /opt/claude-usage/.env

# Build frontend
cd ../frontend
cp .env.example .env
vim .env
npm install
npm run build
sudo mkdir -p /var/www/claude-usage
sudo cp -r dist/* /var/www/claude-usage/
```

### Configure Nginx

```bash
sudo vim /etc/nginx/sites-available/claude-usage
```

Example configuration:

```nginx
server {
    listen 80;
    server_name claude-usage-example.com;

    # Frontend
    root /var/www/claude-usage;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # Proxy API requests to backend
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /health {
        proxy_pass http://127.0.0.1:3000;
    }
}
```

Enable the site and obtain SSL:

```bash
sudo ln -s /etc/nginx/sites-available/claude-usage /etc/nginx/sites-enabled/
sudo nginx -t
sudo certbot --nginx -d claude-usage-example.com
sudo systemctl reload nginx
```

## API Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/health` | No | Health check |
| `POST` | `/api/auth/challenge` | No | Create auth challenge |
| `GET` | `/api/auth/status?token=TOKEN` | No | Poll challenge status |
| `POST` | `/api/auth/logout` | No | Delete session |
| `GET` | `/api/usage/current` | Session | Latest usage snapshot |
| `GET` | `/api/usage/history?from=&to=` | Session | Historical usage data |
| `POST` | `/api/internal/auth/verify` | Localhost | Called by auth-handler |

## Project Structure

```
claude-usage/
├── backend/
│   ├── crates/
│   │   ├── agent/          # Usage data collection daemon
│   │   ├── auth-handler/   # SSH forced command binary
│   │   ├── server/         # Axum API server
│   │   └── shared/         # Shared types, DB queries, auth helpers
│   ├── Cargo.toml
│   └── Cargo.lock
└── frontend/               # Vite + React app
```

## Features

- **Usage gauges**: 5-hour, 7-day (all models), and 7-day (Sonnet only) utilization
- **Historical charts**: View usage trends over 24h, 7d, or 30d
- **Auto-refresh**: Dashboard updates every 5 minutes
- **Dark mode**: Follows system preference
- **Reset timers**: Shows when each usage bucket resets
