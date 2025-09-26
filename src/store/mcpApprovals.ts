import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { enableMapSet } from 'immer'
import { ApiClient } from '../api/client'
import type {
  ToolApprovalResponse,
  CreateConversationApprovalRequest,
  UpdateToolApprovalRequest,
  SetToolGlobalApprovalRequest,
  ListConversationApprovalsQuery,
  DeleteConversationApprovalQuery,
} from '../types/api'

// Enable Map and Set support in Immer
enableMapSet()

interface MCPApprovalsState {
  // Global approvals data
  globalApprovals: Map<string, ToolApprovalResponse> // serverId-toolName -> approval
  globalApprovalsLoading: boolean
  globalApprovalsError: string | null
  isInitialized: boolean

  // Conversation approvals data
  conversationApprovals: Map<string, ToolApprovalResponse[]> // conversationId -> approvals
  conversationApprovalsLoading: boolean
  conversationApprovalsError: string | null

  // Operation loading states
  settingGlobalApproval: boolean
  updatingApproval: boolean
  deletingApproval: boolean
  checkingApproval: boolean

  // Approval checks cache
  approvalChecks: Map<string, { approved: boolean; source: string | null }> // conversationId-serverId-toolName -> result

  __init__: {
    globalApprovals: () => Promise<void>
  }
}

export const useMCPApprovalsStore = create<MCPApprovalsState>()(
  subscribeWithSelector(
    immer(
      (): MCPApprovalsState => ({
        // Initial state
        globalApprovals: new Map<string, ToolApprovalResponse>(),
        globalApprovalsLoading: false,
        globalApprovalsError: null,
        isInitialized: false,
        conversationApprovals: new Map<string, ToolApprovalResponse[]>(),
        conversationApprovalsLoading: false,
        conversationApprovalsError: null,
        settingGlobalApproval: false,
        updatingApproval: false,
        deletingApproval: false,
        checkingApproval: false,
        approvalChecks: new Map<
          string,
          { approved: boolean; source: string | null }
        >(),
        __init__: {
          globalApprovals: () => loadAllGlobalApprovals(),
        },
      }),
    ),
  ),
)

// Store methods - following current Ziee patterns
export const loadAllGlobalApprovals = async (): Promise<void> => {
  const state = useMCPApprovalsStore.getState()

  // ✅ CORRECT: Follow assistants.ts pattern
  if (state.isInitialized || state.globalApprovalsLoading) {
    return
  }

  try {
    useMCPApprovalsStore.setState(draft => {
      draft.globalApprovalsLoading = true
      draft.globalApprovalsError = null
    })

    // Load user's servers first to get global approvals
    const serversResponse = await ApiClient.Mcp.listServers({})
    const globalApprovals = new Map<string, ToolApprovalResponse>()

    // For each server, try to get global approvals for tools
    for (const server of serversResponse.servers) {
      try {
        const tools = await ApiClient.Mcp.getServerTools({ id: server.id })
        for (const tool of tools) {
          try {
            const approval = await ApiClient.Mcp.getGlobalToolApproval({
              server_id: server.id,
              tool_name: tool.tool_name,
            })
            const key = `${server.id}-${tool.tool_name}`
            globalApprovals.set(key, approval)
          } catch (error) {
            // No global approval exists for this tool, which is fine
          }
        }
      } catch (error) {
        console.warn(`Failed to load tools for server ${server.id}:`, error)
      }
    }

    useMCPApprovalsStore.setState(draft => {
      draft.globalApprovals = globalApprovals
      draft.globalApprovalsLoading = false
      draft.isInitialized = true
    })
  } catch (error) {
    console.error('Failed to load global approvals:', error)
    useMCPApprovalsStore.setState(draft => {
      draft.globalApprovalsLoading = false
      draft.globalApprovalsError =
        error instanceof Error ? error.message : 'Failed to load approvals'
    })
    throw error
  }
}

export const setGlobalToolApproval = async (
  serverId: string,
  toolName: string,
  request: SetToolGlobalApprovalRequest,
): Promise<ToolApprovalResponse> => {
  try {
    useMCPApprovalsStore.setState(draft => {
      draft.settingGlobalApproval = true
      draft.globalApprovalsError = null
    })

    const approval = await ApiClient.Mcp.setToolGlobalApproval({
      server_id: serverId,
      tool_name: toolName,
      ...request,
    })
    const key = `${serverId}-${toolName}`

    useMCPApprovalsStore.setState(draft => {
      draft.globalApprovals.set(key, approval)
      draft.settingGlobalApproval = false
    })

    return approval
  } catch (error) {
    console.error('Failed to set global approval:', error)
    useMCPApprovalsStore.setState(draft => {
      draft.settingGlobalApproval = false
      draft.globalApprovalsError =
        error instanceof Error ? error.message : 'Failed to set approval'
    })
    throw error
  }
}

export const removeGlobalToolApproval = async (
  serverId: string,
  toolName: string,
): Promise<void> => {
  try {
    useMCPApprovalsStore.setState(draft => {
      draft.deletingApproval = true
      draft.globalApprovalsError = null
    })

    await ApiClient.Mcp.removeToolGlobalApproval({
      server_id: serverId,
      tool_name: toolName,
    })
    const key = `${serverId}-${toolName}`

    useMCPApprovalsStore.setState(draft => {
      draft.globalApprovals.delete(key)
      draft.deletingApproval = false
    })
  } catch (error) {
    console.error('Failed to remove global approval:', error)
    useMCPApprovalsStore.setState(draft => {
      draft.deletingApproval = false
      draft.globalApprovalsError =
        error instanceof Error ? error.message : 'Failed to remove approval'
    })
    throw error
  }
}

export const loadConversationApprovals = async (
  conversationId: string,
  query?: ListConversationApprovalsQuery,
): Promise<ToolApprovalResponse[]> => {
  const state = useMCPApprovalsStore.getState()

  // Check if already loading this conversation
  if (state.conversationApprovalsLoading) {
    return state.conversationApprovals.get(conversationId) || []
  }

  try {
    useMCPApprovalsStore.setState(draft => {
      draft.conversationApprovalsLoading = true
      draft.conversationApprovalsError = null
    })

    const approvals = await ApiClient.Mcp.listConversationApprovals({
      conversation_id: conversationId,
      ...(query || {}),
    })

    useMCPApprovalsStore.setState(draft => {
      draft.conversationApprovals.set(conversationId, approvals)
      draft.conversationApprovalsLoading = false
    })

    return approvals
  } catch (error) {
    console.error('Failed to load conversation approvals:', error)
    useMCPApprovalsStore.setState(draft => {
      draft.conversationApprovalsLoading = false
      draft.conversationApprovalsError =
        error instanceof Error ? error.message : 'Failed to load approvals'
    })
    throw error
  }
}

export const createConversationApproval = async (
  conversationId: string,
  request: CreateConversationApprovalRequest,
): Promise<ToolApprovalResponse> => {
  try {
    const approval = await ApiClient.Mcp.createConversationApproval({
      conversation_id: conversationId,
      ...request,
    })

    useMCPApprovalsStore.setState(draft => {
      const currentApprovals =
        draft.conversationApprovals.get(conversationId) || []
      const updatedApprovals = [
        ...currentApprovals.filter(
          a =>
            !(
              a.server_id === request.server_id &&
              a.tool_name === request.tool_name
            ),
        ),
        approval,
      ]

      draft.conversationApprovals.set(conversationId, updatedApprovals)
    })

    return approval
  } catch (error) {
    console.error('Failed to create conversation approval:', error)
    throw error
  }
}

export const deleteConversationApproval = async (
  conversationId: string,
  query: DeleteConversationApprovalQuery,
): Promise<void> => {
  try {
    await ApiClient.Mcp.deleteConversationApproval({
      conversation_id: conversationId,
      ...query,
    })

    useMCPApprovalsStore.setState(draft => {
      const currentApprovals =
        draft.conversationApprovals.get(conversationId) || []
      const updatedApprovals = currentApprovals.filter(
        a =>
          !(a.server_id === query.server_id && a.tool_name === query.tool_name),
      )

      draft.conversationApprovals.set(conversationId, updatedApprovals)
    })
  } catch (error) {
    console.error('Failed to delete conversation approval:', error)
    throw error
  }
}

export const checkToolApproval = async (
  conversationId: string,
  serverId: string,
  toolName: string,
): Promise<{ approved: boolean; source: string | null }> => {
  const cacheKey = `${conversationId}-${serverId}-${toolName}`
  const state = useMCPApprovalsStore.getState()

  // Return cached result if available
  if (state.approvalChecks.has(cacheKey)) {
    return state.approvalChecks.get(cacheKey)!
  }

  try {
    useMCPApprovalsStore.setState(draft => {
      draft.checkingApproval = true
    })

    const result = await ApiClient.Mcp.checkConversationApproval({
      conversation_id: conversationId,
      server_id: serverId,
      tool_name: toolName,
    })

    useMCPApprovalsStore.setState(draft => {
      draft.approvalChecks.set(cacheKey, {
        approved: result.approved,
        source: result.source || null,
      })
      draft.checkingApproval = false
    })

    return {
      approved: result.approved,
      source: result.source || null,
    }
  } catch (error) {
    console.error('Failed to check tool approval:', error)
    useMCPApprovalsStore.setState(draft => {
      draft.checkingApproval = false
    })
    throw error
  }
}

export const updateToolApproval = async (
  approvalId: string,
  request: UpdateToolApprovalRequest,
): Promise<ToolApprovalResponse> => {
  try {
    useMCPApprovalsStore.setState(draft => {
      draft.updatingApproval = true
      draft.globalApprovalsError = null
    })

    const updatedApproval = await ApiClient.Mcp.updateToolApproval({
      approval_id: approvalId,
      ...request,
    })

    // Update in global approvals if it's a global approval
    if (updatedApproval.is_global) {
      const key = `${updatedApproval.server_id}-${updatedApproval.tool_name}`
      useMCPApprovalsStore.setState(draft => {
        draft.globalApprovals.set(key, updatedApproval)
        draft.updatingApproval = false
      })
    } else {
      // Update in conversation approvals if it exists
      const conversationId = updatedApproval.conversation_id
      if (conversationId) {
        useMCPApprovalsStore.setState(draft => {
          const currentApprovals =
            draft.conversationApprovals.get(conversationId) || []
          const updatedApprovals = currentApprovals.map(a =>
            a.id === approvalId ? updatedApproval : a,
          )

          draft.conversationApprovals.set(conversationId, updatedApprovals)
          draft.updatingApproval = false
        })
      }
    }

    return updatedApproval
  } catch (error) {
    console.error('Failed to update tool approval:', error)
    useMCPApprovalsStore.setState(draft => {
      draft.updatingApproval = false
      draft.globalApprovalsError =
        error instanceof Error ? error.message : 'Failed to update approval'
    })
    throw error
  }
}

export const cleanExpiredApprovals = async (): Promise<{
  cleaned_count: number
  message: string
}> => {
  try {
    const result = await ApiClient.Mcp.cleanExpiredApprovals()

    // Refresh global approvals to remove any cleaned ones
    await loadAllGlobalApprovals()

    // Clear conversation approval caches since they may be affected
    useMCPApprovalsStore.setState(draft => {
      draft.conversationApprovals.clear()
      draft.approvalChecks.clear() // Clear approval check cache
    })

    return result
  } catch (error) {
    console.error('Failed to clean expired approvals:', error)
    throw error
  }
}

// Utility functions
export const clearApprovalsError = () => {
  useMCPApprovalsStore.setState(draft => {
    draft.globalApprovalsError = null
    draft.conversationApprovalsError = null
  })
}

export const isToolGloballyApproved = (
  serverId: string,
  toolName: string,
): boolean => {
  const { globalApprovals } = useMCPApprovalsStore.getState()
  const key = `${serverId}-${toolName}`
  const approval = globalApprovals.get(key)
  return approval
    ? approval.auto_approve && approval.approved && !approval.is_expired
    : false
}

export const getGlobalApprovalForTool = (
  serverId: string,
  toolName: string,
): ToolApprovalResponse | null => {
  const { globalApprovals } = useMCPApprovalsStore.getState()
  const key = `${serverId}-${toolName}`
  return globalApprovals.get(key) || null
}

export const getConversationApprovals = (
  conversationId: string,
): ToolApprovalResponse[] => {
  const { conversationApprovals } = useMCPApprovalsStore.getState()
  return conversationApprovals.get(conversationId) || []
}

export const isToolApprovedForConversation = (
  conversationId: string,
  serverId: string,
  toolName: string,
): boolean => {
  const approvals = getConversationApprovals(conversationId)
  const approval = approvals.find(
    a =>
      a.server_id === serverId &&
      a.tool_name === toolName &&
      a.approved &&
      !a.is_expired,
  )
  return !!approval
}

export const clearApprovalChecksCache = () => {
  useMCPApprovalsStore.setState(draft => {
    draft.approvalChecks.clear()
  })
}
