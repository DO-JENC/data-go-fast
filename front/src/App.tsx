import { Footer } from "@/components/layout/footer"
import { Header } from "@/components/layout/header"
import Datasources from "@/pages/Datasources"
import Signup from "@/pages/Signup.tsx"
import { BrowserRouter, Route, Routes } from "react-router-dom"
import "./App.css"

function App() {
  return (
    <BrowserRouter>
      <Header />
      <main className="flex min-h-0 flex-1 flex-col bg-[var(--bg)]">
        <Routes>
          <Route path="/" element={<Datasources />} />
          <Route path="/signup" element={<Signup />} />
          <Route path="/datasources" element={<Datasources />} />
        </Routes>
      </main>
      <Footer />
    </BrowserRouter>
  )
}

export default App
