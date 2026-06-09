import { Footer } from "@/components/layout/footer"
import { Header } from "@/components/layout/header"
import { Toaster } from "@/components/ui/sonner"
import Datasources from "@/pages/Datasources"
import Login from "@/pages/Login"
import Signup from "@/pages/Signup"
import {
  BrowserRouter,
  Navigate,
  Outlet,
  Route,
  Routes,
} from "react-router-dom"
import "./App.css"
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

  return <Outlet />
}

/**
 * Layout component for routes that should only be accessible when NOT authenticated (e.g., Login/Signup).
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

  return <Outlet />
}

function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Header />
        <main className="flex min-h-0 flex-1 flex-col bg-[var(--bg)]">
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
        </main>
        <Toaster />
        <Footer />
      </BrowserRouter>
    </AuthProvider>
  )
}

export default App
