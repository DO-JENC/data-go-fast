import { useGroups } from "@/hooks/use-groups"
import { api } from "@/lib/api"
import type { Datasource } from "@/types/datasource"
import { useCallback, useEffect, useState } from "react"

export function useDatasources(initialPage = 1, limit = 10) {
  const { currentGroup, loadingGroup } = useGroups()
  const [datasources, setDatasources] = useState<Datasource[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [total, setTotal] = useState(0)
  const [page, setPage] = useState(initialPage)

  const fetchDatasources = useCallback(
    async (signal?: AbortSignal) => {
      // Wait for GroupProvider to finish resolving before attempting any fetch
      if (loadingGroup) return
      if (!currentGroup?.id) {
        setLoading(false)
        return
      }

      setLoading(true)
      setError(null)

      try {
        const data = await api.get<{ items: Datasource[]; total: number }>(
          "/datasources",
          {
            signal,
            params: {
              group_id: currentGroup.id,
              limit: String(limit),
              offset: String((page - 1) * limit),
            },
          },
        )
        setDatasources(data.items)
        setTotal(data.total)
      } catch (err) {
        if (err instanceof Error && err.name === "AbortError") return
        setError(err instanceof Error ? err.message : String(err))
      } finally {
        setLoading(false)
      }
    },
    // loadingGroup is a dep so the effect re-runs once the group is resolved
    [currentGroup, loadingGroup, page, limit],
  )

  const downloadDatasource = useCallback(async (id: string) => {
    const token = api.getToken()
    const response = await fetch(`/api/datasources/${id}/download`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      credentials: "include",
    })
    if (!response.ok) {
      throw new Error(`Erreur lors du téléchargement (${response.status})`)
    }
    const blob = await response.blob()
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = `datasource-${id}`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, [])

  useEffect(() => {
    const controller = new AbortController()
    // eslint-disable-next-line react-hooks/set-state-in-effect
    fetchDatasources(controller.signal)
    return () => controller.abort()
  }, [fetchDatasources])

  async function removeDatasource(id: string): Promise<boolean> {
    try {
      await api.delete(`/datasources/${id}`)
      setDatasources((prev) => prev.filter((ds) => ds.id !== id))
      setTotal((prev) => prev - 1)
      return true
    } catch (err) {
      throw new Error(
        err instanceof Error ? err.message : "Erreur lors de la suppression",
        { cause: err },
      )
    }
  }

  return {
    datasources,
    // Treat datasource loading as pending while the group itself is still loading
    loading: loading || loadingGroup,
    error,
    total,
    page,
    setPage,
    totalPages: Math.ceil(total / limit),
    removeDatasource,
    downloadDatasource,
    refreshDatasources: () => fetchDatasources(),
  }
}
