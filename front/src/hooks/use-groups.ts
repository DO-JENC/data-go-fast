import { GroupContext } from "@/context/group-context-base"
import { useContext } from "react"

export function useGroups() {
  const context = useContext(GroupContext)
  if (context === undefined) {
    throw new Error("useGroup must be used within a GroupProvider")
  }
  return context
}
