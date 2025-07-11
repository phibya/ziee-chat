import { theme } from 'antd'
import { AppThemeConfig } from './light.ts'
import { ComponentOverrides, TokenOverrides } from './override.ts'

export const darkTheme: AppThemeConfig = {
  algorithm: [theme.darkAlgorithm, theme.compactAlgorithm],
  token: {
    ...TokenOverrides,
  },
  components: {
    ...ComponentOverrides,
    Button: {
      ...ComponentOverrides.Button,
      // Override button tokens for dark theme
    },
    // Other component overrides can go here
  },
  app: {
    chatBackground: '#141414', // Dark background for chat
  },
} as const
