import { useState } from "react"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"

export default function DatasourceForm({
  onRefresh,
}: {
  onRefresh?: () => void
}) {
  const [file, setFile] = useState<File | null>(null)
  const [hasHeader, setHasHeader] = useState(true)
  const [uploading, setUploading] = useState(false)

  const isCsv = file?.name.toLowerCase().endsWith(".csv")

  async function handleSubmit(e: React.SubmitEvent) {
    e.preventDefault()
    if (!file) return

    if (isCsv && !hasHeader) {
      toast.error("Headerless CSV files are not yet supported")
      return
    }

    setUploading(true)
    try {
      const fd = new FormData()
      fd.append("file", file)
      fd.append(
        "metadata",
        JSON.stringify({
          type: isCsv ? "csv" : "json",
          header: hasHeader,
        }),
      )

      const res = await fetch("/api/datasources", { method: "POST", body: fd })
      if (!res.ok) throw new Error(await res.text())

      toast.success("Datasource uploaded successfully")
      onRefresh?.()
      setFile(null)
      setHasHeader(true)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Upload failed")
    } finally {
      setUploading(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="flex w-full flex-col gap-4 p-4">
      <div className="flex flex-col gap-1.5">
        <label htmlFor="ds-file" className="text-sm font-medium text-gray-700">
          Choose a CSV or JSON file
        </label>
        <Input
          id="ds-file"
          type="file"
          accept=".csv,.json"
          onChange={(e) => setFile(e.target.files?.[0] ?? null)}
        />
      </div>

      {isCsv && (
        <label className="flex cursor-pointer items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={hasHeader}
            onChange={(e) => setHasHeader(e.target.checked)}
            className="size-4 rounded border-gray-300 text-[#8828ad] accent-[#8828ad]"
          />
          Has header row
        </label>
      )}

      <Button
        type="submit"
        disabled={!file || uploading}
        className="w-full bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
      >
        {uploading ? "Uploading..." : "Upload"}
      </Button>
    </form>
  )
}
