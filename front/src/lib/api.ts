/**
 * Centralized API client utility.
 * Automatically handles JWT token injection and provides a cleaner interface for requests.
 */

const TOKEN_KEY = "access_token"
const API_BASE_URL = import.meta.env.VITE_API_URL || ""

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
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...options.headers,
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

  // Determine the full URL
  // 1. If it's already a full URL, use it
  // 2. If it starts with /api, use it as is (relative to the base)
  // 3. Otherwise, prepend API_BASE_URL and /api
  let fullUrl = url
  if (!url.startsWith("http")) {
    const prefix = url.startsWith("/api") ? "" : "/api"
    const separator = url.startsWith("/") ? "" : "/"
    fullUrl = `${API_BASE_URL}${prefix}${separator}${url}`
  }

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
      body: body ? JSON.stringify(body) : undefined,
    }),

  put: <T>(url: string, body?: unknown, options?: RequestOptions) =>
    request<T>(url, {
      ...options,
      method: "PUT",
      body: body ? JSON.stringify(body) : undefined,
    }),

  delete: <T>(url: string, options?: RequestOptions) =>
    request<T>(url, { ...options, method: "DELETE" }),

  // Token management
  setToken: (token: string) => localStorage.setItem(TOKEN_KEY, token),
  clearToken: () => localStorage.removeItem(TOKEN_KEY),
  getToken: () => localStorage.getItem(TOKEN_KEY),
}
