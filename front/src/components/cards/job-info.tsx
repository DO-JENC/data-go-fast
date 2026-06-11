import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import type { Job, PipelineOp } from "@/types/job"
import { ArrowRight, FileCog } from "lucide-react"

function renderPipelineOp(op: PipelineOp, i: number) {
  switch (op.op) {
    case "ingest":
      return (
        <div key={i} className="flex items-center gap-2 text-sm">
          <ArrowRight className="size-3 text-muted-foreground" />
          <span className="text-muted-foreground">Ingest</span>
          <span className="rounded border px-1.5 py-0.5 text-xs uppercase">
            {op.type}
          </span>
          <span className="text-xs text-muted-foreground">
            header: {op.header ? "Yes" : "No"}
          </span>
        </div>
      )
    case "filter":
      return (
        <div key={i} className="flex items-center gap-2 text-sm">
          <ArrowRight className="size-3 text-muted-foreground" />
          <span className="text-muted-foreground">Filter</span>
          <code className="rounded bg-muted px-1 py-0.5 text-xs">
            {op.column} {op.operator} {String(op.value)}
          </code>
        </div>
      )
    case "aggregate":
      return (
        <div key={i} className="flex items-center gap-2 text-sm">
          <ArrowRight className="size-3 text-muted-foreground" />
          <span className="text-muted-foreground">Aggregate</span>
          <span className="text-xs text-muted-foreground">columns:</span>
          <code className="rounded bg-muted px-1 py-0.5 text-xs">
            {op.columns.join(", ")}
          </code>
          <span className="text-xs text-muted-foreground">fns:</span>
          <code className="rounded bg-muted px-1 py-0.5 text-xs">
            {op.functions.join(", ")}
          </code>
        </div>
      )
    case "group_by":
      return (
        <div key={i} className="flex items-center gap-2 text-sm">
          <ArrowRight className="size-3 text-muted-foreground" />
          <span className="text-muted-foreground">Group by</span>
          <code className="rounded bg-muted px-1 py-0.5 text-xs">{op.by}</code>
          <span className="text-xs text-muted-foreground">
            {op.aggregate.function}({op.aggregate.column})
          </span>
        </div>
      )
  }
}

export default function JobInfo({ job }: { job: Job }) {
  return (
    <Card
      size="sm"
      className="h-full bg-white shadow-sm transition-shadow hover:shadow-md"
    >
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileCog className="size-4 " />
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
        </CardTitle>
      </CardHeader>
      <CardContent className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
        <span className="text-muted-foreground">Job ID</span>
        <span className="truncate">{job.job_id}</span>
        <span className="text-muted-foreground">Datasource ID</span>
        <span className="truncate">{job.datasource_id}</span>
        {job.result_datasource_id && (
          <>
            <span className="text-muted-foreground">Result ID</span>
            <span className="truncate">{job.result_datasource_id}</span>
          </>
        )}
        {job.pipeline.length > 0 && (
          <>
            <span className="text-muted-foreground self-start">Pipeline</span>
            <div className="flex flex-col gap-1">
              {job.pipeline.map((op, i) => renderPipelineOp(op, i))}
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
