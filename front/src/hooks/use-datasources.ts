import { api } from "@/lib/api"
import type { Datasource } from "@/types/datasource"
import { useEffect, useState } from "react"

export function useDatasources() {
  const [datasources, setDatasources] = useState<Datasource[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api
      .get<Datasource[]>("/datasources")
      .then(setDatasources)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false))
  }, [])

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
