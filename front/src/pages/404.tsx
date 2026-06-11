import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { Home } from "lucide-react"
import { Link } from "react-router-dom"

export default function PageNotFound() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center p-4">
      <Card className="mx-auto w-full max-w-md bg-white shadow-sm transition-shadow hover:shadow-md">
        <CardHeader className="space-y-4 pb-6 pt-8">
          <div className="space-y-1.5 text-center">
            <p className="text-sm font-bold uppercase tracking-widest text-[#f65d19] animate-bounce">
              404
            </p>
            <h1 className="text-2xl font-bold tracking-tight">
              Oops! Lost in Cyberspace...
            </h1>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center gap-4 text-center">
            <p className="text-sm text-muted-foreground">
              Seems that the page you're looking for doesn't exist or has been
              moved
            </p>
            <Link to="/">
              <Button className="bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]">
                <Home className="size-4" />
                Go home
              </Button>
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
