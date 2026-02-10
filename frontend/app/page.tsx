"use client"

import * as React from "react"
import useSWR from "swr"

import { AppSidebar } from "@/components/app-sidebar"
import { SiteHeader } from "@/components/site-header"
import { SectionCards } from "@/components/section-cards"
import { DataTable } from "@/components/data-table"
import { ExploreView } from "@/components/explore-view"
import { AddMediaDialog } from "@/components/add-media-dialog"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import type { MediaItem } from "@/lib/types"

const fetcher = (url: string) => fetch(url).then((r) => r.json())

export default function Page() {
  const [activeView, setActiveView] = React.useState("dashboard")
  const [addDialogOpen, setAddDialogOpen] = React.useState(false)

  const { data: items = [] } = useSWR<MediaItem[]>("/api/items", fetcher, {
    refreshInterval: 5000,
  })

  return (
    <SidebarProvider>
      <AppSidebar
        variant="inset"
        activeView={activeView}
        onNavigate={setActiveView}
        onAddItem={() => setAddDialogOpen(true)}
      />
      <SidebarInset>
        <SiteHeader />
        <div className="flex flex-1 flex-col">
          <div className="@container/main flex flex-1 flex-col gap-2">
            <div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
              {/* Dashboard view */}
              {activeView === "dashboard" && (
                <>
                  <SectionCards />
                  <DataTable data={items} />
                </>
              )}

              {/* Library view (just the table) */}
              {activeView === "library" && (
                <DataTable data={items} />
              )}

              {/* Explore view */}
              {activeView === "explore" && (
                <ExploreView />
              )}
            </div>
          </div>
        </div>
      </SidebarInset>

      {/* Add media dialog */}
      <AddMediaDialog open={addDialogOpen} onOpenChange={setAddDialogOpen} />
    </SidebarProvider>
  )
}
