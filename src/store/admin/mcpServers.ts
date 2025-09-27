import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { enableMapSet } from 'immer'
import { ApiClient } from '../../api/client'
import type {
  MCPServer,
  MCPExecutionLog,
  CreateSystemMCPServerRequest,
  GroupServerAssignmentResponse,
  GroupAssignmentResponse,
  AssignServersRequest,
} from '../../types/api'

// Enable Map and Set support in Immer
enableMapSet()

interface AdminMCPServersState {
  // System servers data
  systemServers: MCPServer[]
  systemServersTotal: number
  systemServersPage: number
  systemServersPageSize: number
  systemServersInitialized: boolean

  // All execution logs (admin view)
  allExecutionLogs: MCPExecutionLog[]
  executionLogsTotal: number
  executionLogsPage: number
  executionLogsPageSize: number
  executionLogsInitialized: boolean

  // Group assignments
  groupAssignments: GroupServerAssignmentResponse[]
  groupAssignmentsInitialized: boolean

  // Statistics
  executionStatistics: any
  toolStatistics: any[][]
  statisticsInitialized: boolean

  // Loading states
  systemServersLoading: boolean
  executionLogsLoading: boolean
  groupAssignmentsLoading: boolean
  statisticsLoading: boolean
  creating: boolean
  updating: boolean
  deleting: boolean

  // Operation-specific loading states
  operationsLoading: Map<string, boolean>

  // Error states
  systemServersError: string | null
  executionLogsError: string | null
  groupAssignmentsError: string | null
  statisticsError: string | null

  // Initialization methods
  __init__: {
    systemServers: () => Promise<void>
    executionLogs: () => Promise<void>
    groupAssignments: () => Promise<void>
    statistics: () => Promise<void>
  }
}

export const useAdminMCPServersStore = create<AdminMCPServersState>()(
  subscribeWithSelector(
    immer(
      (): AdminMCPServersState => ({
        // System servers data
        systemServers: [],
        systemServersTotal: 0,
        systemServersPage: 1,
        systemServersPageSize: 20,
        systemServersInitialized: false,

        // All execution logs
        allExecutionLogs: [],
        executionLogsTotal: 0,
        executionLogsPage: 1,
        executionLogsPageSize: 50,
        executionLogsInitialized: false,

        // Group assignments
        groupAssignments: [],
        groupAssignmentsInitialized: false,

        // Statistics
        executionStatistics: null,
        toolStatistics: [],
        statisticsInitialized: false,

        // Loading states
        systemServersLoading: false,
        executionLogsLoading: false,
        groupAssignmentsLoading: false,
        statisticsLoading: false,
        creating: false,
        updating: false,
        deleting: false,

        // Operation-specific loading states
        operationsLoading: new Map<string, boolean>(),

        // Error states
        systemServersError: null,
        executionLogsError: null,
        groupAssignmentsError: null,
        statisticsError: null,

        // Initialization methods
        __init__: {
          systemServers: () => loadSystemServers(),
          executionLogs: () => loadAllExecutionLogs(),
          groupAssignments: () => loadGroupAssignments(),
          statistics: () => loadStatistics(),
        },
      }),
    ),
  ),
)

// System servers management
export const loadSystemServers = async (
  page?: number,
  pageSize?: number,
  enabled?: boolean,
): Promise<void> => {
  const state = useAdminMCPServersStore.getState()

  // ✅ CORRECT: Follow admin pattern
  if (state.systemServersInitialized && state.systemServersLoading && !page) {
    return
  }

  try {
    const requestPage = page || state.systemServersPage
    const requestPageSize = pageSize || state.systemServersPageSize

    useAdminMCPServersStore.setState(draft => {
      draft.systemServersLoading = true
      draft.systemServersError = null
    })

    const response = await ApiClient.AdminMcp.listSystemServers({
      page: requestPage,
      per_page: requestPageSize,
      enabled,
    })

    useAdminMCPServersStore.setState(draft => {
      draft.systemServers = response.servers
      draft.systemServersTotal = response.total
      draft.systemServersPage = response.page
      draft.systemServersPageSize = response.per_page
      draft.systemServersInitialized = true
      draft.systemServersLoading = false
      draft.systemServersError = null
    })
  } catch (error) {
    console.error('Failed to load system servers:', error)
    useAdminMCPServersStore.setState(draft => {
      draft.systemServersLoading = false
      draft.systemServersError =
        error instanceof Error ? error.message : 'Failed to load system servers'
    })
    throw error
  }
}

export const createSystemServer = async (
  data: CreateSystemMCPServerRequest,
): Promise<MCPServer> => {
  try {
    useAdminMCPServersStore.setState(draft => {
      draft.creating = true
      draft.systemServersError = null
    })

    const newServer = await ApiClient.AdminMcp.createSystemServer(data)

    useAdminMCPServersStore.setState(draft => {
      draft.systemServers.push(newServer)
      draft.creating = false
    })

    return newServer
  } catch (error) {
    console.error('Failed to create system server:', error)
    useAdminMCPServersStore.setState(draft => {
      draft.creating = false
      draft.systemServersError =
        error instanceof Error
          ? error.message
          : 'Failed to create system server'
    })
    throw error
  }
}







// Execution logs management (admin view of all executions)
export const loadAllExecutionLogs = async (
  page?: number,
  pageSize?: number,
  filters?: {
    serverId?: string
    status?: string
    threadId?: string
  },
): Promise<void> => {
  const state = useAdminMCPServersStore.getState()

  if (state.executionLogsInitialized && state.executionLogsLoading && !page) {
    return
  }

  try {
    const requestPage = page || state.executionLogsPage
    const requestPageSize = pageSize || state.executionLogsPageSize

    useAdminMCPServersStore.setState(draft => {
      draft.executionLogsLoading = true
      draft.executionLogsError = null
    })

    const response = await ApiClient.AdminMcp.listAllExecutionLogs({
      page: requestPage,
      per_page: requestPageSize,
      server_id: filters?.serverId,
      status: filters?.status,
      thread_id: filters?.threadId,
    })

    useAdminMCPServersStore.setState(draft => {
      draft.allExecutionLogs = response.logs
      draft.executionLogsTotal = response.total
      draft.executionLogsPage = response.page
      draft.executionLogsPageSize = response.per_page
      draft.executionLogsInitialized = true
      draft.executionLogsLoading = false
      draft.executionLogsError = null
    })
  } catch (error) {
    console.error('Failed to load all execution logs:', error)
    useAdminMCPServersStore.setState(draft => {
      draft.executionLogsLoading = false
      draft.executionLogsError =
        error instanceof Error ? error.message : 'Failed to load execution logs'
    })
    throw error
  }
}

// Group assignments management
export const loadGroupAssignments = async (): Promise<void> => {
  const state = useAdminMCPServersStore.getState()

  if (state.groupAssignmentsInitialized || state.groupAssignmentsLoading) {
    return
  }

  try {
    useAdminMCPServersStore.setState(draft => {
      draft.groupAssignmentsLoading = true
      draft.groupAssignmentsError = null
    })

    const assignments = await ApiClient.AdminMcp.listAllGroupAssignments()

    useAdminMCPServersStore.setState(draft => {
      draft.groupAssignments = assignments
      draft.groupAssignmentsInitialized = true
      draft.groupAssignmentsLoading = false
      draft.groupAssignmentsError = null
    })
  } catch (error) {
    console.error('Failed to load group assignments:', error)
    useAdminMCPServersStore.setState(draft => {
      draft.groupAssignmentsLoading = false
      draft.groupAssignmentsError =
        error instanceof Error
          ? error.message
          : 'Failed to load group assignments'
    })
    throw error
  }
}

export const getGroupServers = async (groupId: string): Promise<string[]> => {
  try {
    const serverIds = await ApiClient.AdminMcp.getGroupServers({
      group_id: groupId,
    })
    return serverIds
  } catch (error) {
    console.error('Failed to get group servers:', error)
    throw error
  }
}

export const getServerAccessGroups = async (
  serverId: string,
): Promise<string[]> => {
  try {
    const groupIds = await ApiClient.AdminMcp.getServerAccessGroups({
      server_id: serverId,
    })
    return groupIds
  } catch (error) {
    console.error('Failed to get server access groups:', error)
    throw error
  }
}

export const assignServersToGroup = async (
  groupId: string,
  serverIds: string[],
): Promise<GroupAssignmentResponse> => {
  try {
    const request: AssignServersRequest = { server_ids: serverIds }
    const response = await ApiClient.AdminMcp.assignServersToGroup({
      group_id: groupId,
      ...request,
    })

    // Refresh group assignments to reflect changes
    await loadGroupAssignments()

    return response
  } catch (error) {
    console.error('Failed to assign servers to group:', error)
    throw error
  }
}

export const removeServerFromGroup = async (
  groupId: string,
  serverId: string,
): Promise<void> => {
  try {
    await ApiClient.AdminMcp.removeServerFromGroup({
      group_id: groupId,
      server_id: serverId,
    })

    // Refresh group assignments to reflect changes
    await loadGroupAssignments()
  } catch (error) {
    console.error('Failed to remove server from group:', error)
    throw error
  }
}

// Statistics
export const loadStatistics = async (): Promise<void> => {
  const state = useAdminMCPServersStore.getState()

  if (state.statisticsInitialized || state.statisticsLoading) {
    return
  }

  try {
    useAdminMCPServersStore.setState(draft => {
      draft.statisticsLoading = true
      draft.statisticsError = null
    })

    // Load both execution and tool statistics
    const [executionStats, toolStats] = await Promise.all([
      ApiClient.AdminMcp.getExecutionStatistics(),
      ApiClient.AdminMcp.getToolStatistics(),
    ])

    useAdminMCPServersStore.setState(draft => {
      draft.executionStatistics = executionStats
      draft.toolStatistics = toolStats
      draft.statisticsInitialized = true
      draft.statisticsLoading = false
      draft.statisticsError = null
    })
  } catch (error) {
    console.error('Failed to load MCP statistics:', error)
    useAdminMCPServersStore.setState(draft => {
      draft.statisticsLoading = false
      draft.statisticsError =
        error instanceof Error ? error.message : 'Failed to load statistics'
    })
    throw error
  }
}

export const refreshStatistics = async (): Promise<void> => {
  // Force refresh by resetting initialized flag
  useAdminMCPServersStore.setState(draft => {
    draft.statisticsInitialized = false
  })

  await loadStatistics()
}

// Utility functions
export const clearAdminMCPErrors = () => {
  useAdminMCPServersStore.setState(draft => {
    draft.systemServersError = null
    draft.executionLogsError = null
    draft.groupAssignmentsError = null
    draft.statisticsError = null
  })
}

export const refreshSystemServers = async (): Promise<void> => {
  // Don't reset initialized flag, just reload current page
  const { systemServersPage, systemServersPageSize } =
    useAdminMCPServersStore.getState()
  await loadSystemServers(systemServersPage, systemServersPageSize)
}

export const refreshExecutionLogs = async (): Promise<void> => {
  // Don't reset initialized flag, just reload current page
  const { executionLogsPage, executionLogsPageSize } =
    useAdminMCPServersStore.getState()
  await loadAllExecutionLogs(executionLogsPage, executionLogsPageSize)
}

export const isServerOperationLoading = (
  serverId: string,
  operation?: string,
): boolean => {
  const { operationsLoading } = useAdminMCPServersStore.getState()
  const operationKey = operation ? `${serverId}-${operation}` : serverId
  return operationsLoading.get(operationKey) || false
}

export const getSystemServerById = (serverId: string): MCPServer | null => {
  const { systemServers } = useAdminMCPServersStore.getState()
  return systemServers.find(server => server.id === serverId) || null
}

export const getActiveSystemServers = (): MCPServer[] => {
  const { systemServers } = useAdminMCPServersStore.getState()
  return systemServers.filter(server => server.is_active)
}

export const getEnabledSystemServers = (): MCPServer[] => {
  const { systemServers } = useAdminMCPServersStore.getState()
  return systemServers.filter(server => server.enabled)
}

export const searchSystemServers = (query: string): MCPServer[] => {
  const { systemServers } = useAdminMCPServersStore.getState()

  if (!query.trim()) return systemServers

  const searchTerm = query.toLowerCase()
  return systemServers.filter(
    server =>
      server.name.toLowerCase().includes(searchTerm) ||
      server.display_name.toLowerCase().includes(searchTerm) ||
      server.description?.toLowerCase().includes(searchTerm) ||
      server.transport_type.toLowerCase().includes(searchTerm),
  )
}

export const getExecutionLogsByServer = (
  serverId: string,
): MCPExecutionLog[] => {
  const { allExecutionLogs } = useAdminMCPServersStore.getState()
  return allExecutionLogs.filter(log => log.server_id === serverId)
}

export const getExecutionStatsSummary = () => {
  const { allExecutionLogs } = useAdminMCPServersStore.getState()

  const stats = {
    total: allExecutionLogs.length,
    completed: 0,
    failed: 0,
    cancelled: 0,
    running: 0,
    pending: 0,
    timeout: 0,
    byServer: new Map<string, number>(),
  }

  allExecutionLogs.forEach(log => {
    // Count by status
    switch (log.status) {
      case 'completed':
        stats.completed++
        break
      case 'failed':
        stats.failed++
        break
      case 'cancelled':
        stats.cancelled++
        break
      case 'running':
        stats.running++
        break
      case 'pending':
        stats.pending++
        break
      case 'timeout':
        stats.timeout++
        break
    }

    // Count by server
    const serverCount = stats.byServer.get(log.server_id) || 0
    stats.byServer.set(log.server_id, serverCount + 1)
  })

  return stats
}
