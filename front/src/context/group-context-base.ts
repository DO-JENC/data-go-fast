import type { Group } from "@/types/group"
import type { Member } from "@/types/member"
import { createContext } from "react"

export interface GroupContextType {
  // Current selection
  currentGroup: Group | null
  loadingGroup: boolean
  selectGroup: (group: Group | null) => void

  // Paginated list of user's groups
  groups: Group[]
  totalGroups: number
  page: number
  pageSize: number
  setPage: (page: number) => void
  getGroups: (page?: number) => Promise<void>

  // Mutations
  joinGroup: (groupId: string) => Promise<void>
  createGroup: (name: string) => Promise<Group>

  // Search (groups user doesn't belong to)
  searchAvailableGroups: (query: string) => Promise<Group[]>

  fetchGroupMembers: (groupId: string) => Promise<Member[]>
}

export const GroupContext = createContext<GroupContextType | undefined>(
  undefined,
)
