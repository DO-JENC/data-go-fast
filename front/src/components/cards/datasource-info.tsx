import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { Datasource } from "@/types/datasource"
import { AlertDialog as AlertDialogPrimitive } from "@base-ui/react/alert-dialog"
import { FileJson, FileSpreadsheet, Trash2 } from "lucide-react"

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

function formatSizeMB(mb: number): string {
  if (mb < 1) return `${(mb * 1024).toFixed(0)} KiB`
  if (mb < 1024) return `${mb.toFixed(1)} MiB`
  return `${(mb / 1024).toFixed(1)} GiB`
}

export default function DatasourceInfo({
  item,
  onDelete,
  onRefresh,
}: {
  item: Datasource
  onDelete?: (id: string) => Promise<boolean>
  onRefresh?: () => void
}) {
  const ds = item as Datasource
  const FileTypeIcon = ({ type: t }: { type: string | null }) => {
    console.log(t)
    if (t?.toLowerCase() === "csv")
      return <FileSpreadsheet className="size-4 text-[#f65d19]" />
    if (t?.toLowerCase() === "json")
      return <FileJson className="size-4 text-[#f65d19]" />
    return null
  }

  return (
    <Card
      size="sm"
      className="h-full bg-white shadow-sm transition-shadow hover:shadow-md"
    >
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <span className="flex items-center gap-2 truncate">
            <FileTypeIcon type={ds.file_type} />
            <span className="truncate">{ds.name}</span>
          </span>
          <span className="flex items-center gap-3 text-sm font-normal text-muted-foreground">
            <span>{formatSizeMB(ds.size)}</span>
            {ds.file_type && (
              <span className="rounded border border-[#8828ad] px-1.5 py-0.5 text-xs uppercase text-[#8828ad]">
                {ds.file_type}
              </span>
            )}
            {onDelete && (
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
                    <AlertDialogPrimitive.Close
                      render={
                        <Button
                          className="cursor-pointer bg-orange-500 text-white hover:bg-purple-600"
                          onClick={async (e) => {
                            e.stopPropagation()
                            try {
                              await onDelete(ds.id)
                              onRefresh?.()
                            } catch {
                              // error handled by the hook
                            }
                          }}
                        >
                          Delete
                        </Button>
                      }
                    />
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            )}
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
        <span className="text-muted-foreground">ID</span>
        <span className="truncate">{ds.id}</span>
        <span className="text-muted-foreground">S3 ID</span>
        <span className="truncate">{ds.s3_id}</span>
        <span className="text-muted-foreground">Créé le</span>
        <span>{formatDate(ds.created_at)}</span>
        <span className="text-muted-foreground">Groupe</span>
        <span className="truncate">{ds.group_id ?? "—"}</span>
      </CardContent>
    </Card>
  )
}
