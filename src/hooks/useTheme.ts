import { createContext, useContext } from 'react'
import { ThemeConfig } from 'antd'

export const ThemeContext = createContext<ThemeConfig | undefined>(undefined)

export function useTheme(): ThemeConfig {
  const theme = useContext(ThemeContext)
  if (!theme) {
    throw new Error('useTheme must be used within a ThemeProvider')
  }
  return theme
}
