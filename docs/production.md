# KARS — Production Build & Deployment

Step-by-step guide for building and deploying KARS to production.

---

## Prerequisites

| Tool | Min Version | Purpose |
|------|-------------|---------|
| **Rust** | 1.85+ (edition 2024) | Backend compilation |
| **Node.js** | 20+ | Frontend build |
| **pnpm** | 9+ | Frontend package manager |

---

## 1. Frontend Static Export

The frontend is built as a Next.js static export to `frontend/out/`. This directory is then embedded into the Rust binary.

```bash
cd frontend
pnpm install --frozen-lockfile
pnpm build
```

> **Check:** Ensure `frontend/out/index.html` exists after build.

Build output in `frontend/out/`:
- `index.html` — Main page
- `_next/` — JavaScript, CSS, and other static assets
- `404.html` — Error page

---

## 2. Rust Release Binary

After the frontend build, compile the Rust binary with the `embed-frontend` feature flag. This embeds the contents of `frontend/out/` directly into the binary.

```bash
cd /project-root
cargo build -p kars --release --features embed-frontend
```

Output: `target/release/kars` (~16 MB, frontend included)

> **Important:** The frontend build MUST complete before the Rust build. If `frontend/out/` doesn't exist, compilation will fail.

---

## 3. Environment Variables

Configure via `.env` file or system environment variables:

### Required

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_MODE` | `local` | `local` (SQLite file) or `turso` (remote Turso DB) |
| `DATABASE_PATH` | `data/kars.db` | SQLite file path (when `DATABASE_MODE=local`) |
| `PORT` | `3001` | Web server port |

### Turso (Remote Database)

| Variable | Description |
|----------|-------------|
| `DATABASE_MODE` | Set to `turso` |
| `TURSO_DATABASE_URL` | `libsql://your-db.turso.io` |
| `TURSO_AUTH_TOKEN` | Turso authentication token |

### Optional API Keys

| Variable | Description |
|----------|-------------|
| `TMDB_API_KEY` | TMDB API key for movie/series search. If unset, movie/series search is disabled. |

### Example `.env`

```env
DATABASE_MODE=turso
TURSO_DATABASE_URL=libsql://your-db.turso.io
TURSO_AUTH_TOKEN=your-auth-token
PORT=3001
TMDB_API_KEY=your-tmdb-api-key
```

---

## 4. Running

### Single Binary (Recommended)

```bash
./kars
```

The server starts and displays:

```
╔══════════════════════════════════════════╗
║      KARS — Media Archive System         ║
║  Web UI:  http://localhost:3001          ║
║  API:     http://localhost:3001/api      ║
╚══════════════════════════════════════════╝
```

### CLI Mode (Emergency)

```bash
./kars --cli
```

---

## 5. Build Order Summary

```
┌─────────────────────────────────────┐
│  1. cd frontend && pnpm install     │
│  2. pnpm build                      │  → frontend/out/
│  3. cd .. (project root)            │
│  4. cargo build -p kars --release \ │
│       --features embed-frontend     │  → target/release/kars
│  5. Deploy binary to server         │
└─────────────────────────────────────┘
```

---

## 6. CI/CD — GitHub Actions

Deployment is automated via `.github/workflows/deploy.yml`. On every push to `main`:

1. Builds the frontend static export
2. Compiles the Rust release binary with embedded frontend
3. SCPs the binary to the production server
4. Restarts the systemd service

### Required GitHub Secrets

| Secret | Description |
|--------|-------------|
| `SSH_HOST` | Server IP or hostname |
| `SSH_USER` | SSH username |
| `SSH_KEY` | SSH private key (full PEM content) |

### Server-Side Secrets

These are stored in `/opt/kars/.env` on the server (NOT in GitHub):

- `DATABASE_MODE`, `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`
- `TMDB_API_KEY`
- `PORT`

---

## 7. Deployment Checklist

- [ ] Server prepared (see [server-setup.md](server-setup.md))
- [ ] `/opt/kars/.env` configured with Turso credentials
- [ ] GitHub secrets set: `SSH_HOST`, `SSH_USER`, `SSH_KEY`
- [ ] `git push origin main` triggers automatic deployment
- [ ] Verify: `curl http://<SERVER_IP>:3001/api/stats`

---

## 8. Architecture

```
┌──────────┐   git push   ┌──────────────┐
│ Developer │ ───────────▶ │ GitHub (pub) │
└──────────┘               └──────┬───────┘
                                  │ Actions CI/CD
                                  │ build → SCP → restart
                                  ▼
                           ┌──────────────┐
                           │ Google Cloud  │
                           │ /opt/kars/    │
                           │ systemd svc   │
                           └──────┬───────┘
                                  │ libsql
                                  ▼
                           ┌──────────────┐
                           │ Turso DB     │
                           └──────────────┘
```

---

## 9. One-Line Build Script

```bash
#!/bin/bash
set -e

echo "==> Frontend build..."
cd frontend
pnpm install --frozen-lockfile
pnpm build
cd ..

echo "==> Rust release build (frontend embedded)..."
cargo build -p kars --release --features embed-frontend

echo "==> Done!"
ls -lh target/release/kars
```

---

## 10. Notes

- **Portability:** The binary is fully self-contained — frontend files are embedded. Only the binary + `.env` are needed.
- **Database:** On first run, the `media_items` table is auto-created via migrations.
- **SPA Routing:** In embed mode, unknown URLs are served `index.html` (SPA fallback).
- **CORS:** Permissive CORS is enabled. Since the frontend is embedded (same origin), this is safe.
