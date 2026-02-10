"use client"

import * as React from "react"
import {
  type ColumnDef,
  type ColumnFiltersState,
  type SortingState,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table"
import {
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsLeftIcon,
  ChevronsRightIcon,
  ColumnsIcon,
  MoreVerticalIcon,
  PencilIcon,
  StarIcon,
  Trash2Icon,
  HeartIcon,
  SparklesIcon,
  ClapperboardIcon,
  MonitorPlayIcon,
  BookOpenIcon,
} from "lucide-react"
import { mutate } from "swr"

import { useIsMobile } from "@/hooks/use-mobile"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Separator } from "@/components/ui/separator"
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import type { MediaItem, Status, MediaType } from "@/lib/types"
import { statusLabel, mediaTypeLabel } from "@/lib/types"
import { cn } from "@/lib/utils"

// --- Helpers ---

function getMediaIcon(type: string) {
  switch (type) {
    case "anime":
      return <SparklesIcon className="size-3.5" />
    case "movie":
      return <ClapperboardIcon className="size-3.5" />
    case "series":
      return <MonitorPlayIcon className="size-3.5" />
    default:
      return <BookOpenIcon className="size-3.5" />
  }
}

function getStatusVariant(status: string) {
  switch (status) {
    case "watching":
    case "reading":
      return "default"
    case "completed":
      return "secondary"
    case "plan_to_watch":
    case "plan_to_read":
      return "outline"
    case "on_hold":
      return "secondary"
    case "dropped":
      return "destructive"
    default:
      return "outline"
  }
}

function getScoreColor(score: number | null) {
  if (score == null) return ""
  if (score >= 8.5) return "text-[hsl(var(--score-high))]"
  if (score >= 7) return "text-[hsl(var(--score-mid))]"
  return "text-[hsl(var(--score-low))]"
}

// --- Columns ---

const columns: ColumnDef<MediaItem>[] = [
  {
    accessorKey: "poster_url",
    header: "",
    cell: ({ row }) => {
      const url = row.original.poster_url
      return (
        <div className="size-10 overflow-hidden rounded-md bg-secondary shrink-0">
          {url ? (
            <img
              src={url}
              alt=""
              className="size-full object-cover"
              crossOrigin="anonymous"
              loading="lazy"
            />
          ) : (
            <div className="flex size-full items-center justify-center text-muted-foreground">
              {getMediaIcon(row.original.media_type)}
            </div>
          )}
        </div>
      )
    },
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "title",
    header: "Title",
    cell: ({ row }) => {
      return <MediaItemEditor item={row.original} />
    },
    enableHiding: false,
  },
  {
    accessorKey: "media_type",
    header: "Type",
    cell: ({ row }) => {
      return (
        <div className="flex items-center gap-1.5">
          {getMediaIcon(row.original.media_type)}
          <span className="text-xs">{mediaTypeLabel(row.original.media_type)}</span>
        </div>
      )
    },
    filterFn: (row, id, value) => {
      return value.includes(row.getValue(id))
    },
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const status = row.original.status
      return (
        <Badge variant={getStatusVariant(status)} className="text-[11px]">
          {statusLabel(status)}
        </Badge>
      )
    },
    filterFn: (row, id, value) => {
      return value.includes(row.getValue(id))
    },
  },
  {
    accessorKey: "progress",
    header: "Progress",
    cell: ({ row }) => {
      const item = row.original
      const total = item.total_episodes
      return (
        <span className="text-xs tabular-nums text-muted-foreground">
          {item.progress}{total ? ` / ${total}` : ""}
        </span>
      )
    },
  },
  {
    accessorKey: "score",
    header: "Score",
    cell: ({ row }) => {
      const score = row.original.score
      if (score == null) return <span className="text-xs text-muted-foreground">—</span>
      const display = (score / 10).toFixed(1)
      return (
        <div className="flex items-center gap-1">
          <StarIcon className={cn("size-3", getScoreColor(score))} />
          <span className={cn("text-xs font-medium tabular-nums", getScoreColor(score))}>
            {display}
          </span>
        </div>
      )
    },
  },
  {
    accessorKey: "favorite",
    header: "",
    cell: ({ row }) => {
      const fav = row.original.favorite
      return fav ? (
        <HeartIcon className="size-3.5 fill-destructive text-destructive" />
      ) : null
    },
    enableSorting: false,
  },
  {
    id: "actions",
    cell: ({ row }) => <RowActions item={row.original} />,
    enableSorting: false,
    enableHiding: false,
  },
]

// --- Row actions dropdown (edit/delete) ---

function RowActions({ item }: { item: MediaItem }) {
  const [deleting, setDeleting] = React.useState(false)

  async function handleDelete() {
    setDeleting(true)
    try {
      const res = await fetch(`/api/items/${item.id}`, { method: "DELETE" })
      if (res.ok) {
        mutate("/api/items")
        mutate("/api/stats")
      }
    } finally {
      setDeleting(false)
    }
  }

  return (
    <AlertDialog>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="size-7">
            <MoreVerticalIcon className="size-4" />
            <span className="sr-only">Actions</span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-36">
          <DropdownMenuItem asChild>
            <MediaItemEditor item={item} triggerVariant="menuitem" />
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <AlertDialogTrigger asChild>
            <DropdownMenuItem className="text-destructive focus:text-destructive">
              <Trash2Icon className="size-4" />
              Delete
            </DropdownMenuItem>
          </AlertDialogTrigger>
        </DropdownMenuContent>
      </DropdownMenu>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete &quot;{item.title}&quot;?</AlertDialogTitle>
          <AlertDialogDescription>
            This will permanently remove this item from your library.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            disabled={deleting}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {deleting ? "Deleting…" : "Delete"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

// --- Media item editor (Sheet) ---

const allStatuses: Status[] = [
  "watching",
  "completed",
  "plan_to_watch",
  "on_hold",
  "dropped",
  "reading",
  "plan_to_read",
]

const allMediaTypes: MediaType[] = [
  "anime",
  "movie",
  "series",
  "manga",
  "manhwa",
  "webtoon",
  "book",
  "light_novel",
  "web_novel",
]

function MediaItemEditor({
  item,
  triggerVariant = "link",
}: {
  item: MediaItem
  triggerVariant?: "link" | "menuitem"
}) {
  const [open, setOpen] = React.useState(false)
  const [saving, setSaving] = React.useState(false)
  const [formData, setFormData] = React.useState({
    title: item.title,
    media_type: item.media_type,
    status: item.status,
    score: item.score,
    progress: item.progress,
    total_episodes: item.total_episodes,
    favorite: item.favorite,
    tags: item.tags,
  })

  // Reset form when sheet opens
  React.useEffect(() => {
    if (open) {
      setFormData({
        title: item.title,
        media_type: item.media_type,
        status: item.status,
        score: item.score,
        progress: item.progress,
        total_episodes: item.total_episodes,
        favorite: item.favorite,
        tags: item.tags,
      })
    }
  }, [open, item])

  async function handleSave() {
    setSaving(true)
    try {
      const res = await fetch(`/api/items/${item.id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          ...item,
          ...formData,
        }),
      })
      if (res.ok) {
        mutate("/api/items")
        mutate("/api/stats")
        setOpen(false)
      }
    } finally {
      setSaving(false)
    }
  }

  const trigger =
    triggerVariant === "link" ? (
      <SheetTrigger asChild>
        <Button
          variant="link"
          className="w-fit px-0 text-left text-foreground font-medium"
        >
          {item.title}
        </Button>
      </SheetTrigger>
    ) : (
      <SheetTrigger asChild>
        <button className="flex w-full items-center gap-2 text-sm">
          <PencilIcon className="size-4" />
          Edit
        </button>
      </SheetTrigger>
    )

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      {trigger}
      <SheetContent side="right" className="flex flex-col">
        <SheetHeader className="gap-1">
          <SheetTitle>{item.title}</SheetTitle>
          <SheetDescription>
            Edit media item details
          </SheetDescription>
        </SheetHeader>

        {/* Poster preview */}
        {item.poster_url && (
          <div className="mx-auto mt-2 h-40 w-28 overflow-hidden rounded-lg">
            <img
              src={item.poster_url}
              alt={item.title}
              className="h-full w-full object-cover"
              crossOrigin="anonymous"
            />
          </div>
        )}

        <div className="flex flex-1 flex-col gap-4 overflow-y-auto py-4 text-sm">
          <Separator />
          <form
            className="flex flex-col gap-4"
            onSubmit={(e) => {
              e.preventDefault()
              handleSave()
            }}
          >
            {/* Title */}
            <div className="flex flex-col gap-2">
              <Label htmlFor="edit-title">Title</Label>
              <Input
                id="edit-title"
                value={formData.title}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, title: e.target.value }))
                }
              />
            </div>

            {/* Type + Status */}
            <div className="grid grid-cols-2 gap-4">
              <div className="flex flex-col gap-2">
                <Label htmlFor="edit-type">Type</Label>
                <Select
                  value={formData.media_type}
                  onValueChange={(v) =>
                    setFormData((prev) => ({
                      ...prev,
                      media_type: v as MediaType,
                    }))
                  }
                >
                  <SelectTrigger id="edit-type" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {allMediaTypes.map((t) => (
                      <SelectItem key={t} value={t}>
                        {mediaTypeLabel(t)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex flex-col gap-2">
                <Label htmlFor="edit-status">Status</Label>
                <Select
                  value={formData.status}
                  onValueChange={(v) =>
                    setFormData((prev) => ({
                      ...prev,
                      status: v as Status,
                    }))
                  }
                >
                  <SelectTrigger id="edit-status" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {allStatuses.map((s) => (
                      <SelectItem key={s} value={s}>
                        {statusLabel(s)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Progress + Total */}
            <div className="grid grid-cols-2 gap-4">
              <div className="flex flex-col gap-2">
                <Label htmlFor="edit-progress">Progress</Label>
                <Input
                  id="edit-progress"
                  type="number"
                  min={0}
                  value={formData.progress}
                  onChange={(e) =>
                    setFormData((prev) => ({
                      ...prev,
                      progress: parseInt(e.target.value) || 0,
                    }))
                  }
                />
              </div>
              <div className="flex flex-col gap-2">
                <Label htmlFor="edit-total">Total Episodes</Label>
                <Input
                  id="edit-total"
                  type="number"
                  min={0}
                  value={formData.total_episodes ?? ""}
                  placeholder="Unknown"
                  onChange={(e) =>
                    setFormData((prev) => ({
                      ...prev,
                      total_episodes: e.target.value
                        ? parseInt(e.target.value)
                        : null,
                    }))
                  }
                />
              </div>
            </div>

            {/* Score slider */}
            <div className="flex flex-col gap-2">
              <Label>
                Score:{" "}
                <span className="font-mono font-bold">
                  {formData.score != null
                    ? (formData.score / 10).toFixed(1)
                    : "—"}
                </span>
              </Label>
              <Slider
                min={0}
                max={100}
                step={5}
                value={[formData.score ?? 0]}
                onValueChange={([v]) =>
                  setFormData((prev) => ({ ...prev, score: v || null }))
                }
              />
              <div className="flex justify-between text-[10px] text-muted-foreground">
                <span>0</span>
                <span>5.0</span>
                <span>10.0</span>
              </div>
            </div>

            {/* Favorite toggle */}
            <div className="flex items-center justify-between">
              <Label htmlFor="edit-favorite" className="flex items-center gap-2">
                <HeartIcon
                  className={cn(
                    "size-4",
                    formData.favorite
                      ? "fill-destructive text-destructive"
                      : "text-muted-foreground"
                  )}
                />
                Favorite
              </Label>
              <Switch
                id="edit-favorite"
                checked={formData.favorite}
                onCheckedChange={(checked) =>
                  setFormData((prev) => ({ ...prev, favorite: checked }))
                }
              />
            </div>
          </form>
        </div>

        <SheetFooter className="mt-auto flex gap-2 sm:flex-col sm:space-x-0">
          <Button className="w-full" onClick={handleSave} disabled={saving}>
            {saving ? "Saving…" : "Save Changes"}
          </Button>
          <SheetClose asChild>
            <Button variant="outline" className="w-full">
              Cancel
            </Button>
          </SheetClose>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}

// --- Main DataTable ---

export function DataTable({ data }: { data: MediaItem[] }) {
  const [sorting, setSorting] = React.useState<SortingState>([])
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  )
  const [columnVisibility, setColumnVisibility] =
    React.useState<VisibilityState>({})
  const [globalFilter, setGlobalFilter] = React.useState("")
  const [statusFilter, setStatusFilter] = React.useState<string>("all")
  const [typeFilter, setTypeFilter] = React.useState<string>("all")

  // Apply status and type filters
  const filteredData = React.useMemo(() => {
    let result = data
    if (statusFilter !== "all") {
      result = result.filter((item) => item.status === statusFilter)
    }
    if (typeFilter !== "all") {
      if (typeFilter === "readable") {
        result = result.filter((item) =>
          ["manga", "manhwa", "webtoon", "book", "light_novel", "web_novel"].includes(item.media_type)
        )
      } else {
        result = result.filter((item) => item.media_type === typeFilter)
      }
    }
    return result
  }, [data, statusFilter, typeFilter])

  const table = useReactTable({
    data: filteredData,
    columns,
    state: {
      sorting,
      columnFilters,
      columnVisibility,
      globalFilter,
    },
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onColumnVisibilityChange: setColumnVisibility,
    onGlobalFilterChange: setGlobalFilter,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    globalFilterFn: (row, _columnId, filterValue) => {
      return row.original.title
        .toLowerCase()
        .includes(filterValue.toLowerCase())
    },
    initialState: {
      pagination: { pageSize: 20 },
    },
  })

  return (
    <div className="flex flex-col gap-4 px-4 lg:px-6">
      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-2">
        <Input
          placeholder="Search library..."
          value={globalFilter}
          onChange={(e) => setGlobalFilter(e.target.value)}
          className="h-8 w-full max-w-xs"
        />

        {/* Status filter */}
        <Select value={statusFilter} onValueChange={setStatusFilter}>
          <SelectTrigger className="h-8 w-[140px]">
            <SelectValue placeholder="Status" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="watching">Watching</SelectItem>
            <SelectItem value="reading">Reading</SelectItem>
            <SelectItem value="completed">Completed</SelectItem>
            <SelectItem value="plan_to_watch">Plan to Watch</SelectItem>
            <SelectItem value="plan_to_read">Plan to Read</SelectItem>
            <SelectItem value="on_hold">On Hold</SelectItem>
            <SelectItem value="dropped">Dropped</SelectItem>
          </SelectContent>
        </Select>

        {/* Type filter */}
        <Select value={typeFilter} onValueChange={setTypeFilter}>
          <SelectTrigger className="h-8 w-[140px]">
            <SelectValue placeholder="Type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Types</SelectItem>
            <SelectItem value="anime">Anime</SelectItem>
            <SelectItem value="movie">Movie</SelectItem>
            <SelectItem value="series">TV Series</SelectItem>
            <SelectItem value="readable">Manga & Books</SelectItem>
          </SelectContent>
        </Select>

        {/* Column visibility */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm" className="ml-auto h-8">
              <ColumnsIcon />
              <span className="hidden lg:inline">Columns</span>
              <ChevronDownIcon />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-44">
            {table
              .getAllColumns()
              .filter(
                (column) =>
                  typeof column.accessorFn !== "undefined" &&
                  column.getCanHide()
              )
              .map((column) => (
                <DropdownMenuCheckboxItem
                  key={column.id}
                  className="capitalize"
                  checked={column.getIsVisible()}
                  onCheckedChange={(value) =>
                    column.toggleVisibility(!!value)
                  }
                >
                  {column.id === "media_type" ? "Type" : column.id}
                </DropdownMenuCheckboxItem>
              ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* Table */}
      <div className="overflow-hidden rounded-lg border">
        <Table>
          <TableHeader className="sticky top-0 z-10 bg-muted">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead key={header.id} colSpan={header.colSpan}>
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow key={row.id}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No items found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      {/* Pagination */}
      <div className="flex items-center justify-between px-2">
        <div className="hidden flex-1 text-sm text-muted-foreground lg:flex">
          {filteredData.length} item(s) total
        </div>
        <div className="flex w-full items-center gap-8 lg:w-fit">
          <div className="hidden items-center gap-2 lg:flex">
            <Label className="text-sm font-medium">Rows per page</Label>
            <Select
              value={`${table.getState().pagination.pageSize}`}
              onValueChange={(value) => table.setPageSize(Number(value))}
            >
              <SelectTrigger className="w-20 h-8">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="top">
                {[10, 20, 30, 50].map((size) => (
                  <SelectItem key={size} value={`${size}`}>
                    {size}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="flex w-fit items-center justify-center text-sm font-medium">
            Page {table.getState().pagination.pageIndex + 1} of{" "}
            {table.getPageCount()}
          </div>
          <div className="ml-auto flex items-center gap-2 lg:ml-0">
            <Button
              variant="outline"
              className="hidden h-8 w-8 p-0 lg:flex"
              onClick={() => table.setPageIndex(0)}
              disabled={!table.getCanPreviousPage()}
            >
              <ChevronsLeftIcon />
            </Button>
            <Button
              variant="outline"
              size="icon"
              className="size-8"
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
            >
              <ChevronLeftIcon />
            </Button>
            <Button
              variant="outline"
              size="icon"
              className="size-8"
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              <ChevronRightIcon />
            </Button>
            <Button
              variant="outline"
              className="hidden size-8 lg:flex"
              size="icon"
              onClick={() => table.setPageIndex(table.getPageCount() - 1)}
              disabled={!table.getCanNextPage()}
            >
              <ChevronsRightIcon />
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
