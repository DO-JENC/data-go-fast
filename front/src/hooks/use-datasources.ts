import type { Datasource } from "@/types/datasource"
import { useEffect, useState } from "react"

export function useDatasources() {
  const [datasources, setDatasources] = useState<Datasource[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch("/api/datasources")
      .then((res) => {
        if (!res.ok) throw new Error(`Erreur ${res.status}`)
        return res.json() as Promise<Datasource[]>
      })
      .then(setDatasources)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false))
  }, [])

  return { datasources, loading, error }
}
