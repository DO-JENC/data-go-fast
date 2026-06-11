import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { useDatasources } from "@/hooks/use-datasources"
import { useGroups } from "@/hooks/use-groups"
import { useJobs } from "@/hooks/use-jobs"
import { cn } from "@/lib/utils"
import type { Datasource } from "@/types/datasource"
import type { Job } from "@/types/job"
import {
  ChevronDown,
  FileCog,
  FileSpreadsheet,
  ListTree,
  Plus,
} from "lucide-react"
import { useEffect, useState } from "react"

export default function TreePanel({
  onSelect,
  refreshKey,
  revealDatasourceId,
  onRevealHandled,
}: {
  onSelect: (
    item: Datasource | Job | string,
    type: "datasource" | "job" | "form",
  ) => void
  refreshKey?: number
  revealDatasourceId?: string | null
  onRevealHandled?: () => void
}) {
  const { currentGroup } = useGroups()
  const {
    datasources,
    loading: dsLoading,
    error: dsError,
    refreshDatasources,
  } = useDatasources()
  const {
    jobs,
    loading: jobsLoading,
    error: jobsError,
    refreshJobs,
  } = useJobs(currentGroup?.id)
  const [openFolders, setOpenFolders] = useState<Set<string>>(new Set())

  useEffect(() => {
    refreshDatasources()
    refreshJobs()
  }, [refreshKey])

  useEffect(() => {
    if (revealDatasourceId) {
      setOpenFolders((prev) => new Set(prev).add(revealDatasourceId!))
      onRevealHandled?.()
    }
  }, [revealDatasourceId])

  const loading = dsLoading || jobsLoading
  const error = dsError || jobsError

  const jobsByDatasource = new Map<string, Job[]>()
  for (const job of jobs) {
    const list = jobsByDatasource.get(job.datasource_id) ?? []
    list.push(job)
    jobsByDatasource.set(job.datasource_id, list)
  }

  function toggleFolder(id: string) {
    setOpenFolders((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  return (
    <Card
      size="sm"
      className="h-full bg-white shadow-sm ring-1 ring-[#f65d19]/30"
    >
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-[#f65d19]">
          <ListTree className="size-4" />
          Tree Structure
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-1 overflow-scroll">
        {loading && <p className="text-sm text-muted-foreground">Loading...</p>}
        {error && (
          <p className="text-sm text-red-500">Unable to load : {error}</p>
        )}
        {!loading &&
          !error &&
          datasources.map((ds) => {
            const isOpen = openFolders.has(ds.id)
            const dsJobs = jobsByDatasource.get(ds.id) ?? []

            return (
              <div key={ds.id}>
                <div
                  className="flex cursor-pointer items-center justify-between rounded-md px-2 py-1.5 transition-colors hover:bg-muted"
                  onClick={() => {
                    toggleFolder(ds.id)
                    onSelect(ds, "datasource")
                  }}
                >
                  <span className="flex items-center gap-2 truncate text-sm font-medium">
                    <FileSpreadsheet className="size-4 shrink-0 text-[#f65d19]" />
                    <span className="truncate">{ds.name}</span>
                  </span>
                  {dsJobs.length > 0 && (
                    <ChevronDown
                      className={cn(
                        "size-4 shrink-0 transition-transform",
                        isOpen && "rotate-180",
                      )}
                      onClick={(e) => {
                        e.stopPropagation()
                        toggleFolder(ds.id)
                      }}
                    />
                  )}
                </div>
                {isOpen &&
                  dsJobs.map((job) => (
                    <div
                      key={job.job_id}
                      className="ml-5 flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors hover:bg-muted"
                      onClick={() => onSelect(job, "job")}
                    >
                      <FileCog className="size-4 shrink-0 text-muted-foreground" />
                      <span className="truncate">{job.name}</span>
                      <span
                        className={cn(
                          "ml-auto shrink-0 rounded px-1.5 py-0.5 text-xs",
                          job.status === "done"
                            ? "bg-green-100 text-green-700"
                            : job.status === "error"
                              ? "bg-red-100 text-red-700"
                              : "bg-yellow-100 text-yellow-700",
                        )}
                      >
                        {job.status}
                      </span>
                    </div>
                  ))}
              </div>
            )
          })}
        {!loading && !error && datasources.length === 0 && (
          <p className="text-sm text-muted-foreground">No datasources found!</p>
        )}
        <Button
          className="rounded-lg! mt-1 w-full bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
          size="default"
          variant="outline"
          onClick={() => {
            onSelect("form", "form")
          }}
        >
          <Plus />
        </Button>
      </CardContent>
    </Card>
  )
}
