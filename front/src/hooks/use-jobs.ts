import { api } from "@/lib/api"
import type { Job } from "@/types/job"
import { useEffect, useState } from "react"

export function useJobs(groupId?: string) {
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  function buildUrl() {
    console.log(groupId)
    return groupId ? `/jobs?group_id=${groupId}` : "/jobs"
  }

  async function fetchJobs() {
    try {
      const data = await api.get<Job[]>(buildUrl())
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
      .get<Job[]>(buildUrl(), { signal: controller.signal })
      .then((data) => setJobs(data))
      .catch((err) => {
        if (err.name !== "AbortError")
          setError(err instanceof Error ? err.message : String(err))
      })
      .finally(() => setLoading(false))
    return () => controller.abort()
  }, [groupId])

  return { jobs, loading, error, refreshJobs: fetchJobs }
}
