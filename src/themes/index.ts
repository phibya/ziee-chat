export { lightTheme } from './light'
export { darkTheme } from './dark'
export type { ThemeType } from './light'

import { lightTheme } from './light'
import { darkTheme } from './dark'

export const themes = {
  light: lightTheme,
  dark: darkTheme,
} as const

export type ThemeName = keyof typeof themes
