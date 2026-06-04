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
import { Eye, EyeOff, Lock, Mail } from "lucide-react"
import { useState } from "react"

export function SignupForm() {
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)

  return (
    <Card className="mx-auto w-full max-w-md bg-white shadow-sm transition-shadow hover:shadow-md">
      <CardHeader className="space-y-4 pb-6 pt-8">
        <div className="space-y-1.5 text-center">
          <CardTitle className="text-2xl font-bold tracking-tight">
            Create an account
          </CardTitle>
          <CardDescription className="text-balance">
            Enter your details below to create your account and get started
          </CardDescription>
        </div>
      </CardHeader>
      <CardContent>
        <form onSubmit={(e) => e.preventDefault()} className="grid gap-6">
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor="email">Email address</FieldLabel>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  id="email"
                  type="email"
                  placeholder="name@example.com"
                  required
                  className="pl-10 focus-visible:ring-[#8828ad]/50"
                />
              </div>
            </Field>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <Field>
                <FieldLabel htmlFor="password">Password</FieldLabel>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="password"
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
              <Field>
                <FieldLabel htmlFor="confirm-password">Confirm</FieldLabel>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="confirm-password"
                    type={showConfirmPassword ? "text" : "password"}
                    required
                    className="px-10 focus-visible:ring-[#8828ad]/50"
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
              className="w-full bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
              size="lg"
            >
              Create Account
            </Button>
          </FieldGroup>
        </form>
      </CardContent>
      <CardFooter className="flex flex-col border-t-0 bg-muted/30">
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
