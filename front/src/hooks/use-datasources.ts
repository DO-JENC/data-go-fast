import { useGroups } from "@/hooks/use-groups"
import { api } from "@/lib/api"
import type { Datasource } from "@/types/datasource"
import { useEffect, useState } from "react"

export function useDatasources(initialPage = 1, limit = 10) {
  const { currentGroup } = useGroups()
  const [datasources, setDatasources] = useState<Datasource[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [total, setTotal] = useState(0)
  const [page, setPage] = useState(initialPage)

  async function fetchDatasources() {
    setLoading(true)
    setError(null)
    try {
      const res = await fetch("/api/datasources") // `/datasources?group_id=${currentGroup.id}&limit=${limit}&offset=${offset}`
      if (!res.ok) throw new Error(`Erreur ${res.status}`)
      setDatasources(await res.json())
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { fetchDatasources() }, [])

  async function removeDatasource(id: string): Promise<boolean> {
    try {
      await api.delete(`/datasources/${id}`)
      setDatasources((prev) => prev.filter((ds) => ds.id !== id))
      setTotal((prev) => prev - 1)
      return true
    } catch (err: unknown) {
      throw new Error(
        err instanceof Error ? err.message : "Erreur lors de la suppression",
        { cause: err },
      )
    }
  }

  return { datasources, loading, error, removeDatasource, refreshDatasources: fetchDatasources }
}
