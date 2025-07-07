import { ApiEndpointUrl, ParameterByUrl, ResponseByUrl } from './enpoints.ts'

import { invoke } from '@tauri-apps/api/core'

const getBaseUrl = (function () {
  let baseUrl: Promise<string>
  //@ts-ignore
  return async function () {
    if (baseUrl) {
      return baseUrl // Return existing promise if already created
    }

    //@ts-ignore
    let isDesktop = !!window.__TAURI_INTERNALS__
    baseUrl = new Promise<string>(resolve => {
      if (isDesktop) {
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
    const response = await fetch(`${bUrl}${endpointUrl}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: params === undefined ? undefined : JSON.stringify(params),
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
