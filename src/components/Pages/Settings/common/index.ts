// Re-export unified proxy form
export {
  ProxySettingsForm,
  type ProxySettingsFormProps,
} from './ProxySettingsForm'

// Re-export proxy utilities
export {
  canEnableProxy,
  createProxyConfigHash,
  createProxyMinimalShouldUpdate,
  createProxyShouldUpdate,
  hasProxyConfigChanged,
  isProxyValid,
  PROXY_FORM_DEPENDENCIES,
} from './proxyUtils'
