import { ProxySettings } from '../../../../types'

/**
 * Validates if a proxy configuration has a valid URL
 */
export function isProxyValid(values: any): boolean {
  if (!values || typeof values !== 'object') {
    return false
  }

  if (
    !values.url ||
    typeof values.url !== 'string' ||
    values.url.trim() === ''
  ) {
    return false
  }

  try {
    new URL(values.url)
    return true
  } catch {
    return false
  }
}

/**
 * Creates a configuration hash for comparison (excludes 'enabled' field)
 */
export function createProxyConfigHash(values: ProxySettings): string {
  return JSON.stringify({
    url: values.url,
    username: values.username,
    password: values.password,
    no_proxy: values.no_proxy,
    ignore_ssl_certificates: values.ignore_ssl_certificates,
  })
}

/**
 * Determines if a proxy can be enabled based on testing status
 */
export function canEnableProxy(
  values: ProxySettings,
  isProxyTested: boolean,
  lastTestedConfig: string | null,
  allowEmptyUrl: boolean = false,
): boolean {
  // If no URL is provided, can enable/disable based on allowEmptyUrl flag
  if (!values.url || values.url.trim() === '') {
    return allowEmptyUrl
  }

  const currentConfig = createProxyConfigHash(values)
  return isProxyTested && currentConfig === lastTestedConfig
}

/**
 * Checks if proxy configuration has changed since last test
 */
export function hasProxyConfigChanged(
  values: ProxySettings,
  lastTestedConfig: string | null,
): boolean {
  if (!lastTestedConfig) return true

  const currentConfig = createProxyConfigHash(values)
  return currentConfig !== lastTestedConfig
}

/**
 * Form validation dependencies for proxy settings
 */
export const PROXY_FORM_DEPENDENCIES = [
  'enabled',
  'url',
  'username',
  'password',
  'no_proxy',
  'ignore_ssl_certificates',
  'proxy_ssl',
  'proxy_host_ssl',
  'peer_ssl',
  'host_ssl',
]

/**
 * Creates shouldUpdate function for Form.Item
 */
export function createProxyShouldUpdate() {
  return (prevValues: any, currentValues: any) =>
    prevValues.enabled !== currentValues.enabled ||
    prevValues.url !== currentValues.url ||
    prevValues.username !== currentValues.username ||
    prevValues.password !== currentValues.password ||
    prevValues.no_proxy !== currentValues.no_proxy ||
    prevValues.ignore_ssl_certificates !== currentValues.ignore_ssl_certificates
}

/**
 * Creates a minimal shouldUpdate function for URL/enabled changes only
 */
export function createProxyMinimalShouldUpdate() {
  return (prevValues: any, currentValues: any) =>
    prevValues.enabled !== currentValues.enabled ||
    prevValues.url !== currentValues.url
}
