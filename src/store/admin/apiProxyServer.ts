import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { ApiClient } from '../../api/client'
import type {
  ApiProxyServerConfig,
  ApiProxyServerModel,
  ApiProxyServerStatus,
  ApiProxyServerTrustedHost,
  CreateApiProxyServerModelRequest,
  CreateTrustedHostRequest,
  UpdateApiProxyServerModelRequest,
  UpdateTrustedHostRequest,
} from '../../types'
import { loadAllModelProviders } from './providers.ts'

interface ApiProxyServerState {
  config: ApiProxyServerConfig | null
  status: ApiProxyServerStatus | null
  models: ApiProxyServerModel[]
  trustedHosts: ApiProxyServerTrustedHost[]
  loadingConfig: boolean
  loadingStatus: boolean
  loadingModels: boolean
  loadingHosts: boolean
  error: string | null
  initialized: boolean
  __init__: {
    models: () => Promise<void>
    status?: () => Promise<void>
    config?: () => Promise<void>
    trustedHosts?: () => Promise<void>
  }
}

export const useApiProxyServerStore = create<ApiProxyServerState>()(
  subscribeWithSelector<ApiProxyServerState>((_set, _get) => ({
    config: null,
    status: null,
    models: [],
    trustedHosts: [],
    loadingConfig: false,
    loadingStatus: false,
    loadingModels: false,
    loadingHosts: false,
    error: null,
    initialized: false,
    __init__: {
      models: async () => loadAllModelProviders(),
      status: async () => loadApiProxyServerStatus(),
      config: async () => loadApiProxyServerConfig(),
      trustedHosts: async () => loadApiProxyServerTrustedHosts(),
    },
  })),
)

// Configuration management
export const loadApiProxyServerConfig = async () => {
  const state = useApiProxyServerStore.getState()
  if (state.loadingConfig) {
    return
  }

  useApiProxyServerStore.setState({ loadingConfig: true, error: null })

  try {
    const config = await ApiClient.Admin.getApiProxyServerConfig()
    useApiProxyServerStore.setState({
      config,
      loadingConfig: false,
      error: null,
    })
  } catch (error) {
    console.error('Failed to load API proxy server config:', error)
    useApiProxyServerStore.setState({
      loadingConfig: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const updateApiProxyServerConfig = async (
  configUpdate: ApiProxyServerConfig,
) => {
  const state = useApiProxyServerStore.getState()
  if (state.loadingConfig) {
    return
  }

  useApiProxyServerStore.setState({ loadingConfig: true, error: null })

  try {
    const updatedConfig =
      await ApiClient.Admin.updateApiProxyServerConfig(configUpdate)
    useApiProxyServerStore.setState({
      config: updatedConfig,
      loadingConfig: false,
      error: null,
    })
    return updatedConfig
  } catch (error) {
    console.error('Failed to update API proxy server config:', error)
    useApiProxyServerStore.setState({
      loadingConfig: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

// Status management
export const loadApiProxyServerStatus = async () => {
  const state = useApiProxyServerStore.getState()
  if (state.loadingStatus) {
    return
  }

  useApiProxyServerStore.setState({ loadingStatus: true, error: null })

  try {
    const status = await ApiClient.Admin.getApiProxyServerStatus()
    useApiProxyServerStore.setState({
      status,
      loadingStatus: false,
      error: null,
    })
  } catch (error) {
    console.error('Failed to load API proxy server status:', error)
    useApiProxyServerStore.setState({
      loadingStatus: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const startApiProxyServer = async () => {
  const state = useApiProxyServerStore.getState()
  if (state.loadingStatus) {
    return
  }

  useApiProxyServerStore.setState({ loadingStatus: true, error: null })

  try {
    await ApiClient.Admin.startApiProxyServer()
    // Refresh status after starting
    await loadApiProxyServerStatus()
  } catch (error) {
    console.error('Failed to start API proxy server:', error)
    useApiProxyServerStore.setState({
      loadingStatus: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const stopApiProxyServer = async () => {
  const state = useApiProxyServerStore.getState()
  if (state.loadingStatus) {
    return
  }

  useApiProxyServerStore.setState({ loadingStatus: true, error: null })

  try {
    await ApiClient.Admin.stopApiProxyServer()
    // Refresh status after stopping
    await loadApiProxyServerStatus()
  } catch (error) {
    console.error('Failed to stop API proxy server:', error)
    useApiProxyServerStore.setState({
      loadingStatus: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

// Model management
export const loadApiProxyServerModels = async () => {
  const state = useApiProxyServerStore.getState()
  if (state.loadingModels) {
    return
  }

  useApiProxyServerStore.setState({ loadingModels: true, error: null })

  try {
    const models = await ApiClient.Admin.listApiProxyServerModels()
    useApiProxyServerStore.setState({
      models,
      loadingModels: false,
      error: null,
    })
  } catch (error) {
    console.error('Failed to load API proxy server models:', error)
    useApiProxyServerStore.setState({
      loadingModels: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const addModelToApiProxyServer = async (
  modelData: CreateApiProxyServerModelRequest,
) => {
  try {
    const newModel = await ApiClient.Admin.addModelToApiProxyServer(modelData)
    const currentModels = useApiProxyServerStore.getState().models
    useApiProxyServerStore.setState({
      models: [...currentModels, newModel],
    })
    return newModel
  } catch (error) {
    console.error('Failed to add model to API proxy server:', error)
    throw error
  }
}

export const updateApiProxyServerModel = async (
  modelId: string,
  updates: UpdateApiProxyServerModelRequest,
) => {
  try {
    const updatedModel = await ApiClient.Admin.updateApiProxyServerModel({
      model_id: modelId,
      ...updates,
    })
    const currentModels = useApiProxyServerStore.getState().models
    const updatedModels = currentModels.map(model =>
      model.model_id === modelId ? updatedModel : model,
    )
    useApiProxyServerStore.setState({
      models: updatedModels,
    })
    return updatedModel
  } catch (error) {
    console.error('Failed to update API proxy server model:', error)
    throw error
  }
}

export const removeModelFromApiProxyServer = async (modelId: string) => {
  try {
    await ApiClient.Admin.removeModelFromApiProxyServer({ model_id: modelId })
    const currentModels = useApiProxyServerStore.getState().models
    const filteredModels = currentModels.filter(
      model => model.model_id !== modelId,
    )
    useApiProxyServerStore.setState({
      models: filteredModels,
    })
  } catch (error) {
    console.error('Failed to remove model from API proxy server:', error)
    throw error
  }
}

// Trusted hosts management
export const loadApiProxyServerTrustedHosts = async () => {
  const state = useApiProxyServerStore.getState()
  if (state.loadingHosts) {
    return
  }

  useApiProxyServerStore.setState({ loadingHosts: true, error: null })

  try {
    const trustedHosts = await ApiClient.Admin.listApiProxyServerTrustedHosts()
    useApiProxyServerStore.setState({
      trustedHosts,
      loadingHosts: false,
      error: null,
    })
  } catch (error) {
    console.error('Failed to load API proxy server trusted hosts:', error)
    useApiProxyServerStore.setState({
      loadingHosts: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    })
    throw error
  }
}

export const addTrustedHostToApiProxyServer = async (
  hostData: CreateTrustedHostRequest,
) => {
  try {
    const newHost = await ApiClient.Admin.addApiProxyServerTrustedHost(hostData)
    const currentHosts = useApiProxyServerStore.getState().trustedHosts
    useApiProxyServerStore.setState({
      trustedHosts: [...currentHosts, newHost],
    })
    return newHost
  } catch (error) {
    console.error('Failed to add trusted host to API proxy server:', error)
    throw error
  }
}

export const updateApiProxyServerTrustedHost = async (
  hostId: string,
  updates: UpdateTrustedHostRequest,
) => {
  try {
    const updatedHost = await ApiClient.Admin.updateApiProxyServerTrustedHost({
      host_id: hostId,
      ...updates,
    })
    const currentHosts = useApiProxyServerStore.getState().trustedHosts
    const updatedHosts = currentHosts.map(host =>
      host.id === hostId ? updatedHost : host,
    )
    useApiProxyServerStore.setState({
      trustedHosts: updatedHosts,
    })
    return updatedHost
  } catch (error) {
    console.error('Failed to update API proxy server trusted host:', error)
    throw error
  }
}

export const removeTrustedHostFromApiProxyServer = async (hostId: string) => {
  try {
    await ApiClient.Admin.removeApiProxyServerTrustedHost({ host_id: hostId })
    const currentHosts = useApiProxyServerStore.getState().trustedHosts
    const filteredHosts = currentHosts.filter(host => host.id !== hostId)
    useApiProxyServerStore.setState({
      trustedHosts: filteredHosts,
    })
  } catch (error) {
    console.error('Failed to remove trusted host from API proxy server:', error)
    throw error
  }
}
