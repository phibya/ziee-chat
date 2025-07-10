import { useEffect, useState } from 'react'
import { ConfigProvider } from 'antd'
import { useUserSettingsStore } from '../store'
import { themes } from '../themes'
import { ThemeContext } from '../hooks/useTheme'
import { AppThemeConfig } from '../themes/light.ts'

interface ThemeProviderProps {
  children: React.ReactNode
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const { getAppearanceTheme, getAppearanceComponentSize, getResolvedTheme } =
    useUserSettingsStore()
  const [isDarkMode, setIsDarkMode] = useState(false)
  const [currentTheme, setCurrentTheme] = useState<AppThemeConfig>(themes.light)

  const selectedTheme = getAppearanceTheme()
  const rawComponentSize = getAppearanceComponentSize()

  // Map component size to Ant Design's expected values
  const componentSize =
    rawComponentSize === 'medium' ? 'middle' : rawComponentSize

  useEffect(() => {
    const updateTheme = () => {
      const resolvedTheme = getResolvedTheme()
      const darkMode = resolvedTheme === 'dark'
      setIsDarkMode(darkMode)
      setCurrentTheme(darkMode ? themes.dark : themes.light)
    }

    updateTheme()

    // Listen for system theme changes if system mode is selected
    if (selectedTheme === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = () => updateTheme()

      mediaQuery.addEventListener('change', handleChange)
      return () => mediaQuery.removeEventListener('change', handleChange)
    }
  }, [selectedTheme, getResolvedTheme])

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

  return (
    <ThemeContext.Provider value={currentTheme}>
      <ConfigProvider componentSize={componentSize} theme={currentTheme}>
        {children}
      </ConfigProvider>
    </ThemeContext.Provider>
  )
}
