import { api } from "@/lib/api"
import type { User } from "@/types/user"
import { useCallback, useEffect, useState } from "react"

interface UserProps {
  email: string
  password: string
}

interface AuthResponse {
  access_token: string
  refresh_token: string
  token_type: string
}

export function useAuth() {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState<boolean>(false)
  const [initialFetchLoading, setInitialFetchLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)

  const fetchUser = useCallback(async () => {
    try {
      const currentUser = await api.get<User>("/users/me")
      setUser(currentUser)
      return currentUser
    } catch {
      setUser(null)
      localStorage.removeItem("access_token")
      return null
    } finally {
      setInitialFetchLoading(false)
    }
  }, [])

  useEffect(() => {
    const initAuth = async () => {
      const token = localStorage.getItem("access_token")
      if (token) {
        await fetchUser()
      } else {
        setInitialFetchLoading(false)
      }
    }
    void initAuth()
  }, [fetchUser])

  const login = async ({ email, password }: UserProps) => {
    setLoading(true)
    setError(null)
    try {
      const data = await api.post<AuthResponse>("/auth/login", {
        email,
        password,
      })
      localStorage.setItem("access_token", data.access_token)
      return await fetchUser()
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError(err.message || "Something went wrong")
      }
      return null
    } finally {
      setLoading(false)
    }
  }

  const signup = async ({ email, password }: UserProps) => {
    setLoading(true)
    setError(null)

    try {
      const data = await api.post<AuthResponse>("/auth/signup", {
        email,
        password,
      })
      localStorage.setItem("access_token", data.access_token)
      return await fetchUser()
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError(err.message || "Something went wrong")
      }
      return null
    } finally {
      setLoading(false)
    }
  }

  const logout = () => {
    localStorage.removeItem("access_token")
    setUser(null)
  }

  return {
    user,
    signup,
    login,
    logout,
    loading,
    initialFetchLoading,
    error,
  }
}
