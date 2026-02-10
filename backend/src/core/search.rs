use crate::core::models::{MediaItem, MediaItemType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaSearchType {
    Anime,
    Manga,
    LightNovel,
    Movie,
    Series,
    Book,
}

pub struct SearchResult {
    pub title: String,
    pub media_type: MediaItemType,
    pub global_score: Option<u8>,
    pub external_id: Option<u32>,
    pub poster_url: Option<String>,
    pub source: &'static str,
    pub format_label: String,
}

impl SearchResult {
    pub fn into_media_item(self) -> MediaItem {
        let mut item = MediaItem::new(self.title, self.media_type);
        item.global_score = self.global_score;
        item.external_id = self.external_id;
        item.poster_url = self.poster_url;
        item.source = Some(self.source.to_string());
        item
    }

    pub fn display_line(&self, idx: usize) -> String {
        let count = match &self.media_type {
            MediaItemType::Series(p, _) => p.total.map(|t| format!(" [{t} ep]")),
            MediaItemType::Readable(_, p, _) => p.total.map(|t| format!(" [{t} ch]")),
            MediaItemType::Movie(_) => None,
        }
        .unwrap_or_default();

        let score = self
            .global_score
            .map(|s| format!(" ★ {:.1}", s as f32 / 10.0))
            .unwrap_or_default();

        format!(
            "  {}. {}{}{} — {}",
            idx, self.title, count, score, self.format_label
        )
    }
}

pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &str;
    fn supported_types(&self) -> &[MediaSearchType];
    fn search(
        &self,
        query: &str,
        media_type: MediaSearchType,
    ) -> Result<Vec<SearchResult>, SearchError>;
}
