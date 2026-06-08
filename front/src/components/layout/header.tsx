import { useAuth } from "@/hooks/use-auth"
import "@coreui/coreui/dist/css/coreui.min.css"
import {
  CContainer,
  CHeader,
  CHeaderBrand,
  CHeaderNav,
  CNavItem,
  CNavLink,
} from "@coreui/react"
import { LogOut } from "lucide-react"
import { Link, useNavigate } from "react-router-dom"

export function Header() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()

  const handleLogout = (e: React.MouseEvent) => {
    e.preventDefault()
    logout()
    navigate("/login")
  }

  return (
    <CHeader className="py-1">
      <CContainer fluid>
        <Link to="/" className="flex items-center gap-3 no-underline!">
          <CHeaderBrand className="flex items-center gap-3 m-0">
            <img
              src="/public/logo.png"
              alt="Data-go-fast logo"
              className="max-h-10"
            />
            <span className="font-heading text-xl font-bold text-[#8828ad]">
              data-go-fast
            </span>
          </CHeaderBrand>
        </Link>

        <CHeaderNav>
          {user && (
            <CNavItem>
              <CNavLink
                href="#"
                onClick={handleLogout}
                title="Logout"
                className="cursor-pointer"
              >
                <LogOut className="h-5 w-5 text-[#8828ad] transition-opacity hover:opacity-80" />
              </CNavLink>
            </CNavItem>
          )}
        </CHeaderNav>
      </CContainer>
    </CHeader>
  )
}
