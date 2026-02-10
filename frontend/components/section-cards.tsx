"use client"

import useSWR from "swr"
import {
  BookOpenIcon,
  ClapperboardIcon,
  LibraryIcon,
  SparklesIcon,
  TrendingUpIcon,
  EyeIcon,
  CheckCircle2Icon,
  ClockIcon,
} from "lucide-react"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import type { Stats } from "@/lib/types"

const fetcher = (url: string) => fetch(url).then((r) => r.json())

export function SectionCards() {
  const { data: stats } = useSWR<Stats>("/api/stats", fetcher, {
    refreshInterval: 10000,
  })

  const cards = [
    {
      title: "Total Items",
      value: stats?.total ?? 0,
      description: "In your library",
      icon: LibraryIcon,
      trend: `${stats?.anime ?? 0} anime, ${stats?.movies ?? 0} movies`,
      trendIcon: TrendingUpIcon,
    },
    {
      title: "Watching",
      value: stats?.watching ?? 0,
      description: "Currently active",
      icon: EyeIcon,
      trend: "In progress",
      trendIcon: SparklesIcon,
    },
    {
      title: "Completed",
      value: stats?.completed ?? 0,
      description: "Finished media",
      icon: CheckCircle2Icon,
      trend: `${stats?.series ?? 0} series, ${stats?.readable ?? 0} readable`,
      trendIcon: ClapperboardIcon,
    },
    {
      title: "Plan to Watch",
      value: stats?.plan_to_watch ?? 0,
      description: "In your backlog",
      icon: ClockIcon,
      trend: `${stats?.on_hold ?? 0} on hold, ${stats?.dropped ?? 0} dropped`,
      trendIcon: BookOpenIcon,
    },
  ]

  return (
    <div className="*:data-[slot=card]:shadow-xs grid grid-cols-1 gap-4 px-4 sm:grid-cols-2 lg:grid-cols-4 lg:px-6">
      {cards.map((card) => (
        <Card key={card.title} className="@container/card">
          <CardHeader className="relative">
            <CardDescription>{card.title}</CardDescription>
            <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
              {card.value}
            </CardTitle>
            <div className="absolute right-4 top-4">
              <card.icon className="size-4 text-muted-foreground" />
            </div>
          </CardHeader>
          <CardFooter className="flex-col items-start gap-1 text-sm">
            <div className="line-clamp-1 flex gap-2 font-medium leading-none">
              {card.trend}
            </div>
            <div className="text-muted-foreground">{card.description}</div>
          </CardFooter>
        </Card>
      ))}
    </div>
  )
}
