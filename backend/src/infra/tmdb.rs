use crate::core::models::{MediaItemType, Progress, WatchStatus};
use crate::core::search::{MediaSearchType, SearchError, SearchProvider, SearchResult};
use reqwest::blocking::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://api.themoviedb.org/3";
const POSTER_BASE: &str = "https://image.tmdb.org/t/p/w500";

// ── Response types ───────────────────────────────────────────────

#[derive(Deserialize)]
struct PagedResponse<T> {
    results: Vec<T>,
}

#[derive(Deserialize)]
struct MovieResult {
    id: u32,
    title: String,
    vote_average: Option<f64>,
    poster_path: Option<String>,
    release_date: Option<String>,
}

#[derive(Deserialize)]
struct TvResult {
    id: u32,
    name: String,
    vote_average: Option<f64>,
    poster_path: Option<String>,
    first_air_date: Option<String>,
}

// ── Client ───────────────────────────────────────────────────────

pub struct TmdbClient {
    client: Client,
    api_key: String,
}

impl TmdbClient {
    /// Reads the TMDB Bearer token from TMDB_API_KEY env var.
    /// Returns None if the env var is not set, so the app can still run without it.
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("TMDB_API_KEY").ok()?;
        if key.is_empty() {
            return None;
        }
        Some(Self {
            client: Client::new(),
            api_key: key,
        })
    }

    fn get(&self, path: &str, query: &str) -> Result<reqwest::blocking::Response, SearchError> {
        let url = format!("{BASE_URL}{path}");
        self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("query", query),
                ("include_adult", "false"),
                ("language", "en-US"),
                ("page", "1"),
            ])
            .send()
            .map_err(|e| SearchError::Network(e.to_string()))
    }

    fn search_movies(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        let resp = self.get("/search/movie", query)?;
        let page: PagedResponse<MovieResult> = resp
            .json()
            .map_err(|e| SearchError::Parse(e.to_string()))?;

        let results = page
            .results
            .into_iter()
            .take(10)
            .map(|m| {
                let year = m
                    .release_date
                    .as_deref()
                    .and_then(|d| d.get(..4))
                    .unwrap_or("?");

                SearchResult {
                    title: m.title,
                    media_type: MediaItemType::Movie(WatchStatus::PlanToWatch),
                    global_score: vote_to_score(m.vote_average),
                    external_id: Some(m.id),
                    poster_url: m.poster_path.map(|p| format!("{POSTER_BASE}{p}")),
                    source: "tmdb",
                    format_label: format!("Movie ({year})"),
                }
            })
            .collect();

        Ok(results)
    }

    fn search_tv(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        let resp = self.get("/search/tv", query)?;
        let page: PagedResponse<TvResult> = resp
            .json()
            .map_err(|e| SearchError::Parse(e.to_string()))?;

        let results = page
            .results
            .into_iter()
            .take(10)
            .map(|t| {
                let year = t
                    .first_air_date
                    .as_deref()
                    .and_then(|d| d.get(..4))
                    .unwrap_or("?");

                SearchResult {
                    title: t.name,
                    media_type: MediaItemType::Series(
                        Progress { current: 0, total: None },
                        WatchStatus::PlanToWatch,
                    ),
                    global_score: vote_to_score(t.vote_average),
                    external_id: Some(t.id),
                    poster_url: t.poster_path.map(|p| format!("{POSTER_BASE}{p}")),
                    source: "tmdb",
                    format_label: format!("TV Series ({year})"),
                }
            })
            .collect();

        Ok(results)
    }
}

/// TMDB vote_average: 0.0-10.0 → our global_score: 0-100 (u8)
fn vote_to_score(vote: Option<f64>) -> Option<u8> {
    vote.filter(|&v| v > 0.0)
        .map(|v| (v.clamp(0.0, 10.0) * 10.0).round() as u8)
}

impl SearchProvider for TmdbClient {
    fn name(&self) -> &str {
        "TMDB"
    }

    fn supported_types(&self) -> &[MediaSearchType] {
        &[MediaSearchType::Movie, MediaSearchType::Series]
    }

    fn search(
        &self,
        query: &str,
        media_type: MediaSearchType,
    ) -> Result<Vec<SearchResult>, SearchError> {
        match media_type {
            MediaSearchType::Movie => self.search_movies(query),
            MediaSearchType::Series => self.search_tv(query),
            _ => Ok(Vec::new()),
        }
    }
}
