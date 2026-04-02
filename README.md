# claude-usage

Monitor your Claude usage over time. A local agent periodically collects usage data, stores it in PostgreSQL, a Rust backend serves it via API, and a React frontend renders dashboards.

## Features

- **Usage gauges** - 5-hour, 7-day (all models), and 7-day (Sonnet only) utilization
- **Historical charts** - View usage trends over 24h, 7d, or 30d
- **Auto-refresh** - Dashboard updates every 5 minutes
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

## How to Install (Ubuntu as example)

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
# Build backend (produces three binaries: server, agent, auth-handler)
cd backend
cp .env.example .env  # edit as needed
cargo build --release
sudo mkdir -p /opt/claude-usage
sudo cp target/release/server /opt/claude-usage/
sudo cp target/release/agent /opt/claude-usage/
sudo cp target/release/auth-handler /opt/claude-usage/

# Configure backend .env
sudo cp .env /opt/claude-usage/.env
sudo vim /opt/claude-usage/.env

# Build frontend
cd ../frontend
cp .env.example .env  # edit as needed
npm install
npm run build
sudo mkdir -p /var/www/claude-usage
sudo cp -r dist/* /var/www/claude-usage/
```

### Step 4: SSH Auth Setup

Create a dedicated system user for SSH authentication:

```bash
sudo useradd -r -s /usr/sbin/nologin -m claude-auth
sudo mkdir -p ~claude-auth/.ssh
sudo chmod 700 ~claude-auth/.ssh
```

Add authorized SSH public keys (admin-managed only):

```bash
# For each user, add a line to authorized_keys:
echo 'command="/opt/claude-usage/auth-handler",no-port-forwarding,no-X11-forwarding,no-agent-forwarding,no-pty ssh-ed25519 AAAA... user@host' \
  | sudo tee -a ~claude-auth/.ssh/authorized_keys

sudo chown -R claude-auth: ~claude-auth/.ssh
sudo chmod 600 ~claude-auth/.ssh/authorized_keys
```

### Step 5: Configure Nginx

```bash
sudo cp configs/nginx.conf /etc/nginx/sites-available/claude-usage
sudo vim /etc/nginx/sites-available/claude-usage

sudo ln -s /etc/nginx/sites-available/claude-usage /etc/nginx/sites-enabled/
sudo nginx -t
sudo certbot --nginx -d claude-usage-example.com
sudo systemctl reload nginx
```

### Step 6: Access Application

Access the application at `https://claude-usage-example.com`.
