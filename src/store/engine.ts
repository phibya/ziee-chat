import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../api/client'
import type { EngineInfo } from '../types'

interface EngineState {
  engines: EngineInfo[]
  loading: boolean
  error: string | null
  initialized: boolean
}

export const useEngineStore = create<EngineState>()(
  subscribeWithSelector((_set, _get) => ({
    engines: [],
    loading: false,
    error: null,
    initialized: false,
  })),
)

// Store methods - defined OUTSIDE the store definition
export const initializeEngines = async () => {
  const state = useEngineStore.getState()
  if (state.initialized || state.loading) {
    return
  }

  useEngineStore.setState({ loading: true, error: null })

  try {
    const response = await ApiClient.Admin.listEngines()
    useEngineStore.setState({
      engines: response,
      initialized: true,
      loading: false,
      error: null,
    })
  } catch (error) {
    console.error('Engine initialization failed:', error)
    useEngineStore.setState({
      loading: false,
      error: error instanceof Error ? error.message : 'Unknown error',
      initialized: false,
    })
    throw error
  }
}

// Helper functions
export const getEngineByType = (
  engines: EngineInfo[],
  engineType: string,
): EngineInfo | undefined => {
  return engines.find(engine => engine.engine_type === engineType)
}

export const getAvailableEngines = (engines: EngineInfo[]): EngineInfo[] => {
  return engines.filter(engine => engine.status === 'available')
}

export const searchEngines = (
  engines: EngineInfo[],
  query: string,
): EngineInfo[] => {
  if (!query.trim()) return engines

  const searchTerm = query.toLowerCase()
  return engines.filter(
    engine =>
      engine.name.toLowerCase().includes(searchTerm) ||
      engine.engine_type.toLowerCase().includes(searchTerm),
  )
}
