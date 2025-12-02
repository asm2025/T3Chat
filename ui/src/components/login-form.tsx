import { useState, useEffect } from "react"
import { useSearchParams, useNavigate } from "react-router-dom"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/card"
import { Button } from "./ui/button"
import { useAuth } from "@/lib/auth-context"
import { handleCallback } from "@/lib/auth"
import { toast } from "@/lib/toast"
import { Loader2 } from "lucide-react"

export function LoginForm() {
  const [isLoading, setIsLoading] = useState(false)
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const { login } = useAuth()

  // Handle OIDC callback
  useEffect(() => {
    const token = searchParams.get('token')
    if (token) {
      setIsLoading(true)
      handleCallback(token)
        .then(() => {
          toast.success("Successfully logged in")
          navigate('/', { replace: true })
          // Reload page to refresh auth state
          window.location.reload()
        })
        .catch((error) => {
          toast.error("Failed to complete login", {
            description: error.message,
          })
        })
        .finally(() => {
          setIsLoading(false)
        })
    }
  }, [searchParams, navigate])

  const handleLogin = () => {
    setIsLoading(true)
    login()
  }

  return (
    <Card className="w-full max-w-md rounded-2xl border border-border bg-card shadow-lg">
      <CardHeader>
        <CardTitle>Authentication</CardTitle>
        <CardDescription>Sign in with your OIDC provider to continue.</CardDescription>
      </CardHeader>
      
      <CardContent className="space-y-6">
        <div className="text-center space-y-2">
          <p className="text-sm text-muted-foreground">
            Click the button below to sign in with your organization's identity provider.
          </p>
        </div>
        
        <Button
          type="button"
          className="w-full"
          onClick={handleLogin}
          disabled={isLoading}
        >
          {isLoading ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Redirecting...
            </>
          ) : (
            "Login with OIDC"
          )}
        </Button>
      </CardContent>
    </Card>
  )
}
