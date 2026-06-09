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

  useEffect(() => {
    if (!currentGroup?.id) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setLoading(false)
      return
    }

    const offset = (page - 1) * limit
    setLoading(true)
    api
      .get<{ items: Datasource[]; total: number }>(
        `/datasources?group_id=${currentGroup.id}&limit=${limit}&offset=${offset}`,
      )
      .then((data) => {
        setDatasources(data.items)
        setTotal(data.total)
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false))
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
    total,
    page,
    setPage,
    limit,
  }
}
