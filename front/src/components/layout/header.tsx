import "@coreui/coreui/dist/css/coreui.min.css"
import {
  CContainer,
  CHeader,
  CHeaderBrand,
  CHeaderNav,
  CNavItem,
  CNavLink,
} from "@coreui/react"

export function Header() {
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
          <CNavItem>
            {/* TODO: Logooout behaviour */}
            <CNavLink href="#">
              <img
                src="/logout-icon.png"
                alt="Déconnexion"
                className="max-h-5"
              />
            </CNavLink>
          </CNavItem>
        </CHeaderNav>
      </CContainer>
    </CHeader>
  )
}
