import { ApiEndpointUrl, ParameterByUrl, ResponseByUrl } from '../types'

import { invoke } from '@tauri-apps/api/core'

// Import getAuthToken function (avoiding circular import)
export const getAuthToken = () => {
  // eslint-disable-next-line no-undef
  const authData = localStorage.getItem('auth-storage')
  if (authData) {
    const parsed = JSON.parse(authData)
    return parsed.state?.token || null
  }
  return null
}

//@ts-ignore
export const isDesktopApp = !!window.__TAURI_INTERNALS__

export const getBaseUrl = (function () {
  let baseUrl: Promise<string>
  //@ts-ignore
  return async function () {
    if (baseUrl) {
      return baseUrl // Return existing promise if already created
    }

    baseUrl = new Promise<string>(resolve => {
      if (isDesktopApp) {
        invoke('get_http_port')
          .then(port => {
            resolve(`http://localhost:${port as number}`)
          })
          .catch(console.error)
      } else {
        // For web, we can use the current origin
        //@ts-ignore
        resolve(window.location.origin)
      }
    })
    return baseUrl
  }
})()

// SSE streaming callback types
export interface SSECallbacks {
  onChunk?: (data: any) => void
  onComplete?: (data: any) => void
  onError?: (error: string) => void
}

// Type-safe callAsync function that maps URL to exact parameter and response types
export const callAsync = async <U extends ApiEndpointUrl>(
  endpointUrl: U,
  params: ParameterByUrl<U>,
  sseCallbacks?: SSECallbacks,
): Promise<ResponseByUrl<U>> => {
  let bUrl = await getBaseUrl()

  try {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    }

    // Add auth token if available
    const token = getAuthToken()
    if (token) {
      headers['Authorization'] = `Bearer ${token}`
    }

    const method = endpointUrl.split(' ')[0] as
      | 'POST'
      | 'GET'
      | 'PUT'
      | 'DELETE'
      | 'PATCH'
    let endpointPath = endpointUrl.replace(/^[A-Z]+\s+/, '').trim()
    //get {capture} from endpointPath
    const captureMatches = (endpointPath.match(/{([^}]+)}/g) || []).map(match =>
      match.slice(1, -1),
    )
    // Replace {capture} with actual values from params
    captureMatches.forEach(capture => {
      let c = capture.trim() as keyof typeof params
      if (params[c] !== undefined) {
        //@ts-ignore
        endpointPath = endpointPath.replace(`{${capture}}`, params[c])
        // delete params[c] // Remove from params to avoid sending it in body
      } else {
        throw new Error(`Missing required parameter: ${capture}`)
      }
    })

    const response = await fetch(`${bUrl}${endpointPath}`, {
      method,
      headers,
      body:
        ['POST', 'PUT', 'PATCH'].includes(method) && params !== undefined
          ? JSON.stringify(params)
          : undefined,
    })

    // Handle SSE streaming if callbacks are provided and response is text/event-stream
    if (
      sseCallbacks &&
      response.headers.get('Content-Type')?.includes('text/event-stream')
    ) {
      if (!response.ok) {
        const errorMessage = `HTTP error! status: ${response.status}`
        sseCallbacks.onError?.(errorMessage)
        throw new Error(errorMessage)
      }

      const reader = response.body?.getReader()
      if (!reader) {
        const error = 'No response body reader available'
        sseCallbacks.onError?.(error)
        throw new Error(error)
      }

      const decoder = new globalThis.TextDecoder()
      let buffer = ''

      try {
        let currentEvent = ''

        while (true) {
          const { done, value } = await reader.read()
          if (done) break

          buffer += decoder.decode(value, { stream: true })
          const lines = buffer.split('\n')
          buffer = lines.pop() || '' // Keep incomplete line in buffer

          for (const line of lines) {
            if (line.trim() === '') {
              // Empty line indicates end of event, reset current event
              currentEvent = ''
              continue
            }

            if (line.startsWith('event: ')) {
              currentEvent = line.slice(7).trim()
            } else if (line.startsWith('data: ')) {
              const data = line.slice(6)

              // Handle special cases
              if (data === 'start') {
                continue // Skip start signal
              }
              if (data === '[DONE]') {
                break
              }

              try {
                const parsed = JSON.parse(data)

                // Handle based on current event type
                if (currentEvent === 'chunk') {
                  sseCallbacks.onChunk?.(parsed)
                } else if (currentEvent === 'complete') {
                  sseCallbacks.onComplete?.(parsed)
                } else if (currentEvent === 'error') {
                  sseCallbacks.onError?.(parsed?.error || 'Stream error')
                }
              } catch {
                console.warn('Failed to parse SSE data:', data)
              }
            }
          }
        }
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : 'Stream reading error'
        sseCallbacks.onError?.(errorMessage)
        throw error
      } finally {
        reader.releaseLock()
      }

      // For SSE streaming, return empty response since data is handled via callbacks
      return {} as ResponseByUrl<U>
    }

    // Parse the response as JSON
    if (!response.ok) {
      let errorMessage = `HTTP error! status: ${response.status}`

      // Handle 403 Forbidden specifically
      if (response.status === 403) {
        try {
          // Try to extract specific error message from response body
          const errorResponse = await response.json()
          if (errorResponse.error) {
            errorMessage = errorResponse.error
          } else {
            errorMessage = 'Permission denied'
          }
        } catch {
          // If we can't parse the error response, use default permission denied message
          errorMessage = 'Permission denied'
        }
      } else {
        try {
          // Try to extract error message from response body for other errors
          const errorResponse = await response.json()
          if (errorResponse.error) {
            errorMessage = errorResponse.error
          }
        } catch {
          // If we can't parse the error response, use the default message
        }
      }

      throw new Error(errorMessage)
    }

    //try to parse the response as JSON else return as text
    if (response.headers.get('Content-Type')?.includes('application/json')) {
      return (await response.json()) as ResponseByUrl<U>
    } else {
      const textResponse = await response.text()
      return textResponse as unknown as ResponseByUrl<U> // Fallback to text if not JSON
    }
  } catch (error) {
    console.error(`Error calling endpoint ${endpointUrl}:`, error)
    throw error // Re-throw to allow caller to handle it
  }
}
