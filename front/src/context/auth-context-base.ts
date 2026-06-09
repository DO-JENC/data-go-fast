import type { User } from "@/types/user"
import { createContext } from "react"

export interface UserProps {
  email: string
  password: string
}

export interface AuthResponse {
  access_token: string
  token_type: string
}

export interface AuthContextType {
  user: User | null
  loading: boolean
  initialFetchLoading: boolean
  error: string | null
  login: (props: UserProps) => Promise<User | null>
  signup: (props: UserProps) => Promise<User | null>
  logout: () => Promise<void>
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined)
