import { useEffect, useState } from 'react'
import { useWindowSize } from 'react-use'
import { useLayoutUIStore } from '../../store'

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

export type MinSize = {
  xs: boolean
  sm: boolean
  md: boolean
  lg: boolean
  xl: boolean
  '2xl': boolean
  '3xl': boolean
}

export const useWindowMinSize = (): MinSize => {
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

const calculateMinSize = (width: number): MinSize => ({
  xs: width <= breakpointValues.sm,
  sm: width <= breakpointValues.md,
  md: width <= breakpointValues.lg,
  lg: width <= breakpointValues.xl,
  xl: width <= breakpointValues['2xl'],
  '2xl': width <= breakpointValues['3xl'],
  '3xl': width > breakpointValues['3xl'],
})

export const useMainContentMinSize = (): MinSize => {
  const [minSize, setMinSize] = useState<MinSize>(() => {
    const currentWidth = useLayoutUIStore.getState().mainContentWidth
    return calculateMinSize(currentWidth)
  })

  useEffect(() => {
    const updateMinSize = (state: any) => {
      const newMinSize = calculateMinSize(state.mainContentWidth)

      // Only update if the new minSize is different from the current one
      setMinSize(prevMinSize => {
        const isEqual = Object.keys(newMinSize).every(
          key =>
            prevMinSize[key as keyof MinSize] ===
            newMinSize[key as keyof MinSize],
        )
        return isEqual ? prevMinSize : newMinSize
      })
    }

    // Subscribe to future changes
    let unsub = useLayoutUIStore.subscribe(updateMinSize)

    return () => {
      unsub()
    }
  }, [])

  return minSize
}
