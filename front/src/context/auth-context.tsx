import { api } from "@/lib/api"
import type { User } from "@/types/user"
import type { ReactNode } from "react"
import { useCallback, useEffect, useState } from "react"
import {
  AuthContext,
  type AuthContextType,
  type AuthResponse,
  type UserProps,
} from "./auth-context-base"

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState<boolean>(false)
  const [initialFetchLoading, setInitialFetchLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)

  const fetchUser = useCallback(async () => {
    try {
      const currentUser = (await api.get("/users/me")) as User
      setUser(currentUser)
      return currentUser
    } catch {
      setUser(null)
      api.clearToken()
      return null
    } finally {
      setInitialFetchLoading(false)
    }
  }, [])

  useEffect(() => {
    const initAuth = async () => {
      const token = api.getToken()
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
      const data = (await api.post("/auth/login", {
        email,
        password,
      })) as AuthResponse
      api.setToken(data.access_token)
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
      api.setToken(data.access_token)
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

  const logout = async () => {
    try {
      await api.post("/auth/logout")
    } catch {
      // Ignore logout errors
    } finally {
      api.clearToken()
      setUser(null)
    }
  }

  const value: AuthContextType = {
    user,
    loading,
    initialFetchLoading,
    error,
    login,
    signup,
    logout,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}
