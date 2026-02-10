mod core;
mod infra;

use infra::database::{Database, SqlStorage};
use infra::terminal::TerminalInput;
use infra::anilist::AniListClient;
use infra::tmdb::TmdbClient;
use infra::openlibrary::OpenLibraryClient;
use infra::mangadex::MangaDexClient;
use crate::core::search::SearchProvider;

fn main() {
    // Load .env (silently ignore if missing — production uses real env vars)
    let _ = dotenvy::dotenv();

    let args: Vec<String> = std::env::args().collect();
    let cli_mode = args.iter().any(|a| a == "--cli");

    if cli_mode {
        run_cli();
    } else {
        run_web();
    }
}

/// Classic terminal UI — kept as emergency / power-user access.
fn run_cli() {
    let db_mode = std::env::var("DATABASE_MODE").unwrap_or_else(|_| "local".into());

    let storage: SqlStorage = match db_mode.as_str() {
        "turso" => {
            let url = std::env::var("TURSO_DATABASE_URL")
                .expect("TURSO_DATABASE_URL must be set when DATABASE_MODE=turso");
            let token = std::env::var("TURSO_AUTH_TOKEN")
                .expect("TURSO_AUTH_TOKEN must be set when DATABASE_MODE=turso");
            SqlStorage::turso(&url, &token).expect("Failed to connect to Turso")
        }
        _ => {
            let path = std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "data/kars.db".into());
            SqlStorage::local(&path).expect("Failed to open local database")
        }
    };

    let input = TerminalInput;

    let mut searchers: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(AniListClient::new()),
        Box::new(MangaDexClient::new()),
        Box::new(OpenLibraryClient::new()),
    ];

    if let Some(tmdb) = TmdbClient::from_env() {
        searchers.push(Box::new(tmdb));
    } else {
        eprintln!("Note: TMDB_API_KEY not set — movie/series search disabled.");
    }

    let mut app = match core::app::App::new(storage, input, searchers) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to initialize: {e}");
            std::process::exit(1);
        }
    };

    app.run();
}

/// Web server mode — default.  Serves the REST API (and embedded frontend
/// when compiled with --features embed-frontend).
fn run_web() {
    // Build search providers BEFORE entering the async runtime.
    // reqwest::blocking::Client creates its own mini-runtime;
    // constructing/dropping it inside block_on causes a panic.
    let searchers = infra::web::build_searchers();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create async runtime");

    rt.block_on(async {
        let db_mode = std::env::var("DATABASE_MODE").unwrap_or_else(|_| "local".into());

        let db = match db_mode.as_str() {
            "turso" => {
                let url = std::env::var("TURSO_DATABASE_URL")
                    .expect("TURSO_DATABASE_URL must be set when DATABASE_MODE=turso");
                let token = std::env::var("TURSO_AUTH_TOKEN")
                    .expect("TURSO_AUTH_TOKEN must be set when DATABASE_MODE=turso");
                Database::turso(&url, &token)
                    .await
                    .expect("Failed to connect to Turso")
            }
            _ => {
                let path = std::env::var("DATABASE_PATH")
                    .unwrap_or_else(|_| "data/kars.db".into());
                Database::local(&path)
                    .await
                    .expect("Failed to open local database")
            }
        };

        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);

        infra::web::start_server(db, port, searchers).await;
    });
}
