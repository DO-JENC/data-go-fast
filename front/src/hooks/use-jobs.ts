import { api } from "@/lib/api"
import type { Job } from "@/types/job"
import { useEffect, useState } from "react"

export function useJobs() {
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  async function fetchJobs() {
    try {
      const data = await api.get<Job[]>("/jobs")
      setJobs(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    const controller = new AbortController()
    api
      .get<Job[]>("/jobs", { signal: controller.signal })
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
