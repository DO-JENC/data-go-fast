import { Footer } from "@/components/layout/footer"
import { Header } from "@/components/layout/header"
import { Toaster } from "@/components/ui/sonner"
import { useAuth } from "@/hooks/use-auth"
import Datasources from "@/pages/Datasources"
import Login from "@/pages/Login"
import Signup from "@/pages/Signup"
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom"
import "./App.css"

function ProtectedRoute({ children }: { children: React.ReactNode }) {
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

  return <>{children}</>
}

function PublicRoute({ children }: { children: React.ReactNode }) {
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

  return <>{children}</>
}

function App() {
  return (
    <BrowserRouter>
      <Header />
      <main className="flex min-h-0 flex-1 flex-col bg-[var(--bg)]">
        <Routes>
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <Datasources />
              </ProtectedRoute>
            }
          />
          <Route
            path="/signup"
            element={
              <PublicRoute>
                <Signup />
              </PublicRoute>
            }
          />
          <Route
            path="/login"
            element={
              <PublicRoute>
                <Login />
              </PublicRoute>
            }
          />
          <Route
            path="/datasources"
            element={
              <ProtectedRoute>
                <Datasources />
              </ProtectedRoute>
            }
          />
        </Routes>
      </main>
      <Toaster />
      <Footer />
    </BrowserRouter>
  )
}

export default App
