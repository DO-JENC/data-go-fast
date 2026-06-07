import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { useAuth } from "@/hooks/use-auth"
import { Eye, EyeOff, Lock, Mail } from "lucide-react"
import { useState } from "react"
import { useNavigate } from "react-router-dom"
import { toast } from "sonner"

export function SignupForm() {
  const { signup, loading, error } = useAuth()
  const [password, setPassword] = useState("")
  const [email, setEmail] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const navigate = useNavigate()

  const handleSubmit = async (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault()

    if (password != confirmPassword) {
      toast.error("Passwords do not match")
      return
    }

    const success = await signup({ email, password })
    if (success) {
      navigate("/datasources")
    }
  }

  return (
    <Card className="mx-auto w-full max-w-md bg-white shadow-sm transition-shadow hover:shadow-md">
      <CardHeader className="space-y-3 pb-4 pt-6">
        <div className="space-y-1 text-center">
          <CardTitle className="text-xl font-bold tracking-tight">
            Create an account
          </CardTitle>
          <CardDescription className="text-balance text-sm">
            Enter your details below to create your account
          </CardDescription>
        </div>
      </CardHeader>
      <CardContent className="pb-4">
        <form onSubmit={(e) => handleSubmit(e)} className="grid gap-4">
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
                  className="h-9 pl-10 focus-visible:ring-[#8828ad]/50"
                />
              </div>
            </Field>

            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
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
                    className="h-9 px-10 focus-visible:ring-[#8828ad]/50"
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
              <Field>
                <FieldLabel htmlFor="confirm-password">Confirm</FieldLabel>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="confirm-password"
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    type={showConfirmPassword ? "text" : "password"}
                    required
                    className="h-9 px-10 focus-visible:ring-[#8828ad]/50"
                  />
                  <button
                    type="button"
                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors hover:text-foreground"
                    tabIndex={-1}
                  >
                    {showConfirmPassword ? (
                      <EyeOff className="size-4" />
                    ) : (
                      <Eye className="size-4" />
                    )}
                  </button>
                </div>
              </Field>
            </div>

            <FieldDescription className="text-xs">
              Password must be at least 8 characters long and include special
              characters.
            </FieldDescription>

            <Button
              type="submit"
              disabled={loading}
              className="mt-1 w-full bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
              size="default"
            >
              {loading ? "Creating account..." : "Create account"}
            </Button>
          </FieldGroup>
        </form>
      </CardContent>
      <CardFooter className="flex flex-col border-t-0 bg-muted/30 py-3">
        <div className="text-center text-sm text-muted-foreground">
          Already have an account?{" "}
          <a
            href="/login"
            className="font-semibold !text-orange-500 transition-colors hover:!text-orange-600 hover:underline underline-offset-4"
          >
            Sign in
          </a>
        </div>
      </CardFooter>
    </Card>
  )
}
