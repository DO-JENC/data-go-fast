import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { Link } from "react-router-dom"

const navItems = [{ title: "Datasources", url: "/datasources" }]

export function AppSidebar() {
  return (
    <Sidebar className="absolute top-0 left-0 h-full z-40 border-r bg-sidebar">
      <SidebarHeader>
        <span className="px-4 py-3 font-semibold text-sidebar-foreground">
          My App
        </span>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Navigation</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton>
                    <Link to={item.url}>{item.title}</Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter>
        <span className="px-4 py-3 text-sm text-sidebar-foreground/60">
          v1.0.0
        </span>
      </SidebarFooter>
    </Sidebar>
  )
}
