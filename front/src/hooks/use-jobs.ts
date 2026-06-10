import type { Job } from "@/types/job"
import { useEffect, useState } from "react"

export function useJobs() {
  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  async function fetchJobs() {
    setLoading(true)
    setError(null)
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

  useEffect(() => { fetchJobs() }, [])

  return { jobs, loading, error, refreshJobs: fetchJobs }
}
