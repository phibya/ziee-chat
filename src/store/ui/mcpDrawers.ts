import { create } from 'zustand'
import type {
  MCPServer,
  MCPToolWithServer,
  MCPExecutionLog,
  ToolApprovalResponse,
} from '../../types/api'

// MCP Server Drawer State
interface MCPServerDrawerState {
  open: boolean
  loading: boolean
  editingServer: MCPServer | null
  isCloning: boolean
  mode: 'create' | 'edit' | 'clone' | 'create-system' | 'edit-system'
}

export const useMCPServerDrawerStore = create<MCPServerDrawerState>(() => ({
  open: false,
  loading: false,
  editingServer: null,
  isCloning: false,
  mode: 'create',
}))

// MCP Server Drawer Actions
export const openMCPServerDrawer = (
  server?: MCPServer,
  mode: 'create' | 'edit' | 'clone' | 'create-system' | 'edit-system' = 'create',
) => {
  useMCPServerDrawerStore.setState({
    open: true,
    editingServer: server || null,
    isCloning: mode === 'clone',
    mode,
  })
}

export const closeMCPServerDrawer = () => {
  useMCPServerDrawerStore.setState({
    open: false,
    loading: false,
    editingServer: null,
    isCloning: false,
    mode: 'create',
  })
}

export const setMCPServerDrawerLoading = (loading: boolean) => {
  useMCPServerDrawerStore.setState({ loading })
}

// MCP Tool Details Drawer State
interface MCPToolDetailsDrawerState {
  open: boolean
  loading: boolean
  selectedTool: MCPToolWithServer | null
  showSchema: boolean
  showExecutionHistory: boolean
}

export const useMCPToolDetailsDrawerStore = create<MCPToolDetailsDrawerState>(
  () => ({
    open: false,
    loading: false,
    selectedTool: null,
    showSchema: false,
    showExecutionHistory: false,
  }),
)

// MCP Tool Details Drawer Actions
export const openMCPToolDetailsDrawer = (
  tool: MCPToolWithServer,
  options: { showSchema?: boolean; showExecutionHistory?: boolean } = {},
) => {
  useMCPToolDetailsDrawerStore.setState({
    open: true,
    selectedTool: tool,
    showSchema: options.showSchema || false,
    showExecutionHistory: options.showExecutionHistory || false,
  })
}

export const closeMCPToolDetailsDrawer = () => {
  useMCPToolDetailsDrawerStore.setState({
    open: false,
    loading: false,
    selectedTool: null,
    showSchema: false,
    showExecutionHistory: false,
  })
}

export const setMCPToolDetailsDrawerLoading = (loading: boolean) => {
  useMCPToolDetailsDrawerStore.setState({ loading })
}

export const toggleMCPToolSchema = () => {
  const { showSchema } = useMCPToolDetailsDrawerStore.getState()
  useMCPToolDetailsDrawerStore.setState({
    showSchema: !showSchema,
  })
}

export const toggleMCPToolExecutionHistory = () => {
  const { showExecutionHistory } = useMCPToolDetailsDrawerStore.getState()
  useMCPToolDetailsDrawerStore.setState({
    showExecutionHistory: !showExecutionHistory,
  })
}

// MCP Tool Execution Drawer State
interface MCPToolExecutionDrawerState {
  open: boolean
  loading: boolean
  executing: boolean
  selectedTool: MCPToolWithServer | null
  parameters: Record<string, any>
  conversationId?: string
  autoApprove: boolean
  requireApproval: boolean
  executionResult?: any
  executionError?: string
}

export const useMCPToolExecutionDrawerStore =
  create<MCPToolExecutionDrawerState>(() => ({
    open: false,
    loading: false,
    executing: false,
    selectedTool: null,
    parameters: {},
    autoApprove: false,
    requireApproval: true,
    executionResult: undefined,
    executionError: undefined,
  }))

// MCP Tool Execution Drawer Actions
export const openMCPToolExecutionDrawer = (
  tool: MCPToolWithServer,
  options: {
    conversationId?: string
    autoApprove?: boolean
    requireApproval?: boolean
    initialParameters?: Record<string, any>
  } = {},
) => {
  useMCPToolExecutionDrawerStore.setState({
    open: true,
    selectedTool: tool,
    conversationId: options.conversationId,
    autoApprove: options.autoApprove || false,
    requireApproval: options.requireApproval !== false, // Default to true
    parameters: options.initialParameters || {},
    executionResult: undefined,
    executionError: undefined,
  })
}

export const closeMCPToolExecutionDrawer = () => {
  useMCPToolExecutionDrawerStore.setState({
    open: false,
    loading: false,
    executing: false,
    selectedTool: null,
    parameters: {},
    conversationId: undefined,
    autoApprove: false,
    requireApproval: true,
    executionResult: undefined,
    executionError: undefined,
  })
}

export const setMCPToolExecutionDrawerLoading = (loading: boolean) => {
  useMCPToolExecutionDrawerStore.setState({ loading })
}

export const setMCPToolExecutionDrawerExecuting = (executing: boolean) => {
  useMCPToolExecutionDrawerStore.setState({ executing })
}

export const updateMCPToolExecutionParameters = (
  parameters: Record<string, any>,
) => {
  useMCPToolExecutionDrawerStore.setState({ parameters })
}

export const setMCPToolExecutionResult = (result: any, error?: string) => {
  useMCPToolExecutionDrawerStore.setState({
    executionResult: result,
    executionError: error,
    executing: false,
  })
}

export const toggleMCPToolAutoApprove = () => {
  const { autoApprove } = useMCPToolExecutionDrawerStore.getState()
  useMCPToolExecutionDrawerStore.setState({
    autoApprove: !autoApprove,
  })
}

export const toggleMCPToolRequireApproval = () => {
  const { requireApproval } = useMCPToolExecutionDrawerStore.getState()
  useMCPToolExecutionDrawerStore.setState({
    requireApproval: !requireApproval,
  })
}

// MCP Execution Logs Drawer State
interface MCPExecutionLogsDrawerState {
  open: boolean
  loading: boolean
  selectedExecution: MCPExecutionLog | null
  showDetails: boolean
  showParameters: boolean
  showResult: boolean
  allowCancel: boolean
}

export const useMCPExecutionLogsDrawerStore =
  create<MCPExecutionLogsDrawerState>(() => ({
    open: false,
    loading: false,
    selectedExecution: null,
    showDetails: true,
    showParameters: false,
    showResult: false,
    allowCancel: false,
  }))

// MCP Execution Logs Drawer Actions
export const openMCPExecutionLogsDrawer = (
  execution: MCPExecutionLog,
  options: {
    showDetails?: boolean
    showParameters?: boolean
    showResult?: boolean
    allowCancel?: boolean
  } = {},
) => {
  useMCPExecutionLogsDrawerStore.setState({
    open: true,
    selectedExecution: execution,
    showDetails: options.showDetails !== false, // Default to true
    showParameters: options.showParameters || false,
    showResult: options.showResult || false,
    allowCancel:
      options.allowCancel || ['pending', 'running'].includes(execution.status),
  })
}

export const closeMCPExecutionLogsDrawer = () => {
  useMCPExecutionLogsDrawerStore.setState({
    open: false,
    loading: false,
    selectedExecution: null,
    showDetails: true,
    showParameters: false,
    showResult: false,
    allowCancel: false,
  })
}

export const setMCPExecutionLogsDrawerLoading = (loading: boolean) => {
  useMCPExecutionLogsDrawerStore.setState({ loading })
}

export const toggleMCPExecutionDetails = () => {
  const { showDetails } = useMCPExecutionLogsDrawerStore.getState()
  useMCPExecutionLogsDrawerStore.setState({
    showDetails: !showDetails,
  })
}

export const toggleMCPExecutionParameters = () => {
  const { showParameters } = useMCPExecutionLogsDrawerStore.getState()
  useMCPExecutionLogsDrawerStore.setState({
    showParameters: !showParameters,
  })
}

export const toggleMCPExecutionResult = () => {
  const { showResult } = useMCPExecutionLogsDrawerStore.getState()
  useMCPExecutionLogsDrawerStore.setState({
    showResult: !showResult,
  })
}

export const updateMCPExecutionInDrawer = (execution: MCPExecutionLog) => {
  const { selectedExecution } = useMCPExecutionLogsDrawerStore.getState()

  if (selectedExecution && selectedExecution.id === execution.id) {
    useMCPExecutionLogsDrawerStore.setState({
      selectedExecution: execution,
      allowCancel: ['pending', 'running'].includes(execution.status),
    })
  }
}

// MCP Approvals Management Drawer State
interface MCPApprovalsDrawerState {
  open: boolean
  loading: boolean
  selectedApproval: ToolApprovalResponse | null
  mode: 'view' | 'edit' | 'create'
  conversationId?: string
  serverId?: string
  toolName?: string
  showGlobalApprovals: boolean
  showConversationApprovals: boolean
}

export const useMCPApprovalsDrawerStore = create<MCPApprovalsDrawerState>(
  () => ({
    open: false,
    loading: false,
    selectedApproval: null,
    mode: 'view',
    conversationId: undefined,
    serverId: undefined,
    toolName: undefined,
    showGlobalApprovals: true,
    showConversationApprovals: true,
  }),
)

// MCP Approvals Management Drawer Actions
export const openMCPApprovalsDrawer = (
  options: {
    approval?: ToolApprovalResponse
    mode?: 'view' | 'edit' | 'create'
    conversationId?: string
    serverId?: string
    toolName?: string
    showGlobalApprovals?: boolean
    showConversationApprovals?: boolean
  } = {},
) => {
  useMCPApprovalsDrawerStore.setState({
    open: true,
    selectedApproval: options.approval || null,
    mode: options.mode || 'view',
    conversationId: options.conversationId,
    serverId: options.serverId,
    toolName: options.toolName,
    showGlobalApprovals: options.showGlobalApprovals !== false, // Default to true
    showConversationApprovals: options.showConversationApprovals !== false, // Default to true
  })
}

export const closeMCPApprovalsDrawer = () => {
  useMCPApprovalsDrawerStore.setState({
    open: false,
    loading: false,
    selectedApproval: null,
    mode: 'view',
    conversationId: undefined,
    serverId: undefined,
    toolName: undefined,
    showGlobalApprovals: true,
    showConversationApprovals: true,
  })
}

export const setMCPApprovalsDrawerLoading = (loading: boolean) => {
  useMCPApprovalsDrawerStore.setState({ loading })
}

export const setMCPApprovalsDrawerMode = (mode: 'view' | 'edit' | 'create') => {
  useMCPApprovalsDrawerStore.setState({ mode })
}

export const toggleMCPGlobalApprovals = () => {
  const { showGlobalApprovals } = useMCPApprovalsDrawerStore.getState()
  useMCPApprovalsDrawerStore.setState({
    showGlobalApprovals: !showGlobalApprovals,
  })
}

export const toggleMCPConversationApprovals = () => {
  const { showConversationApprovals } = useMCPApprovalsDrawerStore.getState()
  useMCPApprovalsDrawerStore.setState({
    showConversationApprovals: !showConversationApprovals,
  })
}

export const updateMCPApprovalInDrawer = (approval: ToolApprovalResponse) => {
  const { selectedApproval } = useMCPApprovalsDrawerStore.getState()

  if (selectedApproval && selectedApproval.id === approval.id) {
    useMCPApprovalsDrawerStore.setState({
      selectedApproval: approval,
    })
  }
}

// MCP Server Management Drawer State (for admin operations)
interface MCPServerManagementDrawerState {
  open: boolean
  loading: boolean
  selectedServer: MCPServer | null
  activeTab: 'details' | 'tools' | 'logs' | 'approvals' | 'permissions'
  showSystemServers: boolean
  showUserServers: boolean
}

export const useMCPServerManagementDrawerStore =
  create<MCPServerManagementDrawerState>(() => ({
    open: false,
    loading: false,
    selectedServer: null,
    activeTab: 'details',
    showSystemServers: true,
    showUserServers: true,
  }))

// MCP Server Management Drawer Actions
export const openMCPServerManagementDrawer = (
  server: MCPServer,
  options: {
    activeTab?: 'details' | 'tools' | 'logs' | 'approvals' | 'permissions'
    showSystemServers?: boolean
    showUserServers?: boolean
  } = {},
) => {
  useMCPServerManagementDrawerStore.setState({
    open: true,
    selectedServer: server,
    activeTab: options.activeTab || 'details',
    showSystemServers: options.showSystemServers !== false, // Default to true
    showUserServers: options.showUserServers !== false, // Default to true
  })
}

export const closeMCPServerManagementDrawer = () => {
  useMCPServerManagementDrawerStore.setState({
    open: false,
    loading: false,
    selectedServer: null,
    activeTab: 'details',
    showSystemServers: true,
    showUserServers: true,
  })
}

export const setMCPServerManagementDrawerLoading = (loading: boolean) => {
  useMCPServerManagementDrawerStore.setState({ loading })
}

export const setMCPServerManagementActiveTab = (
  activeTab: 'details' | 'tools' | 'logs' | 'approvals' | 'permissions',
) => {
  useMCPServerManagementDrawerStore.setState({ activeTab })
}

export const updateMCPServerInManagementDrawer = (server: MCPServer) => {
  const { selectedServer } = useMCPServerManagementDrawerStore.getState()

  if (selectedServer && selectedServer.id === server.id) {
    useMCPServerManagementDrawerStore.setState({
      selectedServer: server,
    })
  }
}

export const toggleMCPSystemServers = () => {
  const { showSystemServers } = useMCPServerManagementDrawerStore.getState()
  useMCPServerManagementDrawerStore.setState({
    showSystemServers: !showSystemServers,
  })
}

export const toggleMCPUserServers = () => {
  const { showUserServers } = useMCPServerManagementDrawerStore.getState()
  useMCPServerManagementDrawerStore.setState({
    showUserServers: !showUserServers,
  })
}
