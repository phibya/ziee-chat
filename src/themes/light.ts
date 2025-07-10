import { theme, ThemeConfig } from 'antd'
import { ComponentTokens, GlobalTokens } from './tokens.ts'
import { ComponentOverrides, TokenOverrides } from './override.ts'

const baseTheme = {
  algorithm: [theme.compactAlgorithm, theme.defaultAlgorithm],
  token: {
    ...GlobalTokens,
    ...TokenOverrides,
  },
  components: {
    ...ComponentTokens,
    ...ComponentOverrides,
    Button: {
      ...ComponentTokens.Button,
      ...ComponentOverrides.Button,
      // Override button tokens for light theme
    },
    // Other component overrides can go here
  },
  app: {
    chatBackground: '#f0f2f5',
  },
} as const

type AppTokenKeys = keyof typeof baseTheme.app
type AppToken = {
  [K in AppTokenKeys]: (typeof baseTheme.app)[K] extends string
    ? string
    : (typeof baseTheme.app)[K] extends number
      ? number
      : (typeof baseTheme.app)[K] extends boolean
        ? boolean
        : (typeof baseTheme.app)[K]
}

export type AppThemeConfig = {
  algorithm: ThemeConfig['algorithm']
  token: ThemeConfig['token']
  components: ThemeConfig['components']
  app: AppToken
}

const lightTheme = baseTheme as unknown as AppThemeConfig

export { lightTheme }
