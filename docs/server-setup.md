# Server Setup — One-Time Configuration

Prepares a fresh Linux server (Ubuntu/Debian) for KARS deployment.

---

## 1. Create Service User

```bash
sudo useradd -r -m -s /usr/sbin/nologin kars
```

## 2. Create Application Directory

```bash
sudo mkdir -p /opt/kars
sudo mkdir -p /opt/kars/data
sudo chown -R kars:kars /opt/kars
```

If you use `DATABASE_MODE=local`, KARS writes SQLite data under `/opt/kars/data` (default: `/opt/kars/data/kars.db`).

## 3. Create Environment File

```bash
sudo tee /opt/kars/.env > /dev/null << 'EOF'
DATABASE_MODE=turso
TURSO_DATABASE_URL=libsql://your-db.turso.io
TURSO_AUTH_TOKEN=your-turso-token
PORT=3001
TMDB_API_KEY=your-tmdb-api-key
EOF

sudo chmod 600 /opt/kars/.env
sudo chown kars:kars /opt/kars/.env
```

`/opt/kars/.env` is required by the systemd unit below (`EnvironmentFile=/opt/kars/.env`), so ensure this file exists before starting the service.

## 4. Install systemd Service

Copy the provided service file:

```bash
sudo cp deploy/kars.service /etc/systemd/system/kars.service
sudo systemctl daemon-reload
sudo systemctl enable kars
```

Or create it directly:

```bash
sudo tee /etc/systemd/system/kars.service > /dev/null << 'EOF'
[Unit]
Description=KARS Media Archive
After=network.target

[Service]
Type=simple
User=kars
Group=kars
WorkingDirectory=/opt/kars
ExecStart=/opt/kars/kars
EnvironmentFile=/opt/kars/.env
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable kars
```

## 5. Configure Firewall

```bash
# UFW
sudo ufw allow 3001/tcp

# Or iptables
sudo iptables -A INPUT -p tcp --dport 3001 -j ACCEPT
```

## 6. SSH Key for GitHub Actions

The deploy user needs SSH access for automated deployment:

```bash
# Add the deploy public key to authorized_keys
mkdir -p ~/.ssh
nano ~/.ssh/authorized_keys
# Paste the public key corresponding to the SSH_KEY GitHub secret

# Allow the deploy user to restart the service without password
echo "<SSH_USER> ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart kars, /usr/bin/systemctl is-active kars" \
  | sudo tee /etc/sudoers.d/kars-deploy
```

## 7. Verify Setup

After the first GitHub Actions deployment:

```bash
# Check service status
sudo systemctl status kars

# Check logs
sudo journalctl -u kars -f

# Test API
curl http://localhost:3001/api/stats
```

## 8. Optional: Reverse Proxy with HTTPS

If you are using Cloudflare Zero Trust/Access with Cloudflare Tunnel (see [auth.md](auth.md)), you can skip this step.

For a custom domain with automatic TLS, install Caddy:

```bash
sudo apt install -y caddy

sudo tee /etc/caddy/Caddyfile > /dev/null << 'EOF'
your-domain.com {
    reverse_proxy localhost:3001
}
EOF

sudo systemctl restart caddy
```

Caddy auto-provisions Let's Encrypt certificates.
