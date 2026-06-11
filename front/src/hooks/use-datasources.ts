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
    if (!currentGroup?.id) return
    try {
      const data = await api.get<{ items: Datasource[]; total: number }>("/datasources", {
        params: { group_id: currentGroup.id, limit: String(limit), offset: String((page - 1) * limit) }
      })
      setDatasources(data.items)
      setTotal(data.total)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (!currentGroup?.id) {
      setLoading(false)
      return
    }

    const controller = new AbortController()
    api
      .get<{ items: Datasource[]; total: number }>("/datasources", {
        signal: controller.signal,
        params: { group_id: currentGroup.id, limit: String(limit), offset: String((page - 1) * limit) }
      })
      .then((data) => {
        setDatasources(data.items)
        setTotal(data.total)
      })
      .catch((err) => {
        if (err.name !== "AbortError")
          setError(err instanceof Error ? err.message : String(err))
      })
      .finally(() => setLoading(false))
    return () => controller.abort()
  }, [currentGroup?.id, page, limit])

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

  return {
    datasources,
    loading,
    error,
    removeDatasource,
    refreshDatasources: fetchDatasources,
  }
}
