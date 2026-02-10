"use client"

import * as React from "react"
import useSWR, { mutate } from "swr"
import {
  Search,
  Plus,
  Sparkles,
  Film,
  BookOpen,
  Star,
  Loader2,
  Check,
  MonitorPlayIcon,
} from "lucide-react"

import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import type { ExploreResult, ExploreSearchType, MediaItem } from "@/lib/types"
import { mediaTypeLabel } from "@/lib/types"
import { cn } from "@/lib/utils"

const fetcher = (url: string) => fetch(url).then((r) => r.json())

const searchTypes: { label: string; value: ExploreSearchType; icon: React.ReactNode }[] = [
  { label: "Anime", value: "anime", icon: <Sparkles className="h-3.5 w-3.5" /> },
  { label: "Movie", value: "movie", icon: <Film className="h-3.5 w-3.5" /> },
  { label: "Series", value: "series", icon: <MonitorPlayIcon className="h-3.5 w-3.5" /> },
  { label: "Manga", value: "manga", icon: <BookOpen className="h-3.5 w-3.5" /> },
  { label: "Book", value: "book", icon: <BookOpen className="h-3.5 w-3.5" /> },
]

export function ExploreView() {
  const [query, setQuery] = React.useState("")
  const [searchType, setSearchType] = React.useState<ExploreSearchType>("anime")
  const [submittedQuery, setSubmittedQuery] = React.useState("")
  const [addingIds, setAddingIds] = React.useState<Set<string>>(new Set())
  const [addedIds, setAddedIds] = React.useState<Set<string>>(new Set())

  const searchKey =
    submittedQuery.length >= 2
      ? `/api/explore?q=${encodeURIComponent(submittedQuery)}&type=${searchType}`
      : null

  const { data: results, isLoading } = useSWR<ExploreResult[]>(searchKey, fetcher, {
    revalidateOnFocus: false,
  })

  function handleSearch(e: React.FormEvent) {
    e.preventDefault()
    if (query.trim().length >= 2) {
      setSubmittedQuery(query.trim())
      setAddedIds(new Set())
    }
  }

  async function addToLibrary(result: ExploreResult) {
    const key = `${result.source}-${result.external_id}`
    if (addingIds.has(key) || addedIds.has(key)) return

    setAddingIds((prev) => new Set(prev).add(key))

    const defaultStatus = [
      "manga", "manhwa", "webtoon", "book", "light_novel", "web_novel",
    ].includes(result.media_type)
      ? "plan_to_read"
      : "plan_to_watch"

    const payload: Partial<MediaItem> = {
      id: "",
      title: result.title,
      media_type: result.media_type as MediaItem["media_type"],
      status: defaultStatus as MediaItem["status"],
      score: null,
      global_score: result.global_score,
      progress: 0,
      total_episodes: result.total_episodes,
      poster_url: result.poster_url,
      source: result.source,
      external_id: result.external_id,
      tags: [],
      favorite: false,
    }

    try {
      const res = await fetch("/api/items", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      })
      if (res.ok) {
        setAddedIds((prev) => new Set(prev).add(key))
        mutate("/api/items")
        mutate("/api/stats")
      }
    } catch {
      // ignore
    } finally {
      setAddingIds((prev) => {
        const next = new Set(prev)
        next.delete(key)
        return next
      })
    }
  }

  return (
    <div className="flex flex-col gap-4 px-4 lg:px-6">
      {/* Search form */}
      <form onSubmit={handleSearch} className="flex flex-wrap items-center gap-2">
        <div className="relative flex-1 min-w-[200px] max-w-md">
          <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search anime, movies, manga..."
            className="h-8 pl-8"
          />
        </div>

        <Select
          value={searchType}
          onValueChange={(v) => setSearchType(v as ExploreSearchType)}
        >
          <SelectTrigger className="h-8 w-[120px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {searchTypes.map((t) => (
              <SelectItem key={t.value} value={t.value}>
                {t.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Button type="submit" size="sm" className="h-8">
          <Search className="mr-1 h-3.5 w-3.5" />
          Search
        </Button>
      </form>

      {/* Results */}
      {isLoading && (
        <div className="flex items-center justify-center py-16">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
          <span className="ml-2 text-sm text-muted-foreground">Searching...</span>
        </div>
      )}

      {!isLoading && !results && !submittedQuery && (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <Sparkles className="mb-3 h-10 w-10 text-muted-foreground/40" />
          <p className="text-sm font-medium">Discover new media</p>
          <p className="mt-1 text-xs text-muted-foreground">
            Search for anime, movies, manga and more from external databases
          </p>
        </div>
      )}

      {!isLoading && results && results.length === 0 && (
        <div className="flex flex-col items-center justify-center py-16">
          <p className="text-sm text-muted-foreground">
            No results for &quot;{submittedQuery}&quot;
          </p>
        </div>
      )}

      {!isLoading && results && results.length > 0 && (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7">
          {results.map((result, i) => {
            const key = `${result.source}-${result.external_id}`
            const isAdding = addingIds.has(key)
            const isAdded = addedIds.has(key)

            return (
              <div
                key={`${key}-${i}`}
                className="group relative flex flex-col overflow-hidden rounded-lg border border-border bg-card transition-all hover:border-primary/30"
              >
                {/* Poster */}
                <div className="relative aspect-[2/3] w-full overflow-hidden bg-secondary">
                  {result.poster_url ? (
                    <img
                      src={result.poster_url}
                      alt={result.title}
                      className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
                      crossOrigin="anonymous"
                      loading="lazy"
                    />
                  ) : (
                    <div className="flex h-full items-center justify-center text-muted-foreground">
                      <Film className="h-8 w-8" />
                    </div>
                  )}

                  {/* Score badge */}
                  {result.global_score != null && (
                    <div className="absolute right-1.5 top-1.5 flex items-center gap-0.5 rounded bg-background/80 px-1.5 py-0.5 text-[10px] font-bold backdrop-blur-sm">
                      <Star className="h-2.5 w-2.5 text-yellow-500" />
                      {result.global_score.toFixed(1)}
                    </div>
                  )}

                  {/* Add overlay */}
                  <div className="absolute inset-x-0 bottom-0 flex justify-center bg-gradient-to-t from-black/60 to-transparent p-2 opacity-0 transition-opacity group-hover:opacity-100">
                    <Button
                      size="sm"
                      variant={isAdded ? "secondary" : "default"}
                      className="h-7 text-[11px]"
                      onClick={() => addToLibrary(result)}
                      disabled={isAdding || isAdded}
                    >
                      {isAdding ? (
                        <Loader2 className="mr-1 h-3 w-3 animate-spin" />
                      ) : isAdded ? (
                        <Check className="mr-1 h-3 w-3" />
                      ) : (
                        <Plus className="mr-1 h-3 w-3" />
                      )}
                      {isAdded ? "Added" : "Add"}
                    </Button>
                  </div>
                </div>

                {/* Info */}
                <div className="flex flex-col gap-1 p-2">
                  <h3 className="line-clamp-2 text-xs font-semibold leading-tight">
                    {result.title}
                  </h3>
                  <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
                    <span>{mediaTypeLabel(result.media_type)}</span>
                    {result.total_episodes && (
                      <>
                        <span>·</span>
                        <span>{result.total_episodes} ep</span>
                      </>
                    )}
                  </div>
                  <Badge variant="outline" className="w-fit text-[9px] uppercase font-mono">
                    {result.source}
                  </Badge>
                </div>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
