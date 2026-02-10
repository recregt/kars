use crate::core::models::{MediaItemType, Progress, ReadStatus, ReadableKind};
use crate::core::search::{MediaSearchType, SearchError, SearchProvider, SearchResult};
use reqwest::blocking::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://api.mangadex.org";
const COVER_BASE: &str = "https://uploads.mangadex.org/covers";
const USER_AGENT: &str = "kars-archive/0.1 (https://github.com/kars)";

// ── Response types ───────────────────────────────────────────────

#[derive(Deserialize)]
struct MangaListResponse {
    data: Vec<MangaData>,
}

#[derive(Deserialize)]
struct MangaData {
    id: String,
    attributes: MangaAttributes,
    relationships: Vec<Relationship>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MangaAttributes {
    title: serde_json::Value, // {"en": "...", "ja": "..."}
    original_language: Option<String>,
    last_chapter: Option<String>,
    year: Option<u32>,
    status: Option<String>,
    tags: Vec<TagData>,
}

#[derive(Deserialize)]
struct TagData {
    attributes: TagAttributes,
}

#[derive(Deserialize)]
struct TagAttributes {
    name: serde_json::Value,
}

#[derive(Deserialize)]
struct Relationship {
    #[serde(rename = "type")]
    rel_type: String,
    attributes: Option<serde_json::Value>,
}

// ── Statistics types ─────────────────────────────────────────────

#[derive(Deserialize)]
struct StatsResponse {
    statistics: serde_json::Value,
}

// ── Client ───────────────────────────────────────────────────────

pub struct MangaDexClient {
    client: Client,
}

impl MangaDexClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    fn extract_title(title_obj: &serde_json::Value) -> String {
        // Prefer English, then Japanese-romanized, then first available
        title_obj
            .get("en")
            .or_else(|| title_obj.get("ja-ro"))
            .or_else(|| title_obj.get("ja"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                title_obj
                    .as_object()
                    .and_then(|m| m.values().next())
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("Unknown")
            .to_string()
    }

    fn extract_cover_filename(relationships: &[Relationship]) -> Option<String> {
        relationships
            .iter()
            .find(|r| r.rel_type == "cover_art")
            .and_then(|r| r.attributes.as_ref())
            .and_then(|a| a.get("fileName"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn extract_author(relationships: &[Relationship]) -> String {
        relationships
            .iter()
            .find(|r| r.rel_type == "author")
            .and_then(|r| r.attributes.as_ref())
            .and_then(|a| a.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    fn has_tag(tags: &[TagData], name: &str) -> bool {
        tags.iter().any(|t| {
            t.attributes
                .name
                .get("en")
                .and_then(|v| v.as_str())
                .map(|s| s.eq_ignore_ascii_case(name))
                .unwrap_or(false)
        })
    }

    fn determine_kind(attrs: &MangaAttributes) -> (ReadableKind, &'static str) {
        let lang = attrs.original_language.as_deref().unwrap_or("ja");
        let is_long_strip = Self::has_tag(&attrs.tags, "Long Strip");

        match lang {
            "ko" => {
                if is_long_strip {
                    (ReadableKind::Webtoon, "Webtoon")
                } else {
                    (ReadableKind::Manhwa, "Manhwa")
                }
            }
            "ja" => (ReadableKind::Manga, "Manga"),
            _ => (ReadableKind::Manga, "Manga"),
        }
    }

    fn fetch_stats(&self, ids: &[&str]) -> serde_json::Value {
        if ids.is_empty() {
            return serde_json::Value::Object(serde_json::Map::new());
        }

        let params: Vec<(&str, &str)> = ids.iter().map(|id| ("manga[]", *id)).collect();

        self.client
            .get(&format!("{BASE_URL}/statistics/manga"))
            .query(&params)
            .send()
            .ok()
            .and_then(|r| r.json::<StatsResponse>().ok())
            .map(|s| s.statistics)
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()))
    }
}

impl SearchProvider for MangaDexClient {
    fn name(&self) -> &str {
        "MangaDex"
    }

    fn supported_types(&self) -> &[MediaSearchType] {
        &[MediaSearchType::Manga]
    }

    fn search(
        &self,
        query: &str,
        media_type: MediaSearchType,
    ) -> Result<Vec<SearchResult>, SearchError> {
        if media_type != MediaSearchType::Manga {
            return Ok(Vec::new());
        }

        let resp = self
            .client
            .get(&format!("{BASE_URL}/manga"))
            .query(&[
                ("title", query),
                ("limit", "10"),
                ("includes[]", "cover_art"),
                ("includes[]", "author"),
                ("order[relevance]", "desc"),
                ("contentRating[]", "safe"),
                ("contentRating[]", "suggestive"),
            ])
            .send()
            .map_err(|e| SearchError::Network(e.to_string()))?;

        let data: MangaListResponse = resp
            .json()
            .map_err(|e| SearchError::Parse(e.to_string()))?;

        // Batch fetch statistics for all results
        let ids: Vec<&str> = data.data.iter().map(|m| m.id.as_str()).collect();
        let stats = self.fetch_stats(&ids);

        let results = data
            .data
            .into_iter()
            .map(|manga| {
                let title = Self::extract_title(&manga.attributes.title);
                let author = Self::extract_author(&manga.relationships);
                let (kind, kind_label) = Self::determine_kind(&manga.attributes);

                let total_chapters = manga
                    .attributes
                    .last_chapter
                    .as_deref()
                    .and_then(|s| s.parse::<f32>().ok())
                    .map(|c| c as u32);

                let year = manga
                    .attributes
                    .year
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "?".into());

                let status = manga
                    .attributes
                    .status
                    .as_deref()
                    .unwrap_or("unknown");

                let poster_url = Self::extract_cover_filename(&manga.relationships)
                    .map(|f| format!("{COVER_BASE}/{}/{f}.256.jpg", manga.id));

                // Stats: rating.bayesian is 1-10
                let global_score = stats
                    .get(&manga.id)
                    .and_then(|s| s.get("rating"))
                    .and_then(|r| r.get("bayesian"))
                    .and_then(|v| v.as_f64())
                    .map(|r| (r.clamp(0.0, 10.0) * 10.0).round() as u8);

                SearchResult {
                    title,
                    media_type: MediaItemType::Readable(
                        kind,
                        Progress { current: 0, total: total_chapters },
                        ReadStatus::PlanToRead,
                    ),
                    global_score,
                    external_id: None, // MangaDex uses UUIDs, not u32
                    poster_url,
                    source: "mangadex",
                    format_label: format!("{kind_label} · {author} ({year}, {status})"),
                }
            })
            .collect();

        Ok(results)
    }
}
