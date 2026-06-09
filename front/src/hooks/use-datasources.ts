import { useGroups } from "@/hooks/use-groups"
import { api } from "@/lib/api"
import type { Datasource } from "@/types/datasource"
import { useEffect, useState } from "react"

export function useDatasources() {
  const { currentGroup } = useGroups()
  const [datasources, setDatasources] = useState<Datasource[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!currentGroup?.id) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setLoading(false)
      return
    }

    setLoading(true)
    api
      .get<Datasource[]>(`/datasources?group_id=${currentGroup.id}`)
      .then(setDatasources)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false))
  }, [currentGroup?.id])

  async function removeDatasource(id: string): Promise<boolean> {
    try {
      await api.delete(`/datasources/${id}`)
      setDatasources((prev) => prev.filter((ds) => ds.id !== id))
      return true
    } catch (err: unknown) {
      throw new Error(
        err instanceof Error ? err.message : "Erreur lors de la suppression",
        { cause: err },
      )
    }
  }

  return { datasources, loading, error, removeDatasource }
}
