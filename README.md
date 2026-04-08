# claude-usage

Monitor your Claude usage over time. A local agent periodically collects usage data, stores it in PostgreSQL, a Rust backend serves it via API, and a React frontend renders dashboards.

## Features

- **Usage gauges** - 5-hour, 7-day (all models), and 7-day (Sonnet only) utilization
- **Historical charts** - View usage trends over 24h, 7d, or 30d
- **Auto-refresh** - Dashboard updates every 10 minutes
- **Dark mode** - Follows system preference
- **Reset timers** - Shows when each usage bucket resets

## Auth Flow

SSH key-based authentication using the system's OpenSSH server:

1. User opens the website and sees an SSH command: `ssh claude-auth@host auth=<token>`
2. User runs the command in their terminal
3. OpenSSH verifies the user's public key against `authorized_keys`
4. The forced command (`auth-handler`) calls the backend to mark the challenge as authenticated
5. The browser receives a session cookie and shows the dashboard

---

## Server Setup (Ubuntu as example)

Deploys the backend API, frontend, and SSH auth on a server.

### Prerequisites

- Rust (stable)
- Node.js (18+)
- PostgreSQL
- Nginx

### Step 1: Clone Repository

```bash
cd ~
git clone https://github.com/Doublefire-Chen/claude-usage.git
cd claude-usage
```

### Step 2: Database Setup

```bash
psql
```
```
CREATE USER claude_usage WITH PASSWORD 'strong-password';
CREATE DATABASE claude_usage OWNER claude_usage;
GRANT ALL PRIVILEGES ON DATABASE claude_usage TO claude_usage;
```

Use [./backend/db_schema/schema.sql](backend/db_schema/schema.sql) to create the database schema.

### Step 3: Build Application

```bash
# Build backend (produces server and auth-handler binaries)
cd backend
cp .env.example .env  # edit as needed
cargo build --release
sudo mkdir -p /opt/claude-usage
sudo cp target/release/server /opt/claude-usage/
sudo cp target/release/auth-handler /opt/claude-usage/

# Configure backend .env
# Set FRONTEND_URL to your frontend domain (e.g. https://claude-usage-example.com)
sudo cp .env /opt/claude-usage/.env
sudo vim /opt/claude-usage/.env

# Build frontend
cd ../frontend
cp .env.example .env
# Set VITE_API_URL to your backend API domain (e.g. https://api.claude-usage-example.com)
npm install
npm run build
sudo mkdir -p /var/www/claude-usage
sudo cp -r dist/* /var/www/claude-usage/
```

### Step 4: SSH Auth Setup

Create a dedicated system user for SSH authentication:

```bash
sudo useradd -r -s /bin/bash -m claude-auth
sudo mkdir -p ~claude-auth/.ssh
sudo chmod 700 ~claude-auth/.ssh
```

Add authorized SSH public keys (admin-managed only):

```bash
# For each user, add a line to authorized_keys (replace user@host with the display name):
echo 'command="/opt/claude-usage/auth-handler --user=user@host",no-port-forwarding,no-X11-forwarding,no-agent-forwarding,no-pty ssh-ed25519 AAAA... user@host' \
  | sudo tee -a ~claude-auth/.ssh/authorized_keys

sudo chown -R claude-auth: ~claude-auth/.ssh
sudo chmod 600 ~claude-auth/.ssh/authorized_keys
```

### Step 5: Create Systemd Service

```bash
sudo cp configs/claude-usage.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable claude-usage
sudo systemctl start claude-usage
sudo systemctl status claude-usage
```

### Step 6: Configure Nginx

```bash
sudo cp configs/nginx.conf /etc/nginx/sites-available/claude-usage
sudo vim /etc/nginx/sites-available/claude-usage

sudo ln -s /etc/nginx/sites-available/claude-usage /etc/nginx/sites-enabled/
sudo nginx -t
sudo certbot --nginx -d claude-usage-example.com -d api.claude-usage-example.com
sudo systemctl reload nginx
```

### Step 7: Access Application

Access the application at `https://claude-usage-example.com`.

---

## Agent Setup (Linux / macOS)

The agent runs on the machine where Claude Code is active. It reads the OAuth token from Claude Code's credentials (macOS Keychain or `~/.claude/.credentials.json` on Linux) and periodically polls the usage API, writing snapshots to PostgreSQL.

### Prerequisites

- Rust (stable)
- Network access to the PostgreSQL database

### Step 1: Clone and Build

```bash
cd ~
git clone https://github.com/Doublefire-Chen/claude-usage.git
cd claude-usage/backend
cargo build --release -p agent
```

### Step 2: Configure and Run

Create a `.env` file next to the binary:

```bash
cp .env.example target/release/.env
vim target/release/.env  # set DATABASE_URL to your remote database
```

```bash
./target/release/agent
```

The agent will poll immediately on start, then every `POLL_INTERVAL_SECS` seconds. It connects directly to PostgreSQL, so ensure the database is accessible from this machine.
