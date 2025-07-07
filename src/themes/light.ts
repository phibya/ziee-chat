export const lightTheme = {
  // Primary colors
  primary: '#1890ff',
  primaryHover: '#40a9ff',
  primaryActive: '#096dd9',

  // Background colors
  background: '#ffffff',
  backgroundSecondary: '#f5f5f5',
  backgroundElevated: '#ffffff',

  // Surface colors
  surface: '#ffffff',
  surfaceSecondary: '#fafafa',
  surfaceElevated: '#ffffff',

  // Text colors
  textPrimary: '#262626',
  textSecondary: '#595959',
  textTertiary: '#8c8c8c',
  textDisabled: '#bfbfbf',

  // Border colors
  border: '#d9d9d9',
  borderSecondary: '#f0f0f0',
  borderLight: '#f5f5f5',

  // Status colors
  success: '#52c41a',
  warning: '#faad14',
  error: '#ff4d4f',
  info: '#1890ff',

  // Sidebar specific
  sidebarBackground: '#ffffff',
  sidebarBorder: '#f0f0f0',
  sidebarItemHover: 'rgba(24, 144, 255, 0.05)',
  sidebarItemActive: 'rgba(24, 144, 255, 0.1)',
  sidebarText: '#262626',
  sidebarTextSecondary: '#595959',

  // Chat specific
  chatBackground: '#fafafa',
  chatMessageUser: '#1890ff',
  chatMessageAssistant: '#f5f5f5',
  chatMessageUserText: '#ffffff',
  chatMessageAssistantText: '#262626',

  // Input specific
  inputBackground: '#ffffff',
  inputBorder: '#d9d9d9',
  inputPlaceholder: '#bfbfbf',

  // Shadow
  shadow: 'rgba(0, 0, 0, 0.1)',
  shadowElevated: 'rgba(0, 0, 0, 0.15)',
}

export type ThemeType = typeof lightTheme
