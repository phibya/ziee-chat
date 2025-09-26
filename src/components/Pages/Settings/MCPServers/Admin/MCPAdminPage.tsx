import { useEffect } from 'react'
import {
  Button,
  Dropdown,
  Empty,
  Flex,
  Segmented,
  message,
  theme,
  Typography,
} from 'antd'
import { useNavigate, useParams } from 'react-router-dom'
import { IoIosArrowDown } from 'react-icons/io'
import { SystemServersTab } from './SystemServersTab.tsx'
import { ExecutionLogsTab } from './SimpleExecutionLogsTab.tsx'
import { GlobalApprovalsTab } from './SimpleGlobalApprovalsTab.tsx'
import { GroupAssignmentsTab } from './SimpleGroupAssignmentsTab.tsx'
import { StatisticsTab } from './StatisticsTab.tsx'
import { Stores } from '../../../../../store'
import { useMainContentMinSize } from '../../../../hooks/useWindowMinSize'
import {
  FaServer,
  FaHistory,
  FaShieldAlt,
  FaUsers,
  FaChartBar,
} from 'react-icons/fa'
import { SettingsPageContainer } from '../../common/SettingsPageContainer'

export function MCPAdminPage() {
  const navigate = useNavigate()
  const { sectionId } = useParams<{ sectionId?: string }>()
  const mainContentMinSize = useMainContentMinSize()
  const { token } = theme.useToken()
  const { systemServersInitialized } = Stores.AdminMCPServers
  const { isInitialized: approvalsInitialized } = Stores.MCPApprovals
  const { executionLogsInitialized } = Stores.MCPExecution

  // Available sections
  const sections = [
    {
      key: 'system-servers',
      label: 'System Servers',
      shortLabel: 'Servers',
      icon: FaServer,
    },
    {
      key: 'execution-logs',
      label: 'Execution Logs',
      shortLabel: 'Logs',
      icon: FaHistory,
    },
    {
      key: 'global-approvals',
      label: 'Global Approvals',
      shortLabel: 'Approvals',
      icon: FaShieldAlt,
    },
    {
      key: 'group-assignments',
      label: 'Group Assignments',
      shortLabel: 'Groups',
      icon: FaUsers,
    },
    {
      key: 'statistics',
      label: 'Statistics',
      shortLabel: 'Stats',
      icon: FaChartBar,
    },
  ]

  const validSections = sections.map(s => s.key)
  const activeSection =
    sectionId && validSections.includes(sectionId) ? sectionId : sections[0].key
  const currentSection =
    sections.find(s => s.key === activeSection) || sections[0]

  useEffect(() => {
    // Handle URL parameter and section selection
    if (sectionId) {
      // If URL has sectionId, check if it's valid
      const sectionExists = sections.find(s => s.key === sectionId)
      if (!sectionExists) {
        // Section doesn't exist, redirect to first section
        navigate('/settings/mcp-admin/system-servers', { replace: true })
      }
    } else {
      // No URL parameter, navigate to first section
      navigate('/settings/mcp-admin/system-servers', { replace: true })
    }
  }, [sectionId, navigate])

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

  const handleSectionChange = (value: string | number) => {
    navigate(`/settings/mcp-admin/${value}`)
  }

  const getCurrentSectionLabel = () => {
    const IconComponent = currentSection.icon
    return (
      <Flex align="center" gap={4}>
        <IconComponent />
        {currentSection.shortLabel}
      </Flex>
    )
  }

  const renderSectionContent = () => {
    switch (activeSection) {
      case 'system-servers':
        return <SystemServersTab />
      case 'execution-logs':
        return <ExecutionLogsTab />
      case 'global-approvals':
        return <GlobalApprovalsTab />
      case 'group-assignments':
        return <GroupAssignmentsTab />
      case 'statistics':
        return <StatisticsTab />
      default:
        return (
          <Empty
            description="Section not found"
            image={Empty.PRESENTED_IMAGE_SIMPLE}
          />
        )
    }
  }

  // Create title with navigation for mobile
  const titleWithNavigation = (
    <Flex align="center" justify="space-between" className="w-full">
      <span>MCP Administration</span>
      {/* Mobile: Show dropdown */}
      {mainContentMinSize.xs && (
        <div className="flex flex-1 items-center justify-end px-2">
          <Dropdown
            menu={{
              items: sections.map(section => {
                const IconComponent = section.icon
                return {
                  key: section.key,
                  label: (
                    <Flex className={'gap-2'}>
                      <IconComponent />
                      {section.label}
                    </Flex>
                  ),
                }
              }),
              onClick: ({ key }) => {
                navigate(`/settings/mcp-admin/${key}`)
              },
              selectedKeys: [activeSection],
            }}
            trigger={['click']}
          >
            <Button type="text" size="small">
              {getCurrentSectionLabel()} <IoIosArrowDown />
            </Button>
          </Dropdown>
        </div>
      )}
    </Flex>
  )

  return (
    <SettingsPageContainer
      title={titleWithNavigation}
      subtitle="Manage and monitor Model Context Protocol servers across the system"
    >
      {/* Desktop: Show segmented control */}
      {!mainContentMinSize.xs && (
        <div className="flex justify-center items-center">
          <Segmented
            value={activeSection}
            onChange={handleSectionChange}
            className={`
            [&_.ant-segmented-item-label]:!px-3
            [&_.ant-segmented-item-label]:!py-1
            `}
            style={{
              backgroundColor: token.colorBgMask,
            }}
            shape="round"
            options={sections.map(section => {
              const IconComponent = section.icon
              return {
                value: section.key,
                label: (
                  <Flex align="center" className={'gap-1'}>
                    <IconComponent />
                    <Typography.Text>{section.shortLabel}</Typography.Text>
                  </Flex>
                ),
              }
            })}
          />
        </div>
      )}

      {/* Content */}
      <div className="flex flex-col gap-3 h-full overflow-hidden">
        {renderSectionContent()}
      </div>
    </SettingsPageContainer>
  )
}
