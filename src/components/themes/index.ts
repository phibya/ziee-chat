export { lightTheme } from './light'
export { darkTheme } from './dark'

import { lightTheme } from './light'
import { darkTheme } from './dark'

export const themes = {
  light: lightTheme,
  dark: darkTheme,
} as const

export type ThemeName = keyof typeof themes
