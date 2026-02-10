use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum WatchStatus {
    Watching,
    PlanToWatch,
    Completed,
    OnHold,
    Dropped,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ReadStatus {
    Reading,
    PlanToRead,
    Completed,
    OnHold,
    Dropped,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Progress {
    pub current: u32,
    pub total: Option<u32>,
}

impl Progress {
    pub fn percent(&self) -> Option<f32> {
        match self.total {
            Some(t) if t > 0 => Some((self.current as f32 / t as f32) * 100.0),
            Some(0) => Some(0.0),
            _ => None,
        }
    }

    pub fn is_finished(&self) -> bool {
        match self.total {
            Some(t) if t > 0 => self.current >= t,
            _ => false,
        }
    }
}

/// Categorizes different types of readable media to reduce code duplication.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ReadableKind {
    Book,
    WebNovel,
    LightNovel,
    Manga,
    Manhwa,
    Webtoon,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MediaItemType {
    Movie(WatchStatus),
    Series(Progress, WatchStatus),
    Readable(ReadableKind, Progress, ReadStatus),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: Uuid,
    pub title: String,
    pub media_type: MediaItemType,
    #[serde(default)]
    pub score: Option<u8>,        // Stored 0-100 (represents 0.0-10.0)
    #[serde(default)]
    pub global_score: Option<u8>, // Stored 0-100 (represents 0.0-10.0)
    #[serde(default)]
    pub external_id: Option<u32>,
    #[serde(default)]
    pub poster_url: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub tags: HashSet<String>,
}

impl MediaItem {
    pub fn new(title: String, media_type: MediaItemType) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            media_type,
            score: None,
            global_score: None,
            external_id: None,
            poster_url: None,
            source: None,
            tags: HashSet::new(),
        }
    }

    fn clamp_score(input_score: f32) -> u8 {
        (input_score.clamp(0.0, 10.0) * 10.0).round() as u8
    }

    fn score_display(score: Option<u8>) -> Option<f32> {
        score.map(|s| s as f32 / 10.0)
    }

    pub fn set_score(&mut self, input_score: f32) {
        self.score = Some(Self::clamp_score(input_score));
    }

    #[allow(dead_code)]
    pub fn set_global_score(&mut self, input_score: f32) {
        self.global_score = Some(Self::clamp_score(input_score));
    }

    pub fn get_score_display(&self) -> Option<f32> {
        Self::score_display(self.score)
    }

    pub fn get_global_score_display(&self) -> Option<f32> {
        Self::score_display(self.global_score)
    }

    pub fn is_completed(&self) -> bool {
        match &self.media_type {
            MediaItemType::Movie(WatchStatus::Completed)
            | MediaItemType::Series(_, WatchStatus::Completed)
            | MediaItemType::Readable(_, _, ReadStatus::Completed) => true,

            MediaItemType::Series(p, _)
            | MediaItemType::Readable(_, p, _) if p.is_finished() => true,

            _ => false,
        }
    }

    pub fn force_complete(&mut self) {
        match &mut self.media_type {
            MediaItemType::Movie(s) => {
                *s = WatchStatus::Completed;
            },
            MediaItemType::Series(p, s) => {
                *s = WatchStatus::Completed;
                p.total = p.total.or(Some(p.current));
                if let Some(t) = p.total { p.current = t; }
            },
            MediaItemType::Readable(_, p, s) => {
                *s = ReadStatus::Completed;
                p.total = p.total.or(Some(p.current));
                if let Some(t) = p.total { p.current = t; }
            }
        }
    }
}