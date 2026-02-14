# KARS - Media Tracker

A self-hosted media tracking application for movies, TV series, anime, manga, and books. Single binary deployment with an embedded web dashboard.

![Build & Deploy](https://github.com/recregt/kars/actions/workflows/deploy.yml/badge.svg)

## Features

- **Track** movies, TV series, anime, manga, light novels, and books
- **Search** external APIs: AniList, TMDB, MangaDex, Open Library
- **Dashboard** with stats cards, sortable data table, and quick filters
- **CRUD** — add, edit, delete items from your library
- **Single binary** — frontend embedded, zero runtime dependencies
- **Dual database** — local SQLite or remote [Turso](https://turso.tech)

## Screenshots

[![KARS Dashboard](assets/dashboard.webp)](assets/dashboard.webp)

[![KARS Explore](assets/explore.webp)](assets/explore.webp)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum, libsql |
| Frontend | Next.js, React, shadcn/ui, Tailwind CSS |
| Database | SQLite / Turso |
| CI/CD | GitHub Actions |

## Quick Start (Development)

```bash
# 1. Clone
git clone https://github.com/recregt/kars.git
cd kars

# 2. Configure
cp .env.example .env
# Edit .env with your settings

# 3. Backend (terminal 1)
cargo run -p kars -- --web

# 4. Frontend (terminal 2)
cd frontend
pnpm install
pnpm dev
```

- Frontend: http://localhost:3000
- API: http://localhost:3001/api

## Production

See [docs/production.md](docs/production.md) for build instructions and [docs/server-setup.md](docs/server-setup.md) for server configuration.

```bash
# Build
cd frontend && pnpm install --frozen-lockfile && pnpm build && cd ..
cargo build -p kars --release --features embed-frontend

# Run
./target/release/kars
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/items` | List all items |
| `POST` | `/api/items` | Create item |
| `GET` | `/api/items/:id` | Get item by ID |
| `PUT` | `/api/items/:id` | Update item |
| `DELETE` | `/api/items/:id` | Delete item |
| `GET` | `/api/search?q=` | Search library |
| `GET` | `/api/explore?q=&type=` | Search external APIs |
| `GET` | `/api/stats` | Library statistics |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_MODE` | `local` | `local` or `turso` |
| `DATABASE_PATH` | `data/kars.db` | SQLite path (local mode) |
| `TURSO_DATABASE_URL` | — | Turso connection URL |
| `TURSO_AUTH_TOKEN` | — | Turso auth token |
| `PORT` | `3001` | Server port |
| `TMDB_API_KEY` | — | TMDB API key (optional) |

## License

[MIT](LICENSE)
