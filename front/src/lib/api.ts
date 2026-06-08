/**
 * Centralized API client utility.
 * Automatically handles JWT token injection and provides a cleaner interface for requests.
 */

interface RequestOptions extends RequestInit {
  params?: Record<string, string>
}

async function request<T>(
  endpoint: string,
  options: RequestOptions = {},
): Promise<T> {
  const token = localStorage.getItem("access_token")

  const headers: HeadersInit = {
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...options.headers,
  }

  const config: RequestInit = {
    ...options,
    headers,
  }

  // Handle query parameters if provided
  let url = endpoint
  if (options.params) {
    const searchParams = new URLSearchParams(options.params)
    url += `?${searchParams.toString()}`
  }

  // Ensure url starts with /api if it doesn't already and isn't a full URL
  const fullUrl =
    url.startsWith("http") || url.startsWith("/api")
      ? url
      : `/api${url.startsWith("/") ? "" : "/"}${url}`

  const response = await fetch(fullUrl, config)

  if (response.status === 401) {
    // Global logout on unauthorized
    localStorage.removeItem("access_token")
    window.location.href = "/login"
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
}
