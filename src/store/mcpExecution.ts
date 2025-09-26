import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { enableMapSet } from 'immer'
import { ApiClient } from '../api/client'
import { checkToolApproval, createConversationApproval } from './mcpApprovals'
import type {
  ExecuteToolRequest,
  ToolExecutionResponse,
  MCPExecutionLog,
  MCPExecutionStatus,
} from '../types/api'

// Enable Map and Set support in Immer
enableMapSet()

interface MCPExecutionState {
  // Execution logs data
  executionLogs: MCPExecutionLog[]
  executionLogsLoading: boolean
  executionLogsError: string | null
  executionLogsInitialized: boolean

  // Thread-specific logs
  threadLogs: Map<string, MCPExecutionLog[]> // threadId -> logs
  threadLogsLoading: Map<string, boolean>
  threadLogsError: Map<string, string | null>

  // Active executions
  activeExecutions: Map<string, ToolExecutionResponse> // executionId -> response
  executingTools: Map<string, boolean> // toolName -> isExecuting

  // Operation states
  executing: boolean
  cancelling: boolean
  cancellingExecution: Map<string, boolean> // executionId -> isCancelling

  // Real-time execution monitoring
  executionUpdates: Map<string, MCPExecutionLog> // executionId -> latest log

  // Initialization methods
  __init__: {
    executionLogs: () => Promise<void>
  }
}

export const useMCPExecutionStore = create<MCPExecutionState>()(
  subscribeWithSelector(
    immer(
      (): MCPExecutionState => ({
        // Execution logs data
        executionLogs: [],
        executionLogsLoading: false,
        executionLogsError: null,
        executionLogsInitialized: false,

        // Thread-specific logs
        threadLogs: new Map<string, MCPExecutionLog[]>(),
        threadLogsLoading: new Map<string, boolean>(),
        threadLogsError: new Map<string, string | null>(),

        // Active executions
        activeExecutions: new Map<string, ToolExecutionResponse>(),
        executingTools: new Map<string, boolean>(),

        // Operation states
        executing: false,
        cancelling: false,
        cancellingExecution: new Map<string, boolean>(),

        // Real-time execution monitoring
        executionUpdates: new Map<string, MCPExecutionLog>(),

        // Initialization methods
        __init__: {
          executionLogs: () => loadExecutionLogs().then(() => {}),
        },
      }),
    ),
  ),
)

// Store methods - following current Ziee patterns
export const loadExecutionLogs = async (
  page?: number,
  perPage?: number,
  status?: MCPExecutionStatus,
  serverId?: string,
): Promise<MCPExecutionLog[]> => {
  const state = useMCPExecutionStore.getState()

  // ✅ CORRECT: Follow assistants.ts pattern for primary data
  if (state.executionLogsInitialized || state.executionLogsLoading) {
    return state.executionLogs
  }

  try {
    useMCPExecutionStore.setState(draft => {
      draft.executionLogsLoading = true
      draft.executionLogsError = null
    })

    const response = await ApiClient.Mcp.listExecutionLogs({
      page: page || 1,
      per_page: perPage || 50,
      status,
      server_id: serverId,
    })

    useMCPExecutionStore.setState(draft => {
      draft.executionLogs = response.logs
      draft.executionLogsInitialized = true
      draft.executionLogsLoading = false
      draft.executionLogsError = null
    })

    return response.logs
  } catch (error) {
    console.error('Failed to load execution logs:', error)
    useMCPExecutionStore.setState(draft => {
      draft.executionLogsLoading = false
      draft.executionLogsError =
        error instanceof Error ? error.message : 'Failed to load execution logs'
    })
    throw error
  }
}

export const loadThreadExecutionLogs = async (
  threadId: string,
): Promise<MCPExecutionLog[]> => {
  const state = useMCPExecutionStore.getState()

  // Check if already loading this thread
  if (state.threadLogsLoading.get(threadId)) {
    return state.threadLogs.get(threadId) || []
  }

  try {
    useMCPExecutionStore.setState(draft => {
      draft.threadLogsLoading.set(threadId, true)
      draft.threadLogsError.set(threadId, null)
    })

    const logs = await ApiClient.Mcp.listThreadExecutionLogs({
      thread_id: threadId,
    })

    useMCPExecutionStore.setState(draft => {
      draft.threadLogs.set(threadId, logs)
      draft.threadLogsLoading.set(threadId, false)
      draft.threadLogsError.delete(threadId)
    })

    return logs
  } catch (error) {
    console.error('Failed to load thread execution logs:', error)
    useMCPExecutionStore.setState(draft => {
      draft.threadLogsLoading.set(threadId, false)
      draft.threadLogsError.set(
        threadId,
        error instanceof Error ? error.message : 'Failed to load thread logs',
      )
    })
    throw error
  }
}

export const executeTool = async (
  toolName: string,
  parameters: any,
  options: {
    serverId?: string
    conversationId?: string
    autoApprove?: boolean
    requireApproval?: boolean
  } = {},
): Promise<ToolExecutionResponse> => {
  const {
    serverId,
    conversationId,
    autoApprove = false,
    requireApproval = true,
  } = options

  try {
    useMCPExecutionStore.setState(draft => {
      draft.executing = true
      draft.executingTools.set(toolName, true)
    })

    // Check if approval is required and tool needs approval
    if (requireApproval && conversationId && serverId) {
      try {
        const approvalCheck = await checkToolApproval(
          conversationId,
          serverId,
          toolName,
        )

        if (!approvalCheck.approved && !autoApprove) {
          // Tool is not approved - throw error or request approval
          throw new Error(
            `Tool '${toolName}' requires approval for this conversation`,
          )
        }

        // If auto-approve is enabled and tool is not approved, create approval
        if (!approvalCheck.approved && autoApprove) {
          await createConversationApproval(conversationId, {
            server_id: serverId,
            tool_name: toolName,
            approved: true,
          })
        }
      } catch (error) {
        // If approval check fails and we're not auto-approving, re-throw
        if (!autoApprove) {
          throw error
        }
      }
    }

    const request: ExecuteToolRequest = {
      tool_name: toolName,
      parameters,
      server_id: serverId,
      conversation_id: conversationId,
      auto_approve: autoApprove,
    }

    const response = await ApiClient.Mcp.executeTool(request)

    useMCPExecutionStore.setState(draft => {
      draft.activeExecutions.set(response.execution_id, response)
      draft.executing = false
      draft.executingTools.set(toolName, false)
    })

    // Refresh execution logs to include the new execution
    await refreshExecutionLogs()

    return response
  } catch (error) {
    console.error('Failed to execute tool:', error)
    useMCPExecutionStore.setState(draft => {
      draft.executing = false
      draft.executingTools.set(toolName, false)
    })
    throw error
  }
}

export const getExecutionLog = async (
  executionId: string,
): Promise<MCPExecutionLog> => {
  try {
    const log = await ApiClient.Mcp.getExecutionLog({ id: executionId })

    // Update execution updates cache
    useMCPExecutionStore.setState(draft => {
      draft.executionUpdates.set(executionId, log)
    })

    return log
  } catch (error) {
    console.error('Failed to get execution log:', error)
    throw error
  }
}

export const cancelExecution = async (executionId: string): Promise<void> => {
  try {
    useMCPExecutionStore.setState(draft => {
      draft.cancelling = true
      draft.cancellingExecution.set(executionId, true)
    })

    await ApiClient.Mcp.cancelExecution({ id: executionId })

    // Update the execution status
    useMCPExecutionStore.setState(draft => {
      // Update active executions
      const activeExecution = draft.activeExecutions.get(executionId)
      if (activeExecution) {
        activeExecution.status = 'cancelled'
      }

      // Update execution logs if present
      const logIndex = draft.executionLogs.findIndex(
        log => log.id === executionId,
      )
      if (logIndex >= 0) {
        draft.executionLogs[logIndex].status = 'cancelled'
      }

      // Update thread logs
      draft.threadLogs.forEach(logs => {
        const threadLogIndex = logs.findIndex(log => log.id === executionId)
        if (threadLogIndex >= 0) {
          logs[threadLogIndex].status = 'cancelled'
        }
      })

      draft.cancelling = false
      draft.cancellingExecution.delete(executionId)
    })
  } catch (error) {
    console.error('Failed to cancel execution:', error)
    useMCPExecutionStore.setState(draft => {
      draft.cancelling = false
      draft.cancellingExecution.delete(executionId)
    })
    throw error
  }
}

export const refreshExecutionLogs = async (): Promise<void> => {
  const state = useMCPExecutionStore.getState()

  // Don't refresh if not initialized yet
  if (!state.executionLogsInitialized) {
    return
  }

  try {
    useMCPExecutionStore.setState(draft => {
      draft.executionLogsLoading = true
      draft.executionLogsError = null
    })

    const response = await ApiClient.Mcp.listExecutionLogs({
      page: 1,
      per_page: 50,
    })

    useMCPExecutionStore.setState(draft => {
      draft.executionLogs = response.logs
      draft.executionLogsLoading = false
      draft.executionLogsError = null
    })
  } catch (error) {
    console.error('Failed to refresh execution logs:', error)
    useMCPExecutionStore.setState(draft => {
      draft.executionLogsLoading = false
      draft.executionLogsError =
        error instanceof Error
          ? error.message
          : 'Failed to refresh execution logs'
    })
  }
}

export const updateExecutionStatus = (
  executionId: string,
  log: MCPExecutionLog,
): void => {
  useMCPExecutionStore.setState(draft => {
    // Update execution updates cache
    draft.executionUpdates.set(executionId, log)

    // Update active executions
    const activeExecution = draft.activeExecutions.get(executionId)
    if (activeExecution) {
      activeExecution.status = log.status
      if (log.error_message) {
        activeExecution.error_message = log.error_message
      }
      if (log.duration_ms) {
        activeExecution.duration_ms = log.duration_ms
      }
      if (log.execution_result) {
        activeExecution.result = log.execution_result
      }
    }

    // Update execution logs if present
    const logIndex = draft.executionLogs.findIndex(
      existingLog => existingLog.id === executionId,
    )
    if (logIndex >= 0) {
      draft.executionLogs[logIndex] = log
    }

    // Update thread logs
    draft.threadLogs.forEach(logs => {
      const threadLogIndex = logs.findIndex(
        existingLog => existingLog.id === executionId,
      )
      if (threadLogIndex >= 0) {
        logs[threadLogIndex] = log
      }
    })

    // Clear executing state if execution completed
    if (['completed', 'failed', 'cancelled', 'timeout'].includes(log.status)) {
      draft.executingTools.set(log.tool_name, false)
    }
  })
}

// Utility functions
export const clearExecutionError = () => {
  useMCPExecutionStore.setState(draft => {
    draft.executionLogsError = null
    draft.threadLogsError.clear()
  })
}

export const getExecutionLogById = (
  executionId: string,
): MCPExecutionLog | null => {
  const { executionLogs, executionUpdates } = useMCPExecutionStore.getState()

  // Check execution updates first for most recent data
  const realtimeLog = executionUpdates.get(executionId)
  if (realtimeLog) {
    return realtimeLog
  }

  // Fall back to execution logs
  return executionLogs.find(log => log.id === executionId) || null
}

export const getExecutionLogsByStatus = (
  status: MCPExecutionStatus,
): MCPExecutionLog[] => {
  const { executionLogs } = useMCPExecutionStore.getState()
  return executionLogs.filter(log => log.status === status)
}

export const getExecutionLogsByTool = (
  toolName: string,
  serverId?: string,
): MCPExecutionLog[] => {
  const { executionLogs } = useMCPExecutionStore.getState()
  return executionLogs.filter(
    log =>
      log.tool_name === toolName && (!serverId || log.server_id === serverId),
  )
}

export const getActiveExecutions = (): ToolExecutionResponse[] => {
  const { activeExecutions } = useMCPExecutionStore.getState()
  return Array.from(activeExecutions.values()).filter(execution =>
    ['pending', 'running'].includes(execution.status),
  )
}

export const isToolExecuting = (toolName: string): boolean => {
  const { executingTools } = useMCPExecutionStore.getState()
  return executingTools.get(toolName) || false
}

export const getThreadLogs = (threadId: string): MCPExecutionLog[] => {
  const { threadLogs } = useMCPExecutionStore.getState()
  return threadLogs.get(threadId) || []
}

export const isExecutionCancelling = (executionId: string): boolean => {
  const { cancellingExecution } = useMCPExecutionStore.getState()
  return cancellingExecution.get(executionId) || false
}

export const searchExecutionLogs = (
  logs: MCPExecutionLog[],
  query: string,
): MCPExecutionLog[] => {
  if (!query.trim()) return logs

  const searchTerm = query.toLowerCase()
  return logs.filter(
    log =>
      log.tool_name.toLowerCase().includes(searchTerm) ||
      log.server_id.toLowerCase().includes(searchTerm) ||
      log.error_message?.toLowerCase().includes(searchTerm) ||
      log.status.toLowerCase().includes(searchTerm),
  )
}

export const filterExecutionLogsByDateRange = (
  logs: MCPExecutionLog[],
  startDate?: Date,
  endDate?: Date,
): MCPExecutionLog[] => {
  if (!startDate && !endDate) return logs

  return logs.filter(log => {
    const logDate = new Date(log.started_at)

    if (startDate && logDate < startDate) return false
    if (endDate && logDate > endDate) return false

    return true
  })
}

export const getExecutionStats = () => {
  const { executionLogs } = useMCPExecutionStore.getState()

  const stats = {
    total: executionLogs.length,
    completed: 0,
    failed: 0,
    cancelled: 0,
    running: 0,
    pending: 0,
    timeout: 0,
  }

  executionLogs.forEach(log => {
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
  })

  return stats
}

// Real-time execution monitoring helpers
export const startExecutionMonitoring = (executionId: string): void => {
  // This would typically set up WebSocket or polling for real-time updates
  // For now, we'll implement basic polling
  const pollExecution = async () => {
    try {
      const log = await getExecutionLog(executionId)
      updateExecutionStatus(executionId, log)

      // Continue polling if execution is still active
      if (['pending', 'running'].includes(log.status)) {
        setTimeout(pollExecution, 2000) // Poll every 2 seconds
      }
    } catch (error) {
      console.error('Failed to poll execution status:', error)
    }
  }

  // Start polling
  pollExecution()
}

export const clearExecutionUpdatesCache = (): void => {
  useMCPExecutionStore.setState(draft => {
    draft.executionUpdates.clear()
  })
}
