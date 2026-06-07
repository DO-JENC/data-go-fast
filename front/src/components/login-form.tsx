import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { useAuth } from "@/hooks/use-auth"
import { Eye, EyeOff, Lock, Mail } from "lucide-react"
import { useState } from "react"
import { useNavigate } from "react-router-dom"

export function LoginForm() {
  const { login, loading, error } = useAuth()
  const [password, setPassword] = useState("")
  const [email, setEmail] = useState("")
  const [showPassword, setShowPassword] = useState(false)
  const navigate = useNavigate()

  const handleSubmit = async (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault()

    const success = await login({ email, password })
    if (success) {
      navigate("/datasources")
    }
  }

  return (
    <Card className="mx-auto w-full max-w-md bg-white shadow-sm transition-shadow hover:shadow-md">
      <CardHeader className="space-y-4 pb-6 pt-8">
        <div className="space-y-1.5 text-center">
          <CardTitle className="text-2xl font-bold tracking-tight">
            Sign in to your account
          </CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        <form onSubmit={(e) => handleSubmit(e)} className="grid gap-6">
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor="email">Email address</FieldLabel>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="name@example.com"
                  required
                  className="pl-10 focus-visible:ring-[#8828ad]/50"
                />
              </div>
            </Field>

            <Field>
              <FieldLabel htmlFor="password">Password</FieldLabel>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  type={showPassword ? "text" : "password"}
                  required
                  className="px-10 focus-visible:ring-[#8828ad]/50"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors hover:text-foreground"
                  tabIndex={-1}
                >
                  {showPassword ? (
                    <EyeOff className="size-4" />
                  ) : (
                    <Eye className="size-4" />
                  )}
                </button>
              </div>
            </Field>

            <Button
              type="submit"
              disabled={loading}
              className="w-full bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
              size="lg"
            >
              {loading ? "Signing in..." : "Sign in"}
            </Button>
          </FieldGroup>
        </form>
      </CardContent>
      <CardFooter className="flex flex-col border-t-0 bg-muted/30">
        <div className="text-center text-sm text-muted-foreground">
          Don't have an account?{" "}
          <a
            href="/login"
            className="font-semibold !text-orange-500 transition-colors hover:!text-orange-600 hover:underline underline-offset-4"
          >
            Create one
          </a>
        </div>
      </CardFooter>
    </Card>
  )
}
