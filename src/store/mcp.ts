import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { enableMapSet } from 'immer'
import { ApiClient } from '../api/client'
import type {
  MCPServer,
  MCPTool,
  MCPToolWithServer,
  CreateMCPServerRequest,
  UpdateMCPServerRequest,
  ServerActionResponse,
  ToolDiscoveryResponse,
  SetToolGlobalApprovalRequest,
} from '../types/api'

// Enable Map and Set support in Immer
enableMapSet()

interface MCPState {
  // Server data
  servers: MCPServer[]
  isInitialized: boolean

  // Tools data
  tools: MCPToolWithServer[]
  toolsInitialized: boolean

  // Loading states
  loading: boolean
  toolsLoading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // Error states
  error: string | null
  toolsError: string | null

  // Operation-specific loading states
  operationsLoading: Map<string, boolean>

  // Initialization methods
  __init__: {
    servers: () => Promise<void>
    tools: () => Promise<void>
  }
}

export const useMCPStore = create<MCPState>()(
  subscribeWithSelector(
    immer(
      (): MCPState => ({
        // Server data
        servers: [],
        isInitialized: false,

        // Tools data
        tools: [],
        toolsInitialized: false,

        // Loading states
        loading: false,
        toolsLoading: false,
        creating: false,
        updating: false,
        deleting: false,

        // Error states
        error: null,
        toolsError: null,

        // Operation-specific loading states
        operationsLoading: new Map<string, boolean>(),

        // Initialization methods
        __init__: {
          servers: () => loadMCPServers(),
          tools: () => loadMCPTools(),
        },
      }),
    ),
  ),
)

// Store methods - following current Ziee patterns for loading state management
export const loadMCPServers = async (): Promise<void> => {
  const state = useMCPStore.getState()

  // ✅ CORRECT: Follow assistants.ts pattern - check both isInitialized AND loading
  if (state.isInitialized || state.loading) {
    return
  }

  try {
    useMCPStore.setState(draft => {
      draft.loading = true
      draft.error = null
    })

    const response = await ApiClient.Mcp.listServers({})

    useMCPStore.setState(draft => {
      draft.servers = response.servers
      draft.isInitialized = true
      draft.loading = false
      draft.error = null
    })
  } catch (error) {
    console.error('MCP servers loading failed:', error)
    useMCPStore.setState(draft => {
      draft.loading = false
      draft.error =
        error instanceof Error ? error.message : 'Failed to load MCP servers'
    })
    throw error
  }
}

export const loadMCPTools = async (): Promise<void> => {
  const state = useMCPStore.getState()

  // ✅ CORRECT: Follow rag.ts pattern - check loading state for secondary data
  if (state.toolsLoading) {
    return
  }

  try {
    useMCPStore.setState(draft => {
      draft.toolsLoading = true
      draft.toolsError = null
    })

    const response = await ApiClient.Mcp.listTools({})

    useMCPStore.setState(draft => {
      draft.tools = response.tools
      draft.toolsInitialized = true
      draft.toolsLoading = false
      draft.toolsError = null
    })
  } catch (error) {
    console.error('MCP tools loading failed:', error)
    useMCPStore.setState(draft => {
      draft.toolsLoading = false
      draft.toolsError =
        error instanceof Error ? error.message : 'Failed to load MCP tools'
    })
    throw error
  }
}

export const createMCPServer = async (
  data: CreateMCPServerRequest,
): Promise<MCPServer> => {
  try {
    useMCPStore.setState(draft => {
      draft.creating = true
      draft.error = null
    })

    const newServer = await ApiClient.Mcp.createServer(data)

    useMCPStore.setState(draft => {
      draft.servers.push(newServer)
      draft.creating = false
    })

    return newServer
  } catch (error) {
    console.error('MCP server creation failed:', error)
    useMCPStore.setState(draft => {
      draft.creating = false
      draft.error =
        error instanceof Error ? error.message : 'Failed to create MCP server'
    })
    throw error
  }
}

export const updateMCPServer = async (
  serverId: string,
  data: UpdateMCPServerRequest,
): Promise<MCPServer> => {
  // Set loading for specific server
  useMCPStore.setState(draft => {
    draft.operationsLoading.set(serverId, true)
    draft.error = null
  })

  try {
    const updatedServer = await ApiClient.Mcp.updateServer({
      id: serverId,
      ...data,
    })

    useMCPStore.setState(draft => {
      const index = draft.servers.findIndex(server => server.id === serverId)
      if (index >= 0) {
        draft.servers[index] = updatedServer
      }
      draft.operationsLoading.delete(serverId)
    })

    return updatedServer
  } catch (error) {
    console.error('MCP server update failed:', error)
    useMCPStore.setState(draft => {
      draft.operationsLoading.delete(serverId)
      draft.error =
        error instanceof Error ? error.message : 'Failed to update MCP server'
    })
    throw error
  }
}

export const deleteMCPServer = async (serverId: string): Promise<void> => {
  useMCPStore.setState(draft => {
    draft.operationsLoading.set(serverId, true)
    draft.error = null
  })

  try {
    await ApiClient.Mcp.deleteServer({ id: serverId })

    useMCPStore.setState(draft => {
      draft.servers = draft.servers.filter(server => server.id !== serverId)
      draft.operationsLoading.delete(serverId)
    })
  } catch (error) {
    console.error('MCP server deletion failed:', error)
    useMCPStore.setState(draft => {
      draft.operationsLoading.delete(serverId)
      draft.error =
        error instanceof Error ? error.message : 'Failed to delete MCP server'
    })
    throw error
  }
}

export const startMCPServer = async (
  serverId: string,
): Promise<ServerActionResponse> => {
  useMCPStore.setState(draft => {
    draft.operationsLoading.set(`${serverId}-start`, true)
    draft.error = null
  })

  try {
    const response = await ApiClient.Mcp.startServer({ id: serverId })

    // Update server status in store
    useMCPStore.setState(draft => {
      const index = draft.servers.findIndex(server => server.id === serverId)
      if (index >= 0) {
        draft.servers[index] = {
          ...draft.servers[index],
          is_active: true,
          status: 'running',
        }
      }
      draft.operationsLoading.delete(`${serverId}-start`)
    })

    return response
  } catch (error) {
    console.error('Server start failed:', error)
    useMCPStore.setState(draft => {
      draft.operationsLoading.delete(`${serverId}-start`)
      draft.error =
        error instanceof Error ? error.message : 'Failed to start server'
    })
    throw error
  }
}

export const stopMCPServer = async (
  serverId: string,
): Promise<ServerActionResponse> => {
  useMCPStore.setState(draft => {
    draft.operationsLoading.set(`${serverId}-stop`, true)
    draft.error = null
  })

  try {
    const response = await ApiClient.Mcp.stopServer({ id: serverId })

    // Update server status in store
    useMCPStore.setState(draft => {
      const index = draft.servers.findIndex(server => server.id === serverId)
      if (index >= 0) {
        draft.servers[index] = {
          ...draft.servers[index],
          is_active: false,
          status: 'stopped',
        }
      }
      draft.operationsLoading.delete(`${serverId}-stop`)
    })

    return response
  } catch (error) {
    console.error('Server stop failed:', error)
    useMCPStore.setState(draft => {
      draft.operationsLoading.delete(`${serverId}-stop`)
      draft.error =
        error instanceof Error ? error.message : 'Failed to stop server'
    })
    throw error
  }
}

export const restartMCPServer = async (
  serverId: string,
): Promise<ServerActionResponse> => {
  useMCPStore.setState(draft => {
    draft.operationsLoading.set(`${serverId}-restart`, true)
    draft.error = null
  })

  try {
    const response = await ApiClient.Mcp.restartServer({ id: serverId })

    // Update server status in store
    useMCPStore.setState(draft => {
      const index = draft.servers.findIndex(server => server.id === serverId)
      if (index >= 0) {
        draft.servers[index] = {
          ...draft.servers[index],
          is_active: true,
          status: 'running',
        }
      }
      draft.operationsLoading.delete(`${serverId}-restart`)
    })

    return response
  } catch (error) {
    console.error('Server restart failed:', error)
    useMCPStore.setState(draft => {
      draft.operationsLoading.delete(`${serverId}-restart`)
      draft.error =
        error instanceof Error ? error.message : 'Failed to restart server'
    })
    throw error
  }
}

export const discoverServerTools = async (
  serverId: string,
): Promise<ToolDiscoveryResponse> => {
  useMCPStore.setState(draft => {
    draft.operationsLoading.set(`${serverId}-discover`, true)
    draft.error = null
  })

  try {
    const response = await ApiClient.Mcp.discoverServerTools({ id: serverId })

    // Refresh both servers and tools to get updated counts
    await loadMCPServers()
    await loadMCPTools()

    useMCPStore.setState(draft => {
      draft.operationsLoading.delete(`${serverId}-discover`)
    })

    return response
  } catch (error) {
    console.error('Tool discovery failed:', error)
    useMCPStore.setState(draft => {
      draft.operationsLoading.delete(`${serverId}-discover`)
      draft.error =
        error instanceof Error ? error.message : 'Failed to discover tools'
    })
    throw error
  }
}

export const getMCPServer = async (serverId: string): Promise<MCPServer> => {
  try {
    const server = await ApiClient.Mcp.getServer({ id: serverId })

    // Update server in store if it exists
    useMCPStore.setState(draft => {
      const index = draft.servers.findIndex(s => s.id === serverId)
      if (index >= 0) {
        draft.servers[index] = server
      }
    })

    return server
  } catch (error) {
    console.error('Failed to get MCP server:', error)
    throw error
  }
}

export const getServerTools = async (serverId: string): Promise<MCPTool[]> => {
  try {
    const tools = await ApiClient.Mcp.getServerTools({ id: serverId })
    return tools
  } catch (error) {
    console.error('Failed to get server tools:', error)
    throw error
  }
}

export const getUserAssignedServers = async (): Promise<string[]> => {
  try {
    const serverIds = await ApiClient.Mcp.getUserAssignedServers()
    return serverIds
  } catch (error) {
    console.error('Failed to get user assigned servers:', error)
    throw error
  }
}

export const findToolByName = async (
  preferredServerId?: string,
): Promise<MCPToolWithServer | null> => {
  try {
    const tool = await ApiClient.Mcp.findTool({
      server_id: preferredServerId,
    })
    return tool
  } catch (error) {
    console.error('Failed to find tool:', error)
    throw error
  }
}

export const clearMCPError = () => {
  useMCPStore.setState(draft => {
    draft.error = null
    draft.toolsError = null
  })
}

// Helper functions
export const getUserServers = (servers: MCPServer[]): MCPServer[] => {
  return servers.filter(server => !server.is_system)
}

export const getSystemServers = (servers: MCPServer[]): MCPServer[] => {
  return servers.filter(server => server.is_system)
}

export const getActiveServers = (servers: MCPServer[]): MCPServer[] => {
  return servers.filter(server => server.is_active)
}

export const getServersByType = (
  servers: MCPServer[],
  transportType: string,
): MCPServer[] => {
  return servers.filter(
    server =>
      server.transport_type.toLowerCase() === transportType.toLowerCase(),
  )
}

export const searchServers = (
  servers: MCPServer[],
  query: string,
): MCPServer[] => {
  if (!query.trim()) return servers

  const searchTerm = query.toLowerCase()
  return servers.filter(
    server =>
      server.name.toLowerCase().includes(searchTerm) ||
      server.display_name.toLowerCase().includes(searchTerm) ||
      server.description?.toLowerCase().includes(searchTerm) ||
      server.transport_type.toLowerCase().includes(searchTerm),
  )
}

export const getEnabledServers = (servers: MCPServer[]): MCPServer[] => {
  return servers.filter(server => server.enabled)
}

// Tool-related helper functions
export const getToolsByServer = (
  tools: MCPToolWithServer[],
): Record<string, MCPToolWithServer[]> => {
  return tools.reduce(
    (acc, tool) => {
      const serverName = tool.server_name
      if (!acc[serverName]) {
        acc[serverName] = []
      }
      acc[serverName].push(tool)
      return acc
    },
    {} as Record<string, MCPToolWithServer[]>,
  )
}

export const searchTools = (
  tools: MCPToolWithServer[],
  query: string,
): MCPToolWithServer[] => {
  if (!query.trim()) return tools

  const searchTerm = query.toLowerCase()
  return tools.filter(
    tool =>
      tool.tool_name.toLowerCase().includes(searchTerm) ||
      tool.tool_description?.toLowerCase().includes(searchTerm) ||
      tool.server_name.toLowerCase().includes(searchTerm) ||
      tool.server_display_name.toLowerCase().includes(searchTerm),
  )
}

export const getToolByName = (
  tools: MCPToolWithServer[],
  toolName: string,
  preferredServerId?: string,
): MCPToolWithServer | undefined => {
  const matchingTools = tools.filter(tool => tool.tool_name === toolName)

  if (matchingTools.length === 0) return undefined
  if (matchingTools.length === 1) return matchingTools[0]

  // Handle conflicts: prefer specified server, then user servers, then most-used
  if (preferredServerId) {
    const preferred = matchingTools.find(
      tool => tool.server_id === preferredServerId,
    )
    if (preferred) return preferred
  }

  // Prefer user servers over system servers
  const userTools = matchingTools.filter(tool => !tool.is_system)
  if (userTools.length > 0) {
    return userTools.reduce((prev, current) =>
      current.usage_count > prev.usage_count ? current : prev,
    )
  }

  // Fall back to most-used system tool
  return matchingTools.reduce((prev, current) =>
    current.usage_count > prev.usage_count ? current : prev,
  )
}

export const getToolConflicts = (
  tools: MCPToolWithServer[],
): Record<string, MCPToolWithServer[]> => {
  const toolsByName = tools.reduce(
    (acc, tool) => {
      if (!acc[tool.tool_name]) {
        acc[tool.tool_name] = []
      }
      acc[tool.tool_name].push(tool)
      return acc
    },
    {} as Record<string, MCPToolWithServer[]>,
  )

  return Object.fromEntries(
    Object.entries(toolsByName).filter(([_, tools]) => tools.length > 1),
  )
}

export const getToolsByServerType = (
  tools: MCPToolWithServer[],
  transportType: string,
): MCPToolWithServer[] => {
  return tools.filter(
    tool => tool.transport_type.toLowerCase() === transportType.toLowerCase(),
  )
}

// Tool Approval Methods
export const setToolGlobalApproval = async (
  serverId: string,
  toolName: string,
  request: SetToolGlobalApprovalRequest,
): Promise<void> => {
  try {
    await ApiClient.Mcp.setToolGlobalApproval({
      server_id: serverId,
      tool_name: toolName,
      ...request,
    })

    // Update local tools state with new approval status
    useMCPStore.setState(draft => {
      draft.tools = draft.tools.map(tool => {
        if (tool.server_id === serverId && tool.tool_name === toolName) {
          return {
            ...tool,
            global_auto_approve: request.auto_approve,
          }
        }
        return tool
      })
    })
  } catch (error) {
    console.error('Failed to set tool global approval:', error)
    throw error
  }
}

export const removeToolGlobalApproval = async (
  serverId: string,
  toolName: string,
): Promise<void> => {
  try {
    await ApiClient.Mcp.removeToolGlobalApproval({
      server_id: serverId,
      tool_name: toolName,
    })

    // Update local tools state to remove approval status
    useMCPStore.setState(draft => {
      draft.tools = draft.tools.map(tool => {
        if (tool.server_id === serverId && tool.tool_name === toolName) {
          return {
            ...tool,
            global_auto_approve: undefined,
          }
        }
        return tool
      })
    })
  } catch (error) {
    console.error('Failed to remove tool global approval:', error)
    throw error
  }
}
