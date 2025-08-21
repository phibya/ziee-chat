import { App, ConfigProvider } from 'antd'
import { useEffect } from 'react'
import { useUpdate } from 'react-use'
import { ThemeContext } from '../../hooks/useTheme.ts'
import { useUserAppearanceTheme } from '../../store'
import { themes } from '../themes'
import { AppThemeConfig } from '../themes/light.ts'
import { resolveSystemTheme } from './resolveTheme.ts'

interface ThemeProviderProps {
  children: React.ReactNode
}
export function ThemeProvider({ children }: ThemeProviderProps) {
  const selectedTheme = useUserAppearanceTheme()
  const resolvedTheme =
    selectedTheme === 'system' ? resolveSystemTheme() : selectedTheme
  const isDarkMode = resolvedTheme === 'dark'
  const currentTheme: AppThemeConfig = themes[resolvedTheme] || themes.light

  const update = useUpdate()

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => update()

    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [selectedTheme])

  // Update document class for global theme styling
  useEffect(() => {
    const root = document.documentElement
    //find meta tag with name="theme-color" and set its content to the theme color
    let metaThemeColor = document.querySelector('meta[name="theme-color"]')
    if (!metaThemeColor) {
      metaThemeColor = document.createElement('meta')
      metaThemeColor.setAttribute('name', 'theme-color')
      document.head.appendChild(metaThemeColor)
    }

    metaThemeColor.setAttribute(
      'content',
      currentTheme.token?.colorBgContainer!,
    )

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
      <ConfigProvider theme={currentTheme}>
        <App
          message={{
            top: 50,
          }}
        >
          {children}
        </App>
      </ConfigProvider>
    </ThemeContext.Provider>
  )
}
