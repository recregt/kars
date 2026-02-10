use crate::core::models::{
    MediaItem, MediaItemType, ReadableKind, Progress, WatchStatus, ReadStatus,
};
use crate::core::input::{InputHandler, InputProvider};
use crate::core::storage::{StorageProvider, StorageError};
use crate::core::search::{SearchProvider, MediaSearchType};

pub struct App<S: StorageProvider, I: InputProvider> {
    archive: Vec<MediaItem>,
    storage: S,
    input: InputHandler<I>,
    searchers: Vec<Box<dyn SearchProvider>>,
    dirty: bool,
}

impl<S: StorageProvider, I: InputProvider> App<S, I> {
    pub fn new(
        storage: S,
        input_provider: I,
        searchers: Vec<Box<dyn SearchProvider>>,
    ) -> Result<Self, StorageError> {
        let archive = storage.load_all()?;
        Ok(Self {
            archive,
            storage,
            input: InputHandler::new(input_provider),
            searchers,
            dirty: false,
        })
    }

    fn auto_save(&mut self) {
        if self.dirty {
            if let Err(e) = self.storage.save_all(&self.archive) {
                eprintln!("Auto-save failed: {e}");
            }
            self.dirty = false;
        }
    }

    fn has_duplicate(&self, title: &str) -> bool {
        self.archive.iter().any(|item| item.title.eq_ignore_ascii_case(title))
    }

    pub fn run(&mut self) {
        println!("== KARS ARCHIVE SYSTEM ==");

        loop {
            println!("\n[1] Search & Add  [2] Add Manual  [3] List  [4] Detail  [5] Score  [6] Complete  [7] Progress  [8] Tags  [9] Save & Exit");
            let choice = match self.input.get_string_trimmed("Selection: ") {
                Ok(c) => c,
                Err(_) => continue,
            };

            match choice.as_str() {
                "1" => self.search_and_add_flow(),
                "2" => self.add_item_flow(),
                "3" => self.list_items(),
                "4" => self.detail_item(),
                "5" => self.set_score_flow(),
                "6" => self.complete_item(),
                "7" => self.update_progress_flow(),
                "8" => self.manage_tags_flow(),
                "9" => {
                    match self.storage.save_all(&self.archive) {
                        Ok(()) => println!("Archive saved. Goodbye!"),
                        Err(e) => eprintln!("Save failed: {e}"),
                    }
                    break;
                }
                _ => println!("Invalid selection, please try again."),
            }
        }
    }

    fn add_item_flow(&mut self) {
        let title = match self.input.get_string_trimmed("Title: ") {
            Ok(t) if !t.is_empty() => t,
            _ => { println!("Title cannot be empty."); return; }
        };

        println!("[1] Movie  [2] Series  [3] Readable");
        let kind = match self.input.get_string_trimmed("Type: ") {
            Ok(k) => k,
            Err(_) => return,
        };

        let media_type = match kind.as_str() {
            "1" => MediaItemType::Movie(WatchStatus::PlanToWatch),
            "2" => {
                let (current, total) = match self.read_progress() {
                    Some(p) => p,
                    None => return,
                };
                MediaItemType::Series(
                    Progress { current, total },
                    WatchStatus::Watching,
                )
            }
            "3" => {
                println!("[1] Book  [2] WebNovel  [3] LightNovel  [4] Manga  [5] Manhwa  [6] Webtoon");
                let readable_kind = match self.input.get_string_trimmed("Kind: ") {
                    Ok(ref k) => match k.as_str() {
                        "1" => ReadableKind::Book,
                        "2" => ReadableKind::WebNovel,
                        "3" => ReadableKind::LightNovel,
                        "4" => ReadableKind::Manga,
                        "5" => ReadableKind::Manhwa,
                        "6" => ReadableKind::Webtoon,
                        _ => { println!("Invalid kind."); return; }
                    },
                    Err(_) => return,
                };
                let (current, total) = match self.read_progress() {
                    Some(p) => p,
                    None => return,
                };
                MediaItemType::Readable(
                    readable_kind,
                    Progress { current, total },
                    ReadStatus::Reading,
                )
            }
            _ => { println!("Invalid type."); return; }
        };

        if self.has_duplicate(&title) {
            println!("Warning: '{}' already exists in archive.", title);
            let confirm = self.input.get_string_trimmed("Add anyway? (y/N): ").unwrap_or_default();
            if confirm != "y" && confirm != "Y" {
                println!("Cancelled.");
                return;
            }
        }

        let item = MediaItem::new(title.clone(), media_type);
        self.archive.push(item);
        self.dirty = true;
        self.auto_save();
        println!("Added: {title}");
    }

    fn search_and_add_flow(&mut self) {
        println!("\nSearch category:");
        println!("[1] Anime  [2] Manga/Manhwa  [3] Light Novel  [4] Movie  [5] Series  [6] Book");

        let search_type = match self.input.get_string_trimmed("Category: ") {
            Ok(ref c) => match c.as_str() {
                "1" => MediaSearchType::Anime,
                "2" => MediaSearchType::Manga,
                "3" => MediaSearchType::LightNovel,
                "4" => MediaSearchType::Movie,
                "5" => MediaSearchType::Series,
                "6" => MediaSearchType::Book,
                _ => { println!("Invalid category."); return; }
            },
            Err(_) => return,
        };

        // Collect all providers that support this type
        let matching: Vec<usize> = self
            .searchers
            .iter()
            .enumerate()
            .filter(|(_, s)| s.supported_types().contains(&search_type))
            .map(|(i, _)| i)
            .collect();

        if matching.is_empty() {
            println!("No search provider available for this category yet.");
            return;
        }

        // If multiple providers, let user choose
        let provider_idx = if matching.len() == 1 {
            matching[0]
        } else {
            println!("\nAvailable sources:");
            for (i, &idx) in matching.iter().enumerate() {
                println!("  [{}] {}", i + 1, self.searchers[idx].name());
            }
            let choice: usize = match self.input.parse_trimmed::<usize>("Source #: ") {
                Ok(v) if v >= 1 && v <= matching.len() => matching[v - 1],
                _ => { println!("Invalid selection."); return; }
            };
            choice
        };

        let query = match self.input.get_string_trimmed("Search: ") {
            Ok(q) if !q.is_empty() => q,
            _ => { println!("Search query cannot be empty."); return; }
        };

        println!("Searching {}...", self.searchers[provider_idx].name());

        let results = match self.searchers[provider_idx].search(&query, search_type) {
            Ok(r) if r.is_empty() => { println!("No results found."); return; }
            Ok(r) => r,
            Err(e) => { eprintln!("Search failed: {e}"); return; }
        };

        println!("\nResults:");
        for (i, r) in results.iter().enumerate() {
            println!("{}", r.display_line(i + 1));
        }
        println!("  [0] Cancel");

        let choice: usize = match self.input.parse_trimmed::<usize>("\nAdd #: ") {
            Ok(0) => return,
            Ok(v) if v >= 1 && v <= results.len() => v - 1,
            _ => { println!("Invalid selection."); return; }
        };

        let result = results.into_iter().nth(choice).unwrap();
        let title = result.title.clone();

        if self.has_duplicate(&title) {
            println!("Warning: '{}' already exists in archive.", title);
            let confirm = self.input.get_string_trimmed("Add anyway? (y/N): ").unwrap_or_default();
            if confirm != "y" && confirm != "Y" {
                println!("Cancelled.");
                return;
            }
        }

        let item = result.into_media_item();
        self.archive.push(item);
        self.dirty = true;
        self.auto_save();
        println!("Added: {title}");
    }

    fn read_progress(&mut self) -> Option<(u32, Option<u32>)> {
        let current: u32 = match self.input.parse_trimmed("Current episode/chapter: ") {
            Ok(v) => v,
            Err(_) => { println!("Invalid number."); return None; }
        };
        let total_str = match self.input.get_string_trimmed("Total (leave empty if unknown): ") {
            Ok(s) => s,
            Err(_) => return None,
        };
        let total: Option<u32> = if total_str.is_empty() {
            None
        } else {
            match total_str.parse() {
                Ok(v) => Some(v),
                Err(_) => { println!("Invalid number."); return None; }
            }
        };
        Some((current, total))
    }

    fn list_items(&self) {
        if self.archive.is_empty() {
            println!("Archive is empty.");
            return;
        }

        for (i, item) in self.archive.iter().enumerate() {
            let status = format_status(&item.media_type);
            let score = item
                .get_score_display()
                .map(|s| format!(" [{s:.1}]"))
                .unwrap_or_default();
            let completed = if item.is_completed() { " ✓" } else { "" };
            println!("  {}. {}{}{} — {}", i + 1, item.title, score, completed, status);
        }
    }

    fn select_item(&mut self, prompt: &str) -> Option<usize> {
        if self.archive.is_empty() {
            println!("Archive is empty.");
            return None;
        }
        self.list_items();
        let idx: usize = match self.input.parse_trimmed::<usize>(prompt) {
            Ok(v) if v >= 1 && v <= self.archive.len() => v - 1,
            _ => { println!("Invalid selection."); return None; }
        };
        Some(idx)
    }

    fn detail_item(&mut self) {
        let idx = match self.select_item("Item #: ") {
            Some(i) => i,
            None => return,
        };
        let item = &self.archive[idx];

        println!("\n--- {} ---", item.title);
        println!("  ID:     {}", item.id);
        println!("  Type:   {}", format_status(&item.media_type));

        if let Some(s) = item.get_score_display() {
            println!("  Score:  {s:.1}");
        }
        if let Some(g) = item.get_global_score_display() {
            println!("  Global: {g:.1}");
        }

        match &item.media_type {
            MediaItemType::Series(p, _) | MediaItemType::Readable(_, p, _) => {
                if let Some(pct) = p.percent() {
                    println!("  Progress: {pct:.1}%");
                }
            }
            _ => {}
        }

        if item.is_completed() {
            println!("  Status: Completed ✓");
        }

        if let Some(url) = &item.poster_url {
            println!("  Poster: {url}");
        }
        if let Some(eid) = item.external_id {
            println!("  ExtID:  {eid}");
        }
        if let Some(src) = &item.source {
            println!("  Source: {src}");
        }

        if !item.tags.is_empty() {
            let tags: Vec<&str> = item.tags.iter().map(|s| s.as_str()).collect();
            println!("  Tags:   {}", tags.join(", "));
        }
    }

    fn set_score_flow(&mut self) {
        let idx = match self.select_item("Score item #: ") {
            Some(i) => i,
            None => return,
        };
        let score: f32 = match self.input.parse_trimmed("Score (0.0 - 10.0): ") {
            Ok(v) => v,
            Err(_) => { println!("Invalid score."); return; }
        };
        self.archive[idx].set_score(score);
        self.dirty = true;
        self.auto_save();
        println!("Score set to {:.1} for '{}'",
            self.archive[idx].get_score_display().unwrap_or(0.0),
            self.archive[idx].title,
        );
    }

    fn complete_item(&mut self) {
        let idx = match self.select_item("Complete item #: ") {
            Some(i) => i,
            None => return,
        };
        if self.archive[idx].is_completed() {
            println!("'{}' is already completed.", self.archive[idx].title);
            return;
        }
        self.archive[idx].force_complete();
        let title = self.archive[idx].title.clone();
        self.dirty = true;
        self.auto_save();
        println!("'{title}' marked as completed ✓");
    }

    fn update_progress_flow(&mut self) {
        let idx = match self.select_item("Update progress for item #: ") {
            Some(i) => i,
            None => return,
        };

        // Read current values before mutable borrow
        let (cur, tot) = match &self.archive[idx].media_type {
            MediaItemType::Series(p, _) | MediaItemType::Readable(_, p, _) => {
                (p.current, p.total)
            }
            MediaItemType::Movie(_) => {
                println!("Movies don't have progress tracking.");
                return;
            }
        };

        let prompt = format!("Current [{}/{}]: ", cur, tot.map_or("?".into(), |t| t.to_string()));
        let new_current: u32 = match self.input.parse_trimmed(&prompt) {
            Ok(v) => v,
            Err(_) => { println!("Invalid number."); return; }
        };

        match &mut self.archive[idx].media_type {
            MediaItemType::Series(p, _) | MediaItemType::Readable(_, p, _) => {
                p.current = new_current;
                let info = if let Some(pct) = p.percent() {
                    format!("Updated — {pct:.1}%")
                } else {
                    format!("Updated — {}/{}", p.current, p.total.map_or("?".into(), |t: u32| t.to_string()))
                };
                println!("{info}");
            }
            _ => unreachable!(),
        }
        self.dirty = true;
        self.auto_save();
    }

    fn manage_tags_flow(&mut self) {
        let idx = match self.select_item("Tag item #: ") {
            Some(i) => i,
            None => return,
        };
        let item = &self.archive[idx];
        println!("\n--- {} ---", item.title);
        if item.tags.is_empty() {
            println!("  No tags.");
        } else {
            let tags: Vec<&str> = item.tags.iter().map(|s| s.as_str()).collect();
            println!("  Tags: {}", tags.join(", "));
        }
        println!("[1] Add tag  [2] Remove tag  [0] Cancel");
        let choice = match self.input.get_string_trimmed("Action: ") {
            Ok(c) => c,
            Err(_) => return,
        };
        match choice.as_str() {
            "1" => {
                let tag = match self.input.get_string_trimmed("New tag: ") {
                    Ok(t) if !t.is_empty() => t,
                    _ => { println!("Tag cannot be empty."); return; }
                };
                if self.archive[idx].tags.insert(tag.clone()) {
                    self.dirty = true;
                    self.auto_save();
                    println!("Tag '{tag}' added.");
                } else {
                    println!("Tag '{tag}' already exists.");
                }
            }
            "2" => {
                let tag = match self.input.get_string_trimmed("Remove tag: ") {
                    Ok(t) if !t.is_empty() => t,
                    _ => return,
                };
                if self.archive[idx].tags.remove(&tag) {
                    self.dirty = true;
                    self.auto_save();
                    println!("Tag '{tag}' removed.");
                } else {
                    println!("Tag '{tag}' not found.");
                }
            }
            _ => {}
        }
    }
}

fn format_status(media_type: &MediaItemType) -> String {
    match media_type {
        MediaItemType::Movie(s) => format!("Movie ({})", watch_label(s)),
        MediaItemType::Series(p, s) => {
            let progress = format_progress(p);
            format!("Series {progress} ({})", watch_label(s))
        }
        MediaItemType::Readable(kind, p, s) => {
            let progress = format_progress(p);
            format!("{kind:?} {progress} ({})", read_label(s))
        }
    }
}

fn format_progress(p: &Progress) -> String {
    let base = match p.total {
        Some(t) => format!("[{}/{}]", p.current, t),
        None => format!("[{}/?]", p.current),
    };
    match p.percent() {
        Some(pct) => format!("{base} {pct:.0}%"),
        None => base,
    }
}

fn watch_label(s: &WatchStatus) -> &'static str {
    match s {
        WatchStatus::Watching => "Watching",
        WatchStatus::PlanToWatch => "Plan to Watch",
        WatchStatus::Completed => "Completed",
        WatchStatus::OnHold => "On Hold",
        WatchStatus::Dropped => "Dropped",
    }
}

fn read_label(s: &ReadStatus) -> &'static str {
    match s {
        ReadStatus::Reading => "Reading",
        ReadStatus::PlanToRead => "Plan to Read",
        ReadStatus::Completed => "Completed",
        ReadStatus::OnHold => "On Hold",
        ReadStatus::Dropped => "Dropped",
    }
}