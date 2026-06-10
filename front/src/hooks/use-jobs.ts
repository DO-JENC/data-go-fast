import type { Job } from "@/types/job"
import { useEffect, useState } from "react"

export function useJobs() {
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  async function fetchJobs() {
    try {
      const res = await fetch("/api/jobs")
      if (!res.ok) throw new Error(`Erreur ${res.status}`)
      setJobs(await res.json())
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    const controller = new AbortController()
    fetch("/api/jobs", { signal: controller.signal })
      .then((res) => {
        if (!res.ok) throw new Error(`Erreur ${res.status}`)
        return res.json()
      })
      .then((data) => setJobs(data))
      .catch((err) => {
        if (err.name !== "AbortError")
          setError(err instanceof Error ? err.message : String(err))
      })
      .finally(() => setLoading(false))
    return () => controller.abort()
  }, [])

  return { jobs, loading, error, refreshJobs: fetchJobs }
}
