import { useEffect, useState } from 'react'
import { ConfigProvider, theme } from 'antd'
import { useSettingsStore, getResolvedTheme } from '../store/settings'

interface ThemeProviderProps {
  children: React.ReactNode
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const { theme: selectedTheme, componentSize } = useSettingsStore()
  const [isDarkMode, setIsDarkMode] = useState(false)

  useEffect(() => {
    const updateTheme = () => {
      const resolvedTheme = getResolvedTheme(selectedTheme)
      setIsDarkMode(resolvedTheme === 'dark')
    }

    updateTheme()

    // Listen for system theme changes if auto mode is selected
    if (selectedTheme === 'auto') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = () => updateTheme()
      
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

  return (
    <ConfigProvider
      componentSize={componentSize}
      theme={{
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
        token: {
          colorPrimary: '#1890ff',
          borderRadius: 8,
          fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
        },
        components: {
          Layout: {
            bodyBg: isDarkMode ? '#141414' : '#ffffff',
            siderBg: isDarkMode ? '#1f1f1f' : '#ffffff',
            headerBg: isDarkMode ? '#1f1f1f' : '#ffffff',
          },
          Menu: {
            itemBg: 'transparent',
            itemSelectedBg: isDarkMode ? 'rgba(255, 255, 255, 0.1)' : 'rgba(24, 144, 255, 0.1)',
            itemHoverBg: isDarkMode ? 'rgba(255, 255, 255, 0.05)' : 'rgba(24, 144, 255, 0.05)',
          },
          Card: {
            colorBgContainer: isDarkMode ? '#1f1f1f' : '#ffffff',
          },
          Input: {
            colorBgContainer: isDarkMode ? '#262626' : '#ffffff',
          },
          Button: {
            colorPrimary: '#1890ff',
            colorPrimaryHover: '#40a9ff',
            colorPrimaryActive: '#096dd9',
          },
        },
      }}
    >
      {children}
    </ConfigProvider>
  )
}