import { Footer } from "@/components/layout/footer"
import { Header } from "@/components/layout/header"
import { Toaster } from "@/components/ui/sonner"
import Datasources from "@/pages/Datasources"
import Login from "@/pages/Login"
import Signup from "@/pages/Signup"
import ReactDOM from "react-dom"
import {
  BrowserRouter,
  Navigate,
  Outlet,
  Route,
  Routes,
} from "react-router-dom"
import "./App.css"
import { AppSidebar } from "./components/layout/app-sidebar"
import { SidebarProvider, SidebarTrigger } from "./components/ui/sidebar"
import { AuthProvider } from "./context/auth-context"
import { useAuth } from "./hooks/use-auth"

/**
 * Layout component for routes that require authentication.
 */
function ProtectedLayout() {
  const { user, initialFetchLoading } = useAuth()

  if (initialFetchLoading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <p className="text-sm text-muted-foreground">Loading...</p>
      </div>
    )
  }

  if (!user) {
    return <Navigate to="/login" replace />
  }

  const portalTarget = document.getElementById("sidebar-trigger-portal")

  return (
    <SidebarProvider>
      {/* Teleport the trigger into the header, next to the logo */}
      {portalTarget && ReactDOM.createPortal(<SidebarTrigger />, portalTarget)}
      <div className="relative flex w-full min-h-[calc(100vh-120px)] z-0">
        <AppSidebar />
        {/* Main content layout */}
        <main className="flex flex-1 flex-col bg-[var(--bg)] p-6 w-full">
          <Outlet />
        </main>
      </div>
    </SidebarProvider>
  )
}

/**
 * Layout component for routes accessible when NOT authenticated (Login/Signup).
 */
function PublicLayout() {
  const { user, initialFetchLoading } = useAuth()

  if (initialFetchLoading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <p className="text-sm text-muted-foreground">Loading...</p>
      </div>
    )
  }

  if (user) {
    return <Navigate to="/datasources" replace />
  }

  return (
    <main className="flex flex-1 flex-col bg-[var(--bg)] p-6">
      <Outlet />
    </main>
  )
}

function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        {/* Global Header */}
        <Header />

        <div className="flex min-h-0 flex-1 flex-col">
          <Routes>
            {/* Protected Routes Group */}
            <Route element={<ProtectedLayout />}>
              <Route path="/" element={<Datasources />} />
              <Route path="/datasources" element={<Datasources />} />
            </Route>

            {/* Public Routes Group */}
            <Route element={<PublicLayout />}>
              <Route path="/signup" element={<Signup />} />
              <Route path="/login" element={<Login />} />
            </Route>
          </Routes>
        </div>

        {/* Global UI & Footer Elements */}
        <Toaster />
        <Footer />
      </BrowserRouter>
    </AuthProvider>
  )
}

export default App
