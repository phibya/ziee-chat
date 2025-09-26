import { useState, useEffect } from 'react'
import { Tabs, Card, Typography, message } from 'antd'
import { SystemServersTab } from './SystemServersTab.tsx'
import { ExecutionLogsTab } from './SimpleExecutionLogsTab.tsx'
import { GlobalApprovalsTab } from './SimpleGlobalApprovalsTab.tsx'
import { GroupAssignmentsTab } from './SimpleGroupAssignmentsTab.tsx'
import { StatisticsTab } from './StatisticsTab.tsx'
import { Stores } from '../../../../../store'
import {
  FaServer,
  FaHistory,
  FaShieldAlt,
  FaUsers,
  FaChartBar,
} from 'react-icons/fa'

const { Title } = Typography

export function MCPAdminPage() {
  const [activeTab, setActiveTab] = useState('system-servers')
  const { systemServersInitialized } = Stores.AdminMCPServers
  const { isInitialized: approvalsInitialized } = Stores.MCPApprovals
  const { executionLogsInitialized } = Stores.MCPExecution

  useEffect(() => {
    // Initialize stores when component mounts
    const initializeStores = async () => {
      try {
        // Initialize admin MCP servers if not already done
        if (!systemServersInitialized) {
          const { loadSystemServers } = await import(
            '../../../../../store/admin/mcpServers.ts'
          )
          await loadSystemServers()
        }

        // Initialize approvals if not already done
        if (!approvalsInitialized) {
          const { loadAllGlobalApprovals } = await import(
            '../../../../../store/mcpApprovals.ts'
          )
          await loadAllGlobalApprovals()
        }

        // Initialize execution logs if not already done
        if (!executionLogsInitialized) {
          const { loadExecutionLogs } = await import(
            '../../../../../store/mcpExecution.ts'
          )
          await loadExecutionLogs()
        }
      } catch (error) {
        console.error('Failed to initialize MCP admin stores:', error)
        message.error('Failed to load MCP administration data')
      }
    }

    initializeStores()
  }, [])

  const tabItems = [
    {
      key: 'system-servers',
      label: (
        <span className="flex items-center gap-2">
          <FaServer />
          System Servers
        </span>
      ),
      children: <SystemServersTab />,
    },
    {
      key: 'execution-logs',
      label: (
        <span className="flex items-center gap-2">
          <FaHistory />
          Execution Logs
        </span>
      ),
      children: <ExecutionLogsTab />,
    },
    {
      key: 'global-approvals',
      label: (
        <span className="flex items-center gap-2">
          <FaShieldAlt />
          Global Approvals
        </span>
      ),
      children: <GlobalApprovalsTab />,
    },
    {
      key: 'group-assignments',
      label: (
        <span className="flex items-center gap-2">
          <FaUsers />
          Group Assignments
        </span>
      ),
      children: <GroupAssignmentsTab />,
    },
    {
      key: 'statistics',
      label: (
        <span className="flex items-center gap-2">
          <FaChartBar />
          Statistics
        </span>
      ),
      children: <StatisticsTab />,
    },
  ]

  return (
    <div className="p-6 h-full overflow-auto">
      <Title level={2} className="mb-6">
        MCP Administration Dashboard
      </Title>

      <Card className="h-full">
        <Tabs
          activeKey={activeTab}
          onChange={setActiveTab}
          items={tabItems}
          className="h-full"
          tabBarStyle={{ marginBottom: '24px' }}
        />
      </Card>
    </div>
  )
}
