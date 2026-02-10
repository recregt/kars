use crate::core::models::{
    MediaItem, MediaItemType, Progress, ReadStatus, ReadableKind, WatchStatus,
};
use crate::core::storage::{StorageError, StorageProvider};
use libsql::{Builder, Connection};
use std::collections::HashSet;
use tokio::runtime::Runtime;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
// Database — async-only, no runtime.  Used by the web server.
// ═══════════════════════════════════════════════════════════════

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Connect to a local SQLite file (async).
    pub async fn local(path: &str) -> Result<Self, StorageError> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(StorageError::Io)?;
        }
        let db = Builder::new_local(path)
            .build()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let conn = db
            .connect()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let storage = Self { conn };
        storage.run_migrations().await?;
        Ok(storage)
    }

    /// Connect to a remote Turso database (async).
    pub async fn turso(url: &str, token: &str) -> Result<Self, StorageError> {
        let db = Builder::new_remote(url.to_string(), token.to_string())
            .build()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let conn = db
            .connect()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let storage = Self { conn };
        storage.run_migrations().await?;
        Ok(storage)
    }

    async fn run_migrations(&self) -> Result<(), StorageError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS media_items (
                    id            TEXT PRIMARY KEY,
                    title         TEXT NOT NULL,
                    media_type    TEXT NOT NULL,
                    readable_kind TEXT,
                    watch_status  TEXT,
                    read_status   TEXT,
                    progress_cur  INTEGER NOT NULL DEFAULT 0,
                    progress_tot  INTEGER,
                    score         INTEGER,
                    global_score  INTEGER,
                    external_id   INTEGER,
                    poster_url    TEXT,
                    source        TEXT,
                    tags          TEXT NOT NULL DEFAULT '[]'
                )",
                (),
            )
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(())
    }

    // ── Bulk operations (used by CLI via SqlStorage) ─────────

    pub async fn load_all(&self) -> Result<Vec<MediaItem>, StorageError> {
        let mut rows = self
            .conn
            .query("SELECT * FROM media_items ORDER BY title", ())
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            items.push(row_to_media_item(&row)?);
        }
        Ok(items)
    }

    pub async fn save_all(&self, items: &[MediaItem]) -> Result<(), StorageError> {
        let tx = self
            .conn
            .transaction()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        tx.execute("DELETE FROM media_items", ())
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        for item in items {
            insert_item_in_tx(&tx, item).await?;
        }

        tx.commit()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(())
    }

    // ── Granular operations (used by web API) ────────────────

    pub async fn get_item(&self, id: Uuid) -> Result<Option<MediaItem>, StorageError> {
        let mut rows = self
            .conn
            .query(
                "SELECT * FROM media_items WHERE id = ?1",
                libsql::params![id.to_string()],
            )
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        match rows
            .next()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            Some(row) => Ok(Some(row_to_media_item(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn upsert_item(&self, item: &MediaItem) -> Result<(), StorageError> {
        let (media_type, readable_kind, watch_status, read_status, cur, tot) =
            decompose_media_type(&item.media_type);
        let tags_json = serde_json::to_string(&item.tags)?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO media_items
                    (id, title, media_type, readable_kind, watch_status, read_status,
                     progress_cur, progress_tot, score, global_score,
                     external_id, poster_url, source, tags)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                libsql::params![
                    item.id.to_string(),
                    item.title.clone(),
                    media_type,
                    readable_kind,
                    watch_status,
                    read_status,
                    cur as i64,
                    tot.map(|t| t as i64),
                    item.score.map(|s| s as i64),
                    item.global_score.map(|s| s as i64),
                    item.external_id.map(|e| e as i64),
                    item.poster_url.clone(),
                    item.source.clone(),
                    tags_json,
                ],
            )
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_item(&self, id: Uuid) -> Result<bool, StorageError> {
        let affected = self
            .conn
            .execute(
                "DELETE FROM media_items WHERE id = ?1",
                libsql::params![id.to_string()],
            )
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(affected > 0)
    }

    pub async fn search_items(&self, query: &str) -> Result<Vec<MediaItem>, StorageError> {
        let pattern = format!("%{query}%");
        let mut rows = self
            .conn
            .query(
                "SELECT * FROM media_items WHERE title LIKE ?1 ORDER BY title",
                libsql::params![pattern],
            )
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            items.push(row_to_media_item(&row)?);
        }
        Ok(items)
    }
}

// ═══════════════════════════════════════════════════════════════
// SqlStorage — sync wrapper for the CLI.  Owns a tokio Runtime.
// ═══════════════════════════════════════════════════════════════

pub struct SqlStorage {
    db: Database,
    rt: Runtime,
}

impl SqlStorage {
    pub fn local(path: &str) -> Result<Self, StorageError> {
        let rt = Runtime::new().map_err(|e| StorageError::Database(e.to_string()))?;
        let db = rt.block_on(Database::local(path))?;
        Ok(Self { db, rt })
    }

    pub fn turso(url: &str, token: &str) -> Result<Self, StorageError> {
        let rt = Runtime::new().map_err(|e| StorageError::Database(e.to_string()))?;
        let db = rt.block_on(Database::turso(url, token))?;
        Ok(Self { db, rt })
    }
}

impl StorageProvider for SqlStorage {
    fn load_all(&self) -> Result<Vec<MediaItem>, StorageError> {
        self.rt.block_on(self.db.load_all())
    }

    fn save_all(&self, items: &[MediaItem]) -> Result<(), StorageError> {
        self.rt.block_on(self.db.save_all(items))
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

async fn insert_item_in_tx(
    tx: &libsql::Transaction,
    item: &MediaItem,
) -> Result<(), StorageError> {
    let (media_type, readable_kind, watch_status, read_status, cur, tot) =
        decompose_media_type(&item.media_type);
    let tags_json = serde_json::to_string(&item.tags)?;

    tx.execute(
        "INSERT INTO media_items
            (id, title, media_type, readable_kind, watch_status, read_status,
             progress_cur, progress_tot, score, global_score,
             external_id, poster_url, source, tags)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        libsql::params![
            item.id.to_string(),
            item.title.clone(),
            media_type,
            readable_kind,
            watch_status,
            read_status,
            cur as i64,
            tot.map(|t| t as i64),
            item.score.map(|s| s as i64),
            item.global_score.map(|s| s as i64),
            item.external_id.map(|e| e as i64),
            item.poster_url.clone(),
            item.source.clone(),
            tags_json,
        ],
    )
    .await
    .map_err(|e| StorageError::Database(e.to_string()))?;
    Ok(())
}

fn decompose_media_type(
    mt: &MediaItemType,
) -> (
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    u32,
    Option<u32>,
) {
    match mt {
        MediaItemType::Movie(ws) => ("movie", None, Some(watch_str(ws)), None, 0, None),
        MediaItemType::Series(p, ws) => {
            ("series", None, Some(watch_str(ws)), None, p.current, p.total)
        }
        MediaItemType::Readable(kind, p, rs) => (
            "readable",
            Some(readable_str(kind)),
            None,
            Some(read_str(rs)),
            p.current,
            p.total,
        ),
    }
}

fn row_to_media_item(row: &libsql::Row) -> Result<MediaItem, StorageError> {
    let id_str: String = row
        .get::<String>(0)
        .map_err(|e| StorageError::Database(e.to_string()))?;
    let title: String = row
        .get::<String>(1)
        .map_err(|e| StorageError::Database(e.to_string()))?;
    let media_type_str: String = row
        .get::<String>(2)
        .map_err(|e| StorageError::Database(e.to_string()))?;
    let readable_kind: Option<String> = row
        .get::<libsql::Value>(3)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Text(s) => Some(s),
            _ => None,
        });
    let watch_status: Option<String> = row
        .get::<libsql::Value>(4)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Text(s) => Some(s),
            _ => None,
        });
    let read_status: Option<String> = row
        .get::<libsql::Value>(5)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Text(s) => Some(s),
            _ => None,
        });
    let progress_cur: i64 = row.get::<i64>(6).unwrap_or(0);
    let progress_tot: Option<i64> = row
        .get::<libsql::Value>(7)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Integer(i) => Some(i),
            _ => None,
        });
    let score: Option<i64> = row
        .get::<libsql::Value>(8)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Integer(i) => Some(i),
            _ => None,
        });
    let global_score: Option<i64> = row
        .get::<libsql::Value>(9)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Integer(i) => Some(i),
            _ => None,
        });
    let external_id: Option<i64> = row
        .get::<libsql::Value>(10)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Integer(i) => Some(i),
            _ => None,
        });
    let poster_url: Option<String> = row
        .get::<libsql::Value>(11)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Text(s) => Some(s),
            _ => None,
        });
    let source: Option<String> = row
        .get::<libsql::Value>(12)
        .ok()
        .and_then(|v| match v {
            libsql::Value::Text(s) => Some(s),
            _ => None,
        });
    let tags_json: String = row.get::<String>(13).unwrap_or_else(|_| "[]".into());

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| StorageError::Corruption(format!("Invalid UUID: {e}")))?;

    let progress = Progress {
        current: progress_cur as u32,
        total: progress_tot.map(|t| t as u32),
    };

    let media_type = match media_type_str.as_str() {
        "movie" => {
            let ws = parse_watch_status(watch_status.as_deref());
            MediaItemType::Movie(ws)
        }
        "series" => {
            let ws = parse_watch_status(watch_status.as_deref());
            MediaItemType::Series(progress, ws)
        }
        "readable" => {
            let kind = parse_readable_kind(readable_kind.as_deref());
            let rs = parse_read_status(read_status.as_deref());
            MediaItemType::Readable(kind, progress, rs)
        }
        other => {
            return Err(StorageError::Corruption(format!(
                "Unknown media_type: {other}"
            )));
        }
    };

    let tags: HashSet<String> = serde_json::from_str(&tags_json).unwrap_or_default();

    Ok(MediaItem {
        id,
        title,
        media_type,
        score: score.map(|s| s as u8),
        global_score: global_score.map(|s| s as u8),
        external_id: external_id.map(|e| e as u32),
        poster_url,
        source,
        tags,
    })
}

// ── Enum ↔ String mappings ───────────────────────────────────

fn watch_str(s: &WatchStatus) -> &'static str {
    match s {
        WatchStatus::Watching => "watching",
        WatchStatus::PlanToWatch => "plan_to_watch",
        WatchStatus::Completed => "completed",
        WatchStatus::OnHold => "on_hold",
        WatchStatus::Dropped => "dropped",
    }
}

fn read_str(s: &ReadStatus) -> &'static str {
    match s {
        ReadStatus::Reading => "reading",
        ReadStatus::PlanToRead => "plan_to_read",
        ReadStatus::Completed => "completed",
        ReadStatus::OnHold => "on_hold",
        ReadStatus::Dropped => "dropped",
    }
}

fn readable_str(k: &ReadableKind) -> &'static str {
    match k {
        ReadableKind::Book => "book",
        ReadableKind::WebNovel => "web_novel",
        ReadableKind::LightNovel => "light_novel",
        ReadableKind::Manga => "manga",
        ReadableKind::Manhwa => "manhwa",
        ReadableKind::Webtoon => "webtoon",
    }
}

fn parse_watch_status(s: Option<&str>) -> WatchStatus {
    match s {
        Some("watching") => WatchStatus::Watching,
        Some("plan_to_watch") => WatchStatus::PlanToWatch,
        Some("completed") => WatchStatus::Completed,
        Some("on_hold") => WatchStatus::OnHold,
        Some("dropped") => WatchStatus::Dropped,
        _ => WatchStatus::PlanToWatch,
    }
}

fn parse_read_status(s: Option<&str>) -> ReadStatus {
    match s {
        Some("reading") => ReadStatus::Reading,
        Some("plan_to_read") => ReadStatus::PlanToRead,
        Some("completed") => ReadStatus::Completed,
        Some("on_hold") => ReadStatus::OnHold,
        Some("dropped") => ReadStatus::Dropped,
        _ => ReadStatus::PlanToRead,
    }
}

fn parse_readable_kind(s: Option<&str>) -> ReadableKind {
    match s {
        Some("book") => ReadableKind::Book,
        Some("web_novel") => ReadableKind::WebNovel,
        Some("light_novel") => ReadableKind::LightNovel,
        Some("manga") => ReadableKind::Manga,
        Some("manhwa") => ReadableKind::Manhwa,
        Some("webtoon") => ReadableKind::Webtoon,
        _ => ReadableKind::Book,
    }
}
