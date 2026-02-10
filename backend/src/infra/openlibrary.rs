use crate::core::models::{MediaItemType, Progress, ReadStatus, ReadableKind};
use crate::core::search::{MediaSearchType, SearchError, SearchProvider, SearchResult};
use reqwest::blocking::Client;
use serde::Deserialize;

const SEARCH_URL: &str = "https://openlibrary.org/search.json";
const COVER_BASE: &str = "https://covers.openlibrary.org/b/id";

// ── Response types ───────────────────────────────────────────────

#[derive(Deserialize)]
struct SearchResponse {
    docs: Vec<BookDoc>,
}

#[derive(Deserialize)]
struct BookDoc {
    key: Option<String>,
    title: Option<String>,
    author_name: Option<Vec<String>>,
    first_publish_year: Option<u32>,
    cover_i: Option<u64>,
    number_of_pages_median: Option<u32>,
    ratings_average: Option<f64>,
}

// ── Client ───────────────────────────────────────────────────────

pub struct OpenLibraryClient {
    client: Client,
}

impl OpenLibraryClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl SearchProvider for OpenLibraryClient {
    fn name(&self) -> &str {
        "Open Library"
    }

    fn supported_types(&self) -> &[MediaSearchType] {
        &[MediaSearchType::Book]
    }

    fn search(
        &self,
        query: &str,
        media_type: MediaSearchType,
    ) -> Result<Vec<SearchResult>, SearchError> {
        if media_type != MediaSearchType::Book {
            return Ok(Vec::new());
        }

        let resp = self
            .client
            .get(SEARCH_URL)
            .query(&[
                ("q", query),
                ("fields", "key,title,author_name,first_publish_year,cover_i,number_of_pages_median,ratings_average"),
                ("limit", "10"),
            ])
            .send()
            .map_err(|e| SearchError::Network(e.to_string()))?;

        let data: SearchResponse = resp
            .json()
            .map_err(|e| SearchError::Parse(e.to_string()))?;

        let results = data
            .docs
            .into_iter()
            .filter_map(|doc| {
                let title = doc.title?;

                let author = doc
                    .author_name
                    .as_ref()
                    .and_then(|a| a.first())
                    .cloned()
                    .unwrap_or_else(|| "Unknown".into());

                let year = doc
                    .first_publish_year
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "?".into());

                let poster_url = doc
                    .cover_i
                    .map(|id| format!("{COVER_BASE}/{id}-M.jpg"));

                // ratings_average: 1.0-5.0 → our global_score: 0-100
                let global_score = doc.ratings_average.map(|r| {
                    ((r.clamp(0.0, 5.0) / 5.0) * 100.0).round() as u8
                });

                // Extract numeric work ID from key like "/works/OL27448W"
                let external_id = doc
                    .key
                    .as_deref()
                    .and_then(|k| k.trim_start_matches("/works/OL").trim_end_matches('W').parse::<u32>().ok());

                Some(SearchResult {
                    title,
                    media_type: MediaItemType::Readable(
                        ReadableKind::Book,
                        Progress {
                            current: 0,
                            total: doc.number_of_pages_median,
                        },
                        ReadStatus::PlanToRead,
                    ),
                    global_score,
                    external_id,
                    poster_url,
                    source: "openlibrary",
                    format_label: format!("{author} ({year})"),
                })
            })
            .collect();

        Ok(results)
    }
}
