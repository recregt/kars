export type MediaType =
  | "movie"
  | "series"
  | "anime"
  | "manga"
  | "manhwa"
  | "webtoon"
  | "book"
  | "light_novel"
  | "web_novel"

/** Broad category used for sidebar filters */
export type MediaFilter = "all" | "movie" | "series" | "anime" | "readable"

export type Status =
  | "watching"
  | "completed"
  | "plan_to_watch"
  | "on_hold"
  | "dropped"
  | "reading"
  | "plan_to_read"

export interface MediaItem {
  id: string
  title: string
  media_type: MediaType
  status: Status
  score: number | null
  global_score: number | null
  progress: number
  total_episodes: number | null
  poster_url: string | null
  source: string | null
  external_id: string | null
  tags: string[]
  favorite: boolean
}

export interface ExploreResult {
  title: string
  media_type: string
  global_score: number | null
  external_id: string | null
  poster_url: string | null
  source: string
  total_episodes: number | null
  format_label: string
}

export type ExploreSearchType = "anime" | "movie" | "series" | "manga" | "book" | "light_novel"

export interface Stats {
  total: number
  watching: number
  completed: number
  plan_to_watch: number
  on_hold: number
  dropped: number
  movies: number
  series: number
  anime: number
  readable: number
}

/** Check if a media type falls under the "readable" group */
export function isReadable(type: MediaType): boolean {
  return ["manga", "manhwa", "webtoon", "book", "light_novel", "web_novel"].includes(type)
}

/** Check if a status is a "reading" variant */
export function isActiveStatus(status: Status): boolean {
  return status === "watching" || status === "reading"
}

/** Readable label for a media type */
export function mediaTypeLabel(type: MediaType | string): string {
  switch (type) {
    case "movie": return "Movie"
    case "series": return "TV Series"
    case "anime": return "Anime"
    case "manga": return "Manga"
    case "manhwa": return "Manhwa"
    case "webtoon": return "Webtoon"
    case "book": return "Book"
    case "light_novel": return "Light Novel"
    case "web_novel": return "Web Novel"
    default: return type
  }
}

/** Readable label for a status */
export function statusLabel(status: Status): string {
  switch (status) {
    case "watching": return "Watching"
    case "completed": return "Completed"
    case "plan_to_watch": return "Plan to Watch"
    case "on_hold": return "On Hold"
    case "dropped": return "Dropped"
    case "reading": return "Reading"
    case "plan_to_read": return "Plan to Read"
    default: return status
  }
}
