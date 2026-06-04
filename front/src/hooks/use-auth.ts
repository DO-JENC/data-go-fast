import type { User } from "@/types/user"
import { useEffect, useState } from "react"

interface UserProps {
  email: string
  password: string
}

export function useAuth() {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState<boolean>(false)
  const [initialFetchLoading, setInitialFetchLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch("/api/users/me")
      .then((res) => {
        if (!res.ok) throw new Error("Not authenticated")
        return res.json() as Promise<User>
      })
      .then((currentUser) => setUser(currentUser))
      .catch(() => setUser(null))
      .finally(() => setInitialFetchLoading(false))
  }, [])

  const signup = async ({ email, password }: UserProps) => {
    setLoading(true)
    setError(null)

    try {
      const res = await fetch("/api/users/signup", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email, password }),
      })

      if (!res.ok) {
        throw new Error(`Error: ${res.status} - Failed to sign up`)
      }

      const createdUser = (await res.json()) as User
      setUser(createdUser)
      return createdUser
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError(err.message || "Something went wrong")
      }
      return null
    } finally {
      setLoading(false)
    }
  }

  return {
    user,
    signup,
    loading,
    initialFetchLoading,
    error,
  }
}
