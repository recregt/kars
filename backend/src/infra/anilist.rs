use crate::core::models::{
    MediaItemType, Progress, ReadStatus, ReadableKind, WatchStatus,
};
use crate::core::search::{MediaSearchType, SearchError, SearchProvider, SearchResult};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const ANILIST_URL: &str = "https://graphql.anilist.co";

const SEARCH_QUERY: &str = r#"
query ($search: String, $type: MediaType, $format: MediaFormat) {
  Page(perPage: 10) {
    media(search: $search, type: $type, format: $format, sort: SEARCH_MATCH) {
      id
      title {
        romaji
        english
      }
      episodes
      chapters
      meanScore
      coverImage {
        large
      }
      format
      countryOfOrigin
    }
  }
}
"#;

// ── GraphQL request ──────────────────────────────────────────────

#[derive(Serialize)]
struct GqlRequest {
    query: &'static str,
    variables: GqlVariables,
}

#[derive(Serialize)]
struct GqlVariables {
    search: String,
    #[serde(rename = "type")]
    media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

// ── GraphQL response ─────────────────────────────────────────────

#[derive(Deserialize)]
struct GqlResponse {
    data: Option<GqlData>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize)]
struct GqlError {
    message: String,
}

#[derive(Deserialize)]
struct GqlData {
    #[serde(rename = "Page")]
    page: GqlPage,
}

#[derive(Deserialize)]
struct GqlPage {
    media: Vec<GqlMedia>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlMedia {
    id: u32,
    title: GqlTitle,
    episodes: Option<u32>,
    chapters: Option<u32>,
    mean_score: Option<u32>,
    cover_image: Option<GqlCoverImage>,
    format: Option<String>,
    country_of_origin: Option<String>,
}

#[derive(Deserialize)]
struct GqlTitle {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Deserialize)]
struct GqlCoverImage {
    large: Option<String>,
}

// ── Client ───────────────────────────────────────────────────────

pub struct AniListClient {
    client: Client,
}

impl AniListClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn map_media(
        &self,
        media: GqlMedia,
        search_type: MediaSearchType,
    ) -> Option<SearchResult> {
        let title = media
            .title
            .english
            .filter(|s| !s.is_empty())
            .or(media.title.romaji)
            .unwrap_or_else(|| "Unknown".into());

        let format_str = media.format.as_deref().unwrap_or("UNKNOWN");
        let country = media.country_of_origin.as_deref().unwrap_or("JP");

        let (media_type, format_label) = match search_type {
            MediaSearchType::Anime => {
                if format_str == "MOVIE" {
                    (
                        MediaItemType::Movie(WatchStatus::PlanToWatch),
                        "Movie".to_string(),
                    )
                } else {
                    let label = match format_str {
                        "TV" => "TV",
                        "TV_SHORT" => "TV Short",
                        "OVA" => "OVA",
                        "ONA" => "ONA",
                        "SPECIAL" => "Special",
                        "MUSIC" => "Music",
                        other => other,
                    };
                    (
                        MediaItemType::Series(
                            Progress { current: 0, total: media.episodes },
                            WatchStatus::PlanToWatch,
                        ),
                        label.to_string(),
                    )
                }
            }
            MediaSearchType::Manga | MediaSearchType::LightNovel => {
                let (kind, label) = if format_str == "NOVEL" {
                    (ReadableKind::LightNovel, "Light Novel")
                } else {
                    match country {
                        "KR" => (ReadableKind::Manhwa, "Manhwa"),
                        _ => (ReadableKind::Manga, "Manga"),
                    }
                };
                (
                    MediaItemType::Readable(
                        kind,
                        Progress { current: 0, total: media.chapters },
                        ReadStatus::PlanToRead,
                    ),
                    label.to_string(),
                )
            }
            _ => return None,
        };

        // AniList meanScore: 0-100 → our global_score: 0-100 (u8)
        let global_score = media.mean_score.map(|s| s.min(100) as u8);

        Some(SearchResult {
            title,
            media_type,
            global_score,
            external_id: Some(media.id),
            poster_url: media.cover_image.and_then(|c| c.large),
            source: "anilist",
            format_label,
        })
    }
}

impl SearchProvider for AniListClient {
    fn name(&self) -> &str {
        "AniList"
    }

    fn supported_types(&self) -> &[MediaSearchType] {
        &[
            MediaSearchType::Anime,
            MediaSearchType::Manga,
            MediaSearchType::LightNovel,
        ]
    }

    fn search(
        &self,
        query: &str,
        media_type: MediaSearchType,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let (api_type, format_filter) = match media_type {
            MediaSearchType::Anime => ("ANIME", None),
            MediaSearchType::Manga => ("MANGA", None),
            MediaSearchType::LightNovel => ("MANGA", Some("NOVEL")),
            _ => return Ok(Vec::new()),
        };

        let body = GqlRequest {
            query: SEARCH_QUERY,
            variables: GqlVariables {
                search: query.to_string(),
                media_type: api_type.to_string(),
                format: format_filter.map(|f| f.to_string()),
            },
        };

        let response = self
            .client
            .post(ANILIST_URL)
            .json(&body)
            .send()
            .map_err(|e| SearchError::Network(e.to_string()))?;

        let gql: GqlResponse = response
            .json()
            .map_err(|e| SearchError::Parse(e.to_string()))?;

        if let Some(errors) = gql.errors {
            let msg = errors
                .iter()
                .map(|e| e.message.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(SearchError::Api(msg));
        }

        let data = gql
            .data
            .ok_or_else(|| SearchError::Api("No data in response".into()))?;

        let results = data
            .page
            .media
            .into_iter()
            .filter_map(|m| self.map_media(m, media_type))
            .collect();

        Ok(results)
    }
}
