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

export function Header() {
  const { user } = useAuth()

  return (
    <CHeader className="py-1">
      <CContainer fluid>
        <CHeaderBrand href="#" className="flex items-center gap-3">
          <img
            src="/public/logo.png"
            alt="Data-go-fast logo"
            className="max-h-10"
          />
          <span className="font-heading text-xl font-bold text-[#8828ad]">
            data-go-fast
          </span>
        </CHeaderBrand>

        <CHeaderNav>
          {user && (
            <CNavItem>
              {/* TODO: Logooout behaviour */}
              <CNavLink href="#">
                <LogOut className="h-5 w-5 text-[#8828ad] transition-opacity hover:opacity-80" />
              </CNavLink>
            </CNavItem>
          )}
        </CHeaderNav>
      </CContainer>
    </CHeader>
  )
}
