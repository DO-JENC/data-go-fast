import type { Job } from "@/types/job"
import { useEffect, useState } from "react"

export function useJobs() {
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch("/api/jobs")
      .then((res) => {
        if (!res.ok) throw new Error(`Erreur ${res.status}`)
        return res.json() as Promise<Job[]>
      })
      .then(setJobs)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false))
  }, [])

  return { jobs, loading, error }
}
