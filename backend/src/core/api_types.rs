use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::models::{
    MediaItem, MediaItemType, Progress, ReadStatus, ReadableKind, WatchStatus,
};

/// Flat JSON representation for the REST API.
/// This is what the frontend sends and receives.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiMediaItem {
    pub id: String,
    pub title: String,
    pub media_type: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_score: Option<f32>,
    pub progress: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_episodes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
}

// ── MediaItem → ApiMediaItem ─────────────────────────────────

impl From<&MediaItem> for ApiMediaItem {
    fn from(item: &MediaItem) -> Self {
        let (media_type, status, progress, total) = match &item.media_type {
            MediaItemType::Movie(ws) => ("movie", watch_status_str(ws), 0, None),
            MediaItemType::Series(p, ws) => {
                let mt = match item.source.as_deref() {
                    Some("anilist") => "anime",
                    _ => "series",
                };
                (mt, watch_status_str(ws), p.current, p.total)
            }
            MediaItemType::Readable(kind, p, rs) => {
                let mt = readable_kind_str(kind);
                (mt, read_status_str(rs), p.current, p.total)
            }
        };

        ApiMediaItem {
            id: item.id.to_string(),
            title: item.title.clone(),
            media_type: media_type.to_string(),
            status: status.to_string(),
            score: item.get_score_display(),
            global_score: item.get_global_score_display(),
            progress,
            total_episodes: total,
            poster_url: item.poster_url.clone(),
            source: item.source.clone(),
            external_id: item.external_id.map(|e| e.to_string()),
            tags: item.tags.iter().cloned().collect(),
            favorite: item.tags.contains("favorite"),
        }
    }
}

// ── ApiMediaItem → MediaItem ─────────────────────────────────

impl ApiMediaItem {
    pub fn into_media_item(self) -> Result<MediaItem, String> {
        let id = if self.id.is_empty() {
            Uuid::new_v4()
        } else {
            Uuid::parse_str(&self.id).map_err(|e| format!("Invalid UUID: {e}"))?
        };

        let progress = Progress {
            current: self.progress,
            total: self.total_episodes,
        };

        let media_type = match self.media_type.as_str() {
            "movie" => MediaItemType::Movie(parse_watch_status(&self.status)),
            "series" | "anime" => {
                MediaItemType::Series(progress, parse_watch_status(&self.status))
            }
            "manga" => MediaItemType::Readable(
                ReadableKind::Manga,
                progress,
                parse_read_status(&self.status),
            ),
            "manhwa" => MediaItemType::Readable(
                ReadableKind::Manhwa,
                progress,
                parse_read_status(&self.status),
            ),
            "webtoon" => MediaItemType::Readable(
                ReadableKind::Webtoon,
                progress,
                parse_read_status(&self.status),
            ),
            "book" => MediaItemType::Readable(
                ReadableKind::Book,
                progress,
                parse_read_status(&self.status),
            ),
            "light_novel" => MediaItemType::Readable(
                ReadableKind::LightNovel,
                progress,
                parse_read_status(&self.status),
            ),
            "web_novel" => MediaItemType::Readable(
                ReadableKind::WebNovel,
                progress,
                parse_read_status(&self.status),
            ),
            other => return Err(format!("Unknown media_type: {other}")),
        };

        let mut tags: std::collections::HashSet<String> =
            self.tags.into_iter().collect();
        if self.favorite {
            tags.insert("favorite".to_string());
        }

        let mut item = MediaItem {
            id,
            title: self.title,
            media_type,
            score: None,
            global_score: None,
            external_id: self.external_id.and_then(|e| e.parse().ok()),
            poster_url: self.poster_url,
            source: self.source,
            tags,
        };

        if let Some(s) = self.score {
            item.set_score(s);
        }
        if let Some(g) = self.global_score {
            item.set_global_score(g);
        }

        Ok(item)
    }
}

// ── Explore result (external search) ─────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiExploreResult {
    pub title: String,
    pub media_type: String,
    pub global_score: Option<f32>,
    pub external_id: Option<String>,
    pub poster_url: Option<String>,
    pub source: String,
    pub total_episodes: Option<u32>,
    pub format_label: String,
}

impl ApiExploreResult {
    pub fn from_search_result(r: &crate::core::search::SearchResult) -> Self {
        let (media_type, total) = match &r.media_type {
            MediaItemType::Movie(_) => ("movie", None),
            MediaItemType::Series(p, _) => {
                let mt = match r.source {
                    "anilist" => "anime",
                    _ => "series",
                };
                (mt, p.total)
            }
            MediaItemType::Readable(kind, p, _) => {
                (readable_kind_str(kind), p.total)
            }
        };

        ApiExploreResult {
            title: r.title.clone(),
            media_type: media_type.to_string(),
            global_score: r.global_score.map(|s| s as f32 / 10.0),
            external_id: r.external_id.map(|e| e.to_string()),
            poster_url: r.poster_url.clone(),
            source: r.source.to_string(),
            total_episodes: total,
            format_label: r.format_label.clone(),
        }
    }
}

// ── Stats ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ApiStats {
    pub total: usize,
    pub watching: usize,
    pub completed: usize,
    pub plan_to_watch: usize,
    pub on_hold: usize,
    pub dropped: usize,
    pub movies: usize,
    pub series: usize,
    pub anime: usize,
    pub readable: usize,
}

impl ApiStats {
    pub fn from_items(items: &[ApiMediaItem]) -> Self {
        let mut stats = ApiStats {
            total: items.len(),
            watching: 0,
            completed: 0,
            plan_to_watch: 0,
            on_hold: 0,
            dropped: 0,
            movies: 0,
            series: 0,
            anime: 0,
            readable: 0,
        };

        for item in items {
            match item.status.as_str() {
                "watching" | "reading" => stats.watching += 1,
                "completed" => stats.completed += 1,
                "plan_to_watch" | "plan_to_read" => stats.plan_to_watch += 1,
                "on_hold" => stats.on_hold += 1,
                "dropped" => stats.dropped += 1,
                _ => {}
            }
            match item.media_type.as_str() {
                "movie" => stats.movies += 1,
                "series" => stats.series += 1,
                "anime" => stats.anime += 1,
                _ => stats.readable += 1,
            }
        }

        stats
    }
}

// ── Helpers ──────────────────────────────────────────────────

fn watch_status_str(s: &WatchStatus) -> &'static str {
    match s {
        WatchStatus::Watching => "watching",
        WatchStatus::PlanToWatch => "plan_to_watch",
        WatchStatus::Completed => "completed",
        WatchStatus::OnHold => "on_hold",
        WatchStatus::Dropped => "dropped",
    }
}

fn read_status_str(s: &ReadStatus) -> &'static str {
    match s {
        ReadStatus::Reading => "reading",
        ReadStatus::PlanToRead => "plan_to_read",
        ReadStatus::Completed => "completed",
        ReadStatus::OnHold => "on_hold",
        ReadStatus::Dropped => "dropped",
    }
}

fn readable_kind_str(k: &ReadableKind) -> &'static str {
    match k {
        ReadableKind::Manga => "manga",
        ReadableKind::Manhwa => "manhwa",
        ReadableKind::Webtoon => "webtoon",
        ReadableKind::Book => "book",
        ReadableKind::LightNovel => "light_novel",
        ReadableKind::WebNovel => "web_novel",
    }
}

fn parse_watch_status(s: &str) -> WatchStatus {
    match s {
        "watching" | "reading" => WatchStatus::Watching,
        "plan_to_watch" | "plan_to_read" => WatchStatus::PlanToWatch,
        "completed" => WatchStatus::Completed,
        "on_hold" => WatchStatus::OnHold,
        "dropped" => WatchStatus::Dropped,
        _ => WatchStatus::PlanToWatch,
    }
}

fn parse_read_status(s: &str) -> ReadStatus {
    match s {
        "reading" | "watching" => ReadStatus::Reading,
        "plan_to_read" | "plan_to_watch" => ReadStatus::PlanToRead,
        "completed" => ReadStatus::Completed,
        "on_hold" => ReadStatus::OnHold,
        "dropped" => ReadStatus::Dropped,
        _ => ReadStatus::PlanToRead,
    }
}
