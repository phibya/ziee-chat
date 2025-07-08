import { useEffect, useState } from 'react'
import { ConfigProvider, theme } from 'antd'
import { useUserSettingsStore } from '../store/settings'
import { themes, ThemeType } from '../themes'
import { ThemeContext } from '../hooks/useTheme'

interface ThemeProviderProps {
  children: React.ReactNode
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const { getAppearanceTheme, getAppearanceComponentSize, getResolvedTheme } =
    useUserSettingsStore()
  const [isDarkMode, setIsDarkMode] = useState(false)
  const [currentTheme, setCurrentTheme] = useState<ThemeType>(themes.light)

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
      <ConfigProvider
        componentSize={componentSize}
        theme={{
          algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
          token: {
            colorPrimary: currentTheme.primary,
            borderRadius: 8,
            fontFamily:
              '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
            colorText: currentTheme.textPrimary,
            colorTextSecondary: currentTheme.textSecondary,
            colorTextTertiary: currentTheme.textTertiary,
            colorTextDisabled: currentTheme.textDisabled,
            colorBgContainer: currentTheme.surface,
            colorBgElevated: currentTheme.surfaceElevated,
            colorBorder: currentTheme.border,
            colorBorderSecondary: currentTheme.borderSecondary,
          },
          components: {
            Layout: {
              bodyBg: currentTheme.background,
              siderBg: currentTheme.sidebarBackground,
              headerBg: currentTheme.surface,
            },
            Menu: {
              itemBg: 'transparent',
              itemSelectedBg: currentTheme.sidebarItemActive,
              itemHoverBg: currentTheme.sidebarItemHover,
            },
            Card: {
              colorBgContainer: currentTheme.surface,
            },
            Input: {
              colorBgContainer: currentTheme.inputBackground,
              colorBorder: currentTheme.inputBorder,
              colorTextPlaceholder: currentTheme.inputPlaceholder,
            },
            Button: {
              colorPrimary: currentTheme.primary,
              colorPrimaryHover: currentTheme.primaryHover,
              colorPrimaryActive: currentTheme.primaryActive,
            },
          },
        }}
      >
        {children}
      </ConfigProvider>
    </ThemeContext.Provider>
  )
}
