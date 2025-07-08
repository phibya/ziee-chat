import { ApiEndpointUrl, ParameterByUrl, ResponseByUrl } from './enpoints.ts'

import { invoke } from '@tauri-apps/api/core'

// Import getAuthToken function (avoiding circular import)
const getAuthToken = () => {
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

const getBaseUrl = (function () {
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

// Type-safe callAsync function that maps URL to exact parameter and response types
export const callAsync = async <U extends ApiEndpointUrl>(
  endpointUrl: U,
  params: ParameterByUrl<U>,
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
        delete params[c] // Remove from params to avoid sending it in body
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

    // Parse the response as JSON
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
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
