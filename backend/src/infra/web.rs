use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::core::api_types::{ApiMediaItem, ApiStats, ApiExploreResult};
use crate::core::search::{MediaSearchType, SearchProvider};
use crate::infra::database::Database;
use crate::infra::anilist::AniListClient;
use crate::infra::tmdb::TmdbClient;
use crate::infra::openlibrary::OpenLibraryClient;
use crate::infra::mangadex::MangaDexClient;

// ── App state ────────────────────────────────────────────────

pub struct WebState {
    pub db: Database,
}

type SharedState = Arc<Mutex<WebState>>;
type Searchers = Arc<Vec<Box<dyn SearchProvider + Send + Sync>>>;

/// Combined state passed to handlers via axum State extractor.
#[derive(Clone)]
struct AppState {
    db_state: SharedState,
    searchers: Searchers,
}

// ── Server bootstrap ─────────────────────────────────────────

/// Build search providers. Must be called **outside** an async context because
/// reqwest::blocking::Client spawns its own Tokio runtime internally.
pub fn build_searchers() -> Vec<Box<dyn SearchProvider + Send + Sync>> {
    let mut searchers: Vec<Box<dyn SearchProvider + Send + Sync>> = vec![
        Box::new(AniListClient::new()),
        Box::new(MangaDexClient::new()),
        Box::new(OpenLibraryClient::new()),
    ];
    if let Some(tmdb) = TmdbClient::from_env() {
        searchers.push(Box::new(tmdb));
    } else {
        eprintln!("Note: TMDB_API_KEY not set — movie/series search disabled.");
    }
    searchers
}

pub async fn start_server(
    db: Database,
    port: u16,
    searchers: Vec<Box<dyn SearchProvider + Send + Sync>>,
) {
    let app_state = AppState {
        db_state: Arc::new(Mutex::new(WebState { db })),
        searchers: Arc::new(searchers),
    };

    let api = Router::new()
        .route("/api/items", get(list_items).post(create_item))
        .route(
            "/api/items/{id}",
            get(get_item).put(update_item).delete(delete_item),
        )
        .route("/api/search", get(search_items))
        .route("/api/explore", get(explore_items))
        .route("/api/stats", get(get_stats))
        .with_state(app_state);

    // Add CORS for development (Next.js on :3000 → Rust on :3001)
    let app = api
        .fallback(static_handler)
        .layer(tower_http::cors::CorsLayer::permissive());

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    println!("╔══════════════════════════════════════════╗");
    println!("║      KARS — Media Archive System         ║");
    println!("║                                          ║");
    println!("║  Web UI:  http://localhost:{port:<5}         ║");
    println!("║  API:     http://localhost:{port:<5}/api     ║");
    println!("╚══════════════════════════════════════════╝");

    axum::serve(listener, app).await.unwrap();
}

// ── GET /api/items ───────────────────────────────────────────

async fn list_items(State(state): State<AppState>) -> Response {
    let st = state.db_state.lock().await;
    match st.db.load_all().await {
        Ok(items) => {
            let api: Vec<ApiMediaItem> = items.iter().map(ApiMediaItem::from).collect();
            Json(api).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── POST /api/items ──────────────────────────────────────────

async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<ApiMediaItem>,
) -> Response {
    let item = match payload.into_media_item() {
        Ok(i) => i,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let st = state.db_state.lock().await;
    match st.db.upsert_item(&item).await {
        Ok(()) => {
            let api = ApiMediaItem::from(&item);
            (StatusCode::CREATED, Json(api)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── GET /api/items/:id ───────────────────────────────────────

async fn get_item(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UUID").into_response(),
    };

    let st = state.db_state.lock().await;
    match st.db.get_item(uuid).await {
        Ok(Some(item)) => Json(ApiMediaItem::from(&item)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── PUT /api/items/:id ───────────────────────────────────────

async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut payload): Json<ApiMediaItem>,
) -> Response {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UUID").into_response(),
    };

    // Ensure the ID in the path matches the body
    payload.id = uuid.to_string();

    let item = match payload.into_media_item() {
        Ok(i) => i,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let st = state.db_state.lock().await;
    match st.db.upsert_item(&item).await {
        Ok(()) => {
            let api = ApiMediaItem::from(&item);
            Json(api).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── DELETE /api/items/:id ────────────────────────────────────

async fn delete_item(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UUID").into_response(),
    };

    let st = state.db_state.lock().await;
    match st.db.delete_item(uuid).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── GET /api/search?q=... ────────────────────────────────────

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

async fn search_items(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Response {
    let query = params.q.unwrap_or_default();
    if query.is_empty() {
        return Json(Vec::<ApiMediaItem>::new()).into_response();
    }

    let st = state.db_state.lock().await;
    match st.db.search_items(&query).await {
        Ok(items) => {
            let api: Vec<ApiMediaItem> = items.iter().map(ApiMediaItem::from).collect();
            Json(api).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── GET /api/stats ───────────────────────────────────────────

async fn get_stats(State(state): State<AppState>) -> Response {
    let st = state.db_state.lock().await;
    match st.db.load_all().await {
        Ok(items) => {
            let api_items: Vec<ApiMediaItem> = items.iter().map(ApiMediaItem::from).collect();
            let stats = ApiStats::from_items(&api_items);
            Json(stats).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── GET /api/explore?q=...&type=anime|movie|manga|book ───────

#[derive(Deserialize)]
struct ExploreQuery {
    q: Option<String>,
    #[serde(rename = "type")]
    media_type: Option<String>,
}

async fn explore_items(
    State(state): State<AppState>,
    Query(params): Query<ExploreQuery>,
) -> Response {
    let query = params.q.unwrap_or_default();
    if query.len() < 2 {
        return Json(Vec::<ApiExploreResult>::new()).into_response();
    }

    let search_type = match params.media_type.as_deref() {
        Some("anime") => MediaSearchType::Anime,
        Some("movie") => MediaSearchType::Movie,
        Some("series") => MediaSearchType::Series,
        Some("manga") => MediaSearchType::Manga,
        Some("book") => MediaSearchType::Book,
        Some("light_novel") => MediaSearchType::LightNovel,
        _ => MediaSearchType::Anime, // default
    };

    // Run blocking search providers on a dedicated thread so
    // reqwest::blocking doesn't panic inside the async runtime.
    let searchers = Arc::clone(&state.searchers);
    let q = query.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut all_results = Vec::new();
        for searcher in searchers.iter() {
            if searcher.supported_types().contains(&search_type) {
                match searcher.search(&q, search_type) {
                    Ok(results) => {
                        all_results.extend(
                            results.iter().map(ApiExploreResult::from_search_result)
                        );
                    }
                    Err(e) => {
                        eprintln!("Search provider {} error: {e}", searcher.name());
                    }
                }
            }
        }
        all_results
    })
    .await;

    match result {
        Ok(items) => Json(items).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ── Static file serving ──────────────────────────────────────

#[cfg(feature = "embed-frontend")]
mod embedded {
    use rust_embed::Embed;

    #[derive(Embed)]
    #[folder = "../frontend/out/"]
    pub struct Assets;
}

async fn static_handler(uri: axum::http::Uri) -> Response {
    #[cfg(feature = "embed-frontend")]
    {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };

        if let Some(content) = embedded::Assets::get(path) {
            let mime = guess_mime(path);
            return (
                StatusCode::OK,
                [("content-type", mime)],
                content.data.to_vec(),
            )
                .into_response();
        }

        // SPA fallback — serve index.html for unmatched routes
        if let Some(content) = embedded::Assets::get("index.html") {
            return (
                StatusCode::OK,
                [("content-type", "text/html; charset=utf-8")],
                content.data.to_vec(),
            )
                .into_response();
        }

        (StatusCode::NOT_FOUND, "Not found").into_response()
    }

    #[cfg(not(feature = "embed-frontend"))]
    {
        let _ = uri;
        axum::response::Html(
            r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>KARS</title></head>
<body style="font-family:system-ui;background:#0f1117;color:#e5e7eb;display:flex;align-items:center;justify-content:center;height:100vh;margin:0">
<div style="text-align:center">
<h1>KARS API Server</h1>
<p>Frontend is not embedded. For development, run:</p>
<pre style="background:#1a1d27;padding:1rem;border-radius:8px;text-align:left">cd frontend
pnpm dev</pre>
<p style="margin-top:1rem;color:#6b7280">API is available at <code>/api/items</code></p>
</div>
</body></html>"#,
        )
        .into_response()
    }
}

#[cfg(feature = "embed-frontend")]
fn guess_mime(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}
