import { AuthContext } from "@/context/auth-context-base"
import { useContext } from "react"

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider")
  }
  return context
}
