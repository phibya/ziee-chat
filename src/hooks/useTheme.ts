import { createContext, useContext } from 'react'
import { ThemeType } from '../themes'

export const ThemeContext = createContext<ThemeType | undefined>(undefined)

export function useTheme(): ThemeType {
  const theme = useContext(ThemeContext)
  if (!theme) {
    throw new Error('useTheme must be used within a ThemeProvider')
  }
  return theme
}
