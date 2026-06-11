import { useAuth } from "@/hooks/use-auth"
import { api } from "@/lib/api"
import type { Group } from "@/types/group"
import type { Member } from "@/types/member"
import type { ReactNode } from "react"
import { useCallback, useEffect, useState } from "react"
import { GroupContext } from "./group-context-base"

const PAGE_SIZE = 5

export function GroupProvider({ children }: { children: ReactNode }) {
  const { user, initialFetchLoading } = useAuth()

  const [groups, setGroups] = useState<Group[]>([])
  const [totalGroups, setTotalGroups] = useState(0)
  const [page, setPageState] = useState(1)
  const [loadingGroup, setLoadingGroup] = useState(true)

  const [currentGroup, setCurrentGroup] = useState<Group | null>(() => {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem("current_group_id")
      // Store only the id; resolve to full object after fetch
      return saved ? ({ id: saved } as Group) : null
    }
    return null
  })

  const selectGroup = useCallback((group: Group | null) => {
    setCurrentGroup(group)
    if (group) {
      localStorage.setItem("current_group_id", group.id)
    } else {
      localStorage.removeItem("current_group_id")
    }
  }, [])

  const getGroups = useCallback(async (targetPage = 1) => {
    try {
      setLoadingGroup(true)
      const res = await api.get<{ groups: Group[]; total: number }>(
        `/groups?page=${targetPage}&pageSize=${PAGE_SIZE}`,
      )
      setGroups(res.groups)
      setTotalGroups(res.total)
      setPageState(targetPage)

      // Resolve the current group from the fresh list (fixes stale localStorage)
      setCurrentGroup((prev) => {
        if (!prev) {
          return res.groups[0] ?? null
        }
        const resolved = res.groups.find((g) => g.id === prev.id)
        return resolved ?? res.groups[0] ?? null
      })
    } catch {
      setGroups([])
      setTotalGroups(0)
    } finally {
      setLoadingGroup(false)
    }
  }, [])

  const setPage = useCallback(
    (p: number) => {
      getGroups(p)
    },
    [getGroups],
  )

  const joinGroup = useCallback(
    async (groupId: string) => {
      await api.post(`/groups/${groupId}/join`, {})
      await getGroups(1) // refresh from page 1
    },
    [getGroups],
  )

  const fetchGroupMembers = useCallback(
    async (groupId: string): Promise<Member[]> => {
      try {
        return await api.get<Member[]>(`/groups/${groupId}/members`)
      } catch {
        return []
      }
    },
    [],
  )

  const createGroup = useCallback(
    async (name: string): Promise<Group> => {
      const group = await api.post<Group>("/groups", { name })
      await getGroups(1)
      selectGroup(group)
      return group
    },
    [getGroups, selectGroup],
  )

  const searchAvailableGroups = useCallback(
    async (query: string): Promise<Group[]> => {
      if (!query.trim()) return []
      try {
        return await api.get<Group[]>(
          `/groups/search?q=${encodeURIComponent(query)}&exclude_mine=true`,
        )
      } catch {
        return []
      }
    },
    [],
  )

  // Coordinate with auth lifecycle
  useEffect(() => {
    if (initialFetchLoading) {
      return
    }
    if (!user) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      selectGroup(null)
      setGroups([])
      setTotalGroups(0)
      setLoadingGroup(false)
    } else {
      getGroups(1)
    }
  }, [user, initialFetchLoading, getGroups, selectGroup])

  return (
    <GroupContext.Provider
      value={{
        currentGroup,
        loadingGroup,
        selectGroup,
        groups,
        totalGroups,
        page,
        pageSize: PAGE_SIZE,
        setPage,
        getGroups,
        joinGroup,
        createGroup,
        searchAvailableGroups,
        fetchGroupMembers,
      }}
    >
      {children}
    </GroupContext.Provider>
  )
}
