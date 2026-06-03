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
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import type { Datasource } from "@/types/datasource"
import { ChevronDown, FileJson, FileSpreadsheet, Trash2 } from "lucide-react"
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
  onDelete,
}: {
  datasource: Datasource
  onDelete: (id: string) => Promise<boolean>
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
            <AlertDialog>
              <AlertDialogTrigger
                onClick={(e) => e.stopPropagation()}
                className="cursor-pointer text-muted-foreground transition-colors hover:text-red-600"
              >
                <Trash2 className="size-4" />
              </AlertDialogTrigger>
              <AlertDialogContent className="border-2 border-orange-500 ring-0 bg-white gap-2">
                <AlertDialogHeader>
                  <AlertDialogTitle>
                    Are you sure you want to delete this datasource?
                  </AlertDialogTitle>
                  <AlertDialogDescription className="text-center sm:text-center">
                    This action cannot be undone.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel onClick={(e) => e.stopPropagation()}>
                    Cancel
                  </AlertDialogCancel>
                  <AlertDialogAction
                    className="cursor-pointer bg-orange-500 text-white hover:bg-purple-600"
                    onClick={async (e) => {
                      e.stopPropagation()
                      try {
                        await onDelete(datasource.id)
                      } catch {
                        // error is handled by the hook
                      }
                    }}
                  >
                    Delete
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
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
