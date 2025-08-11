import { ConfigProvider } from 'antd'
import { useEffect } from 'react'
import { useUpdate } from 'react-use'
import { ThemeContext } from '../../hooks/useTheme.ts'
import { useUserAppearanceTheme } from '../../store'
import { themes } from '../themes'
import { AppThemeConfig } from '../themes/light.ts'

interface ThemeProviderProps {
  children: React.ReactNode
}

const resolveSystemTheme = (): 'light' | 'dark' => {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  return mediaQuery.matches ? 'dark' : 'light'
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const selectedTheme = useUserAppearanceTheme()
  const resolvedTheme =
    selectedTheme === 'system' ? resolveSystemTheme() : selectedTheme
  const isDarkMode = resolvedTheme === 'dark'
  const currentTheme: AppThemeConfig = themes[resolvedTheme] || themes.light

  const update = useUpdate()

  useEffect(() => {
    // Listen for system theme changes if system mode is selected
    if (selectedTheme === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = () => update()

      mediaQuery.addEventListener('change', handleChange)
      return () => mediaQuery.removeEventListener('change', handleChange)
    }
  }, [selectedTheme])

  // Update document class for global theme styling
  useEffect(() => {
    const root = document.documentElement
    if (isDarkMode) {
      root.classList.add('dark')
      root.classList.remove('light')
    } else {
      root.classList.add('light')
      root.classList.remove('dark')
    }
  }, [isDarkMode])

  console.log(currentTheme)

  return (
    <ThemeContext.Provider value={currentTheme}>
      <ConfigProvider theme={currentTheme}>{children}</ConfigProvider>
    </ThemeContext.Provider>
  )
}
