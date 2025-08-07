import { useWindowSize } from 'react-use'

export type Breakpoint = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl'

const breakpointValues: Record<Breakpoint, number> = {
  xs: 0,
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536,
  '3xl': 1920,
}

export const useWindowMinSize = (): {
  xs: boolean
  sm: boolean
  md: boolean
  lg: boolean
  xl: boolean
  '2xl': boolean
  '3xl': boolean
} => {
  const { width } = useWindowSize()

  return {
    xs: width <= breakpointValues.sm,
    sm: width <= breakpointValues.md,
    md: width <= breakpointValues.lg,
    lg: width <= breakpointValues.xl,
    xl: width <= breakpointValues.xl,
    '2xl': width <= breakpointValues['xl'],
    '3xl': width <= breakpointValues['2xl'],
  }
}
