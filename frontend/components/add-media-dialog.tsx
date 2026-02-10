"use client"

import * as React from "react"
import { mutate } from "swr"
import { PlusIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import type { MediaType, Status } from "@/lib/types"
import { mediaTypeLabel, statusLabel } from "@/lib/types"

const mediaTypes: MediaType[] = [
  "anime", "movie", "series", "manga", "manhwa", "webtoon", "book", "light_novel", "web_novel",
]

const statuses: Status[] = [
  "watching", "completed", "plan_to_watch", "on_hold", "dropped", "reading", "plan_to_read",
]

interface AddMediaDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function AddMediaDialog({ open, onOpenChange }: AddMediaDialogProps) {
  const [saving, setSaving] = React.useState(false)
  const [title, setTitle] = React.useState("")
  const [mediaType, setMediaType] = React.useState<MediaType>("anime")
  const [status, setStatus] = React.useState<Status>("plan_to_watch")
  const [totalEpisodes, setTotalEpisodes] = React.useState("")

  function reset() {
    setTitle("")
    setMediaType("anime")
    setStatus("plan_to_watch")
    setTotalEpisodes("")
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!title.trim()) return

    setSaving(true)
    try {
      const payload = {
        id: "",
        title: title.trim(),
        media_type: mediaType,
        status: status,
        score: null,
        global_score: null,
        progress: 0,
        total_episodes: totalEpisodes ? parseInt(totalEpisodes) : null,
        poster_url: null,
        source: null,
        external_id: null,
        tags: [],
        favorite: false,
      }

      const res = await fetch("/api/items", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      })
      if (res.ok) {
        mutate("/api/items")
        mutate("/api/stats")
        reset()
        onOpenChange(false)
      }
    } finally {
      setSaving(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Add Media</DialogTitle>
          <DialogDescription>
            Add a new item to your library manually.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="add-title">Title</Label>
            <Input
              id="add-title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Enter title..."
              required
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="add-type">Type</Label>
              <Select value={mediaType} onValueChange={(v) => setMediaType(v as MediaType)}>
                <SelectTrigger id="add-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {mediaTypes.map((t) => (
                    <SelectItem key={t} value={t}>
                      {mediaTypeLabel(t)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="add-status">Status</Label>
              <Select value={status} onValueChange={(v) => setStatus(v as Status)}>
                <SelectTrigger id="add-status">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {statuses.map((s) => (
                    <SelectItem key={s} value={s}>
                      {statusLabel(s)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="add-total">Total Episodes / Chapters</Label>
            <Input
              id="add-total"
              type="number"
              min={0}
              value={totalEpisodes}
              onChange={(e) => setTotalEpisodes(e.target.value)}
              placeholder="Leave empty if unknown"
            />
          </div>
          <DialogFooter>
            <Button type="submit" disabled={saving || !title.trim()}>
              {saving ? "Adding…" : "Add to Library"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
