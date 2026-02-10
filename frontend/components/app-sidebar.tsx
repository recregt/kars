"use client"

import * as React from "react"
import {
  CompassIcon,
  HelpCircleIcon,
  LayoutDashboardIcon,
  LibraryIcon,
  SettingsIcon,
} from "lucide-react"

import { NavMain } from "@/components/nav-main"
import { NavSecondary } from "@/components/nav-secondary"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

const navMainItems = [
  { title: "Dashboard", url: "#dashboard", icon: LayoutDashboardIcon },
  { title: "Library", url: "#library", icon: LibraryIcon },
  { title: "Explore", url: "#explore", icon: CompassIcon },
]

const navSecondaryItems = [
  { title: "Settings", url: "#settings", icon: SettingsIcon },
  { title: "Help", url: "#help", icon: HelpCircleIcon },
]

export function AppSidebar({
  activeView,
  onNavigate,
  onAddItem,
  ...props
}: React.ComponentProps<typeof Sidebar> & {
  activeView?: string
  onNavigate?: (view: string) => void
  onAddItem?: () => void
}) {
  return (
    <Sidebar collapsible="offcanvas" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" asChild>
              <a href="#">
                <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground font-bold text-sm">
                  K
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-bold">KARS</span>
                  <span className="truncate text-xs text-muted-foreground">
                    Media Archive
                  </span>
                </div>
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <NavMain
          items={navMainItems}
          activeView={activeView}
          onNavigate={onNavigate}
          onAddItem={onAddItem}
        />
        <NavSecondary items={navSecondaryItems} className="mt-auto" />
      </SidebarContent>
      <SidebarFooter>
        <div className="px-2 py-1 text-[10px] text-muted-foreground/60 text-center">
          KARS v0.1.0
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
