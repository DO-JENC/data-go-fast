import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { useAuth } from "@/hooks/use-auth"
import { useGroups } from "@/hooks/use-groups"
import { cn } from "@/lib/utils"
import type { Group } from "@/types/group"
import { Check, ChevronLeft, ChevronRight, Loader2, Plus } from "lucide-react"
import { useCallback, useEffect, useRef, useState } from "react"

type DialogMode = "join" | "create"

export function AppSidebar() {
  const {
    currentGroup,
    selectGroup,
    loadingGroup,
    groups,
    totalGroups,
    page,
    pageSize,
    setPage,
    joinGroup,
    createGroup,
    searchAvailableGroups,
  } = useGroups()
  const { user } = useAuth()

  const [dialogOpen, setDialogOpen] = useState(false)
  const [mode, setMode] = useState<DialogMode>("join")

  // Join mode state
  const [searchQuery, setSearchQuery] = useState("")
  const [searchResults, setSearchResults] = useState<Group[]>([])
  const [searchLoading, setSearchLoading] = useState(false)
  const [selectedResult, setSelectedResult] = useState<Group | null>(null)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Create mode state
  const [newGroupName, setNewGroupName] = useState("")
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const totalPages = Math.ceil(totalGroups / pageSize)

  // Debounced search for groups the user doesn't belong to
  useEffect(() => {
    if (mode !== "join") return
    if (debounceRef.current) clearTimeout(debounceRef.current)

    if (!searchQuery.trim()) {
      setSearchResults([])
      return
    }

    debounceRef.current = setTimeout(async () => {
      setSearchLoading(true)
      const results = await searchAvailableGroups(searchQuery)
      setSearchResults(results)
      setSearchLoading(false)
    }, 300)

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current)
    }
  }, [searchQuery, mode, searchAvailableGroups])

  const resetDialog = useCallback(() => {
    setSearchQuery("")
    setSearchResults([])
    setSelectedResult(null)
    setNewGroupName("")
    setError(null)
    setSubmitting(false)
  }, [])

  const handleOpenChange = (open: boolean) => {
    setDialogOpen(open)
    if (!open) resetDialog()
  }

  const handleJoin = async () => {
    if (!selectedResult) return
    setSubmitting(true)
    setError(null)
    try {
      await joinGroup(selectedResult.id)
      selectGroup(selectedResult)
      setDialogOpen(false)
      resetDialog()
    } catch {
      setError("Failed to join group. Please try again.")
    } finally {
      setSubmitting(false)
    }
  }

  const handleCreate = async () => {
    if (!newGroupName.trim()) return
    setSubmitting(true)
    setError(null)
    try {
      await createGroup(newGroupName.trim())
      setDialogOpen(false)
      resetDialog()
    } catch {
      setError("Failed to create group. Name may already be taken.")
    } finally {
      setSubmitting(false)
    }
  }

  const userInitials = user?.email ? user.email.slice(0, 2).toUpperCase() : "??"

  return (
    <>
      <Sidebar className="absolute top-0 left-0 h-full z-40 border-r bg-sidebar">
        <SidebarHeader>
          <div className="px-4 py-3">
            <p className="text-[11px] uppercase tracking-wider text-sidebar-foreground/40 mb-0.5">
              Workspace
            </p>
            <p className="font-medium text-sidebar-foreground truncate">
              {currentGroup?.name ?? "No group selected"}
            </p>
          </div>
        </SidebarHeader>

        <SidebarContent>
          <SidebarGroup>
            <div className="flex items-center justify-between pr-3">
              <SidebarGroupLabel className="text-[11px] uppercase tracking-wider">
                Groups
              </SidebarGroupLabel>
              <button
                onClick={() => setDialogOpen(true)}
                className="w-5 h-5 flex items-center justify-center rounded border border-sidebar-border text-sidebar-foreground/50 hover:text-sidebar-foreground hover:bg-sidebar-accent transition-colors"
                aria-label="Add or join a group"
              >
                <Plus className="w-3 h-3" />
              </button>
            </div>

            <SidebarGroupContent>
              <SidebarMenu className="gap-0.5 px-2">
                {loadingGroup ? (
                  <div className="flex items-center justify-center py-6">
                    <Loader2 className="w-4 h-4 animate-spin text-sidebar-foreground/40" />
                  </div>
                ) : groups.length === 0 ? (
                  <p className="text-xs text-sidebar-foreground/40 px-2 py-4 text-center">
                    No groups yet.{" "}
                    <button
                      onClick={() => setDialogOpen(true)}
                      className="underline hover:text-sidebar-foreground/70"
                    >
                      Create one
                    </button>
                  </p>
                ) : (
                  groups.map((group) => (
                    <SidebarMenuItem key={group.id}>
                      <button
                        onClick={() => selectGroup(group)}
                        className={cn(
                          "w-full flex items-center gap-2.5 px-2.5 py-2 rounded-md text-sm transition-colors text-left",
                          currentGroup?.id === group.id
                            ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
                            : "text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground",
                        )}
                      >
                        <span className="w-1.5 h-1.5 rounded-full bg-primary/60 flex-shrink-0" />
                        <span className="flex-1 truncate">{group.name}</span>
                        {currentGroup?.id === group.id && (
                          <Check className="w-3.5 h-3.5 flex-shrink-0 text-[var(--color-purple)]" />
                        )}
                      </button>
                    </SidebarMenuItem>
                  ))
                )}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between px-4 py-2 border-t border-sidebar-border">
            <button
              onClick={() => setPage(page - 1)}
              disabled={page <= 1}
              className="text-xs text-sidebar-foreground/50 hover:text-sidebar-foreground disabled:opacity-30 disabled:cursor-not-allowed flex items-center gap-1 transition-colors"
            >
              <ChevronLeft className="w-3.5 h-3.5" />
              Prev
            </button>
            <span className="text-xs text-sidebar-foreground/40">
              {page} / {totalPages}
            </span>
            <button
              onClick={() => setPage(page + 1)}
              disabled={page >= totalPages}
              className="text-xs text-sidebar-foreground/50 hover:text-sidebar-foreground disabled:opacity-30 disabled:cursor-not-allowed flex items-center gap-1 transition-colors"
            >
              Next
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>
        )}

        <SidebarFooter>
          <div className="flex items-center gap-2.5 px-4 py-3">
            <div className="w-7 h-7 rounded-full bg-primary/15 flex items-center justify-center text-[11px] font-medium text-[var(--color-purple)] flex-shrink-0">
              {userInitials}
            </div>
            <div className="min-w-0">
              <p className="text-sm font-medium text-sidebar-foreground truncate leading-tight">
                {user?.email ?? ""}
              </p>
            </div>
          </div>
        </SidebarFooter>
      </Sidebar>

      {/* Join / Create Dialog */}
      <Dialog open={dialogOpen} onOpenChange={handleOpenChange}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Add a group</DialogTitle>
            <DialogDescription>
              Join an existing group or create a new one.
            </DialogDescription>
          </DialogHeader>

          {/* Mode tabs */}
          <div className="flex gap-2">
            {(["join", "create"] as DialogMode[]).map((m) => (
              <button
                key={m}
                onClick={() => {
                  setMode(m)
                  setError(null)
                }}
                className={cn(
                  "text-sm px-4 py-1.5 rounded-full border transition-colors",
                  mode === m
                    ? "bg-primary/10 border-primary/30 text-[var(--color-purple)] font-medium"
                    : "border-border text-muted-foreground hover:text-foreground",
                )}
              >
                {m === "join" ? "Join" : "Create"}
              </button>
            ))}
          </div>

          {mode === "join" ? (
            <div className="space-y-3">
              <Input
                placeholder="Search groups…"
                value={searchQuery}
                onChange={(e) => {
                  setSearchQuery(e.target.value)
                  setSelectedResult(null)
                }}
                autoFocus
              />
              {searchLoading && (
                <div className="flex justify-center py-3">
                  <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
                </div>
              )}
              {!searchLoading && searchResults.length > 0 && (
                <div className="border border-border rounded-md overflow-hidden max-h-44 overflow-y-auto">
                  {searchResults.map((g) => (
                    <button
                      key={g.id}
                      onClick={() => setSelectedResult(g)}
                      className={cn(
                        "w-full text-left px-3 py-2 text-sm transition-colors",
                        selectedResult?.id === g.id
                          ? "bg-primary/10 text-[var(--color-purple)] font-medium"
                          : "hover:bg-accent text-foreground",
                      )}
                    >
                      {g.name}
                    </button>
                  ))}
                </div>
              )}
              {!searchLoading && searchQuery && searchResults.length === 0 && (
                <p className="text-sm text-muted-foreground text-center py-2">
                  No groups found.
                </p>
              )}
              {error && <p className="text-sm text-destructive">{error}</p>}
              <Button
                className="w-full rounded-md mt-3 bg-[var(--color-purple)] text-white"
                disabled={!selectedResult || submitting}
                onClick={handleJoin}
              >
                {submitting ? (
                  <Loader2 className="w-4 h-4 animate-spin mr-2" />
                ) : null}
                Join group
              </Button>
            </div>
          ) : (
            <div className="space-y-3">
              <Input
                placeholder="Group name"
                value={newGroupName}
                onChange={(e) => setNewGroupName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                autoFocus
              />
              {error && <p className="text-sm text-destructive">{error}</p>}
              <Button
                className="w-full rounded-md mt-3 bg-[var(--color-purple)] text-white"
                disabled={!newGroupName.trim() || submitting}
                onClick={handleCreate}
              >
                {submitting ? (
                  <Loader2 className="w-4 h-4 animate-spin mr-2" />
                ) : null}
                Create group
              </Button>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  )
}
