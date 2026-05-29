import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import type { Datasource } from "@/types/datasource"
import { ChevronDown, FileJson, FileSpreadsheet } from "lucide-react"
import { useState } from "react"

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

function formatDate(iso: string | null): string {
  if (!iso) return "—"
  return new Date(iso).toLocaleDateString("fr-FR", {
    day: "numeric",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

function FileTypeIcon({ type }: { type: string | null }) {
  if (type === "csv") return <FileSpreadsheet className="size-4" />
  if (type === "json") return <FileJson className="size-4" />
  return null
}

export default function DatasourceCard({
  datasource,
}: {
  datasource: Datasource
}) {
  const [open, setOpen] = useState(false)

  return (
    <Card
      size="sm"
      className="cursor-pointer bg-white shadow-sm transition-shadow hover:shadow-md"
      onClick={() => setOpen((prev) => !prev)}
    >
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <span className="flex items-center gap-2 truncate">
            <FileTypeIcon type={datasource.file_type} />
            <span className="truncate">{datasource.name}</span>
          </span>
          <span className="flex items-center gap-3 text-sm font-normal text-muted-foreground">
            <span>{formatSize(datasource.size)}</span>
            {datasource.file_type && (
              <span
                className={cn(
                  "rounded px-1.5 py-0.5 text-xs uppercase",
                  open ? "bg-muted" : "border text-[#8828ad] border-[#8828ad]",
                )}
              >
                {datasource.file_type}
              </span>
            )}
            <ChevronDown
              className={cn(
                "size-4 transition-transform",
                open && "rotate-180",
              )}
            />
          </span>
        </CardTitle>
      </CardHeader>
      {open && (
        <CardContent className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
          <span className="text-muted-foreground">ID</span>
          <span className="truncate">{datasource.id}</span>
          <span className="text-muted-foreground">S3 ID</span>
          <span className="truncate">{datasource.s3_id}</span>
          <span className="text-muted-foreground">Créé le</span>
          <span>{formatDate(datasource.created_at)}</span>
          <span className="text-muted-foreground">Groupe</span>
          <span className="truncate">{datasource.group_id ?? "—"}</span>
        </CardContent>
      )}
    </Card>
  )
}
