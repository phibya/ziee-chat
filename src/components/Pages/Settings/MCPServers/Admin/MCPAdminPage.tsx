import { useEffect } from 'react'
import { message } from 'antd'
import { SystemServersTab } from './SystemServersTab.tsx'
import { Stores } from '../../../../../store'
import { SettingsPageContainer } from '../../common/SettingsPageContainer'

export function MCPAdminPage() {
  const { systemServersInitialized } = Stores.AdminMCPServers

  useEffect(() => {
    // Initialize admin MCP servers store when component mounts
    const initializeStores = async () => {
      try {
        if (!systemServersInitialized) {
          const { loadSystemServers } = await import(
            '../../../../../store/admin/mcpServers.ts'
          )
          await loadSystemServers()
        }
      } catch (error) {
        console.error('Failed to initialize MCP admin servers:', error)
        message.error('Failed to load system MCP servers')
      }
    }

    initializeStores()
  }, [systemServersInitialized])

  return (
    <SettingsPageContainer
      title="System MCP Servers"
      subtitle="Manage Model Context Protocol servers across the system"
    >
      <SystemServersTab />
    </SettingsPageContainer>
  )
}
