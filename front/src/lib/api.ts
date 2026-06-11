/**
 * Centralized API client utility.
 * Automatically handles JWT token injection and provides a cleaner interface for requests.
 */

const TOKEN_KEY = "access_token"

interface RequestOptions extends RequestInit {
  params?: Record<string, string>
}

interface RefreshTokenResponse {
  access_token?: string
}

async function request<T>(
  endpoint: string,
  options: RequestOptions = {},
): Promise<T> {
  const token = localStorage.getItem(TOKEN_KEY)

  const headers: HeadersInit = {
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...options.headers,
  }

  // Only set Content-Type to application/json if it's not FormData and not already set
  if (!(options.body instanceof FormData) && !headers.hasOwnProperty("Content-Type")) {
    ;(headers as Record<string, string>)["Content-Type"] = "application/json"
  }

  const config: RequestInit = {
    ...options,
    headers,
    credentials: "include", // Required for cookies (refresh token)
  }

  // Handle query parameters if provided
  let url = endpoint
  if (options.params) {
    const searchParams = new URLSearchParams(options.params)
    url += `?${searchParams.toString()}`
  }

  const fullUrl = url.startsWith("/api")
    ? url
    : `/api/${url.replace(/^\//, "")}`

  const response = await fetch(fullUrl, config)

  if (response.status === 401) {
    // If we're not on the login page and it's not a refresh request, try to refresh
    if (
      !window.location.pathname.includes("/login") &&
      !url.includes("/auth/refresh")
    ) {
      try {
        const refreshResponse =
          await api.post<RefreshTokenResponse>("/auth/refresh")
        if (refreshResponse.access_token) {
          api.setToken(refreshResponse.access_token)
          // Retry original request
          return request<T>(endpoint, options)
        }
      } catch {
        // Refresh failed silently, proceeding to global logout
      }
    }

    // Global logout on unauthorized
    localStorage.removeItem(TOKEN_KEY)
    if (!window.location.pathname.includes("/login")) {
      window.location.href = "/login"
    }
    throw new Error("Unauthorized")
  }

  if (!response.ok) {
    const errorBody = await response.text().catch(() => "Unknown error")
    throw new Error(
      errorBody || `Error ${response.status}: ${response.statusText}`,
    )
  }

  // Some endpoints might return empty body (like 204 No Content)
  if (response.status === 204) {
    return {} as T
  }

  return response.json()
}

export const api = {
  get: <T>(url: string, options?: RequestOptions) =>
    request<T>(url, { ...options, method: "GET" }),

  post: <T>(url: string, body?: unknown, options?: RequestOptions) =>
    request<T>(url, {
      ...options,
      method: "POST",
      body:
        body instanceof FormData ? body : body ? JSON.stringify(body) : undefined,
    }),

  put: <T>(url: string, body?: unknown, options?: RequestOptions) =>
    request<T>(url, {
      ...options,
      method: "PUT",
      body:
        body instanceof FormData ? body : body ? JSON.stringify(body) : undefined,
    }),

  delete: <T>(url: string, options?: RequestOptions) =>
    request<T>(url, { ...options, method: "DELETE" }),

  // Token management
  setToken: (token: string) => localStorage.setItem(TOKEN_KEY, token),
  clearToken: () => localStorage.removeItem(TOKEN_KEY),
  getToken: () => localStorage.getItem(TOKEN_KEY),
}
