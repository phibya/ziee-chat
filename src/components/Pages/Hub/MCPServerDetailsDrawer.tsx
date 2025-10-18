import {
  Typography,
  Tag,
  Flex,
  Button,
  App,
  Card,
} from 'antd'
import {
  DownloadOutlined,
  StarOutlined,
  CodeOutlined,
  GlobalOutlined,
  GithubOutlined,
  FileTextOutlined,
} from '@ant-design/icons'
import type { HubMCPServer, CreateMCPServerRequest } from '../../../types'
import { isTauriView } from '../../../api/core'
import { createMCPServer } from '../../../store/mcp'
import { Drawer } from '../../common/Drawer'

const { Title, Text } = Typography

interface MCPServerDetailsDrawerProps {
  server: HubMCPServer | null
  open: boolean
  onClose: () => void
}

export function MCPServerDetailsDrawer({
  server,
  open,
  onClose,
}: MCPServerDetailsDrawerProps) {
  const { message } = App.useApp()
  const isDesktop = isTauriView

  if (!server) return null

  const handleInstall = async () => {
    try {
      const request: CreateMCPServerRequest = {
        name: server.name,
        display_name: server.display_name,
        description: server.description || undefined,
        transport_type: server.transport_type as any,
        command: server.command || undefined,
        args: server.args || undefined,
        environment_variables: server.environment_variables || undefined,
        url: server.url || undefined,
        headers: server.headers || undefined,
        enabled: true,
      }

      await createMCPServer(request)
      message.success(`${server.display_name} installed successfully!`)
      onClose()
    } catch (error) {
      console.error('Failed to install MCP server:', error)
      message.error('Failed to install MCP server')
    }
  }

  const canInstall = isDesktop || !server.requires_desktop

  return (
    <Drawer
      title={server.display_name}
      open={open}
      onClose={onClose}
      width={600}
      footer={[
        <Button
          key="install"
          type="primary"
          icon={<DownloadOutlined />}
          onClick={handleInstall}
          disabled={!canInstall}
        >
          Install Server
        </Button>,
      ]}
    >
      <Flex vertical className="gap-4">
        {/* Basic Info */}
        <div>
          <Flex justify="space-between" align="center" className="mb-2">
            <Title level={3} className="m-0">
              {server.display_name}
            </Title>
            {server.author && (
              <Text type="secondary" className="text-sm">
                by {server.author}
              </Text>
            )}
          </Flex>
          {server.description && (
            <Text type="secondary">{server.description}</Text>
          )}
        </div>

        {/* Stats */}
        <div>
          <Title level={5}>Statistics</Title>
          <Flex vertical className="gap-2">
            {server.rating && (
              <Flex justify="space-between">
                <Text type="secondary">Rating:</Text>
                <Flex align="center" gap={4}>
                  <StarOutlined />
                  <Text>{server.rating.toFixed(1)}</Text>
                </Flex>
              </Flex>
            )}
            {server.download_count && (
              <Flex justify="space-between">
                <Text type="secondary">Downloads:</Text>
                <Text>{server.download_count.toLocaleString()}</Text>
              </Flex>
            )}
            {server.popularity_score && (
              <Flex justify="space-between">
                <Text type="secondary">Popularity Score:</Text>
                <Text>{server.popularity_score}</Text>
              </Flex>
            )}
          </Flex>
        </div>

        {/* Server Details */}
        <div>
          <Title level={5}>Server Details</Title>
          <Flex vertical className="gap-2">
            <Flex justify="space-between">
              <Text type="secondary">Category:</Text>
              <Tag color="blue">{server.category}</Tag>
            </Flex>
            <Flex justify="space-between">
              <Text type="secondary">Transport Type:</Text>
              <Tag>{server.transport_type.toUpperCase()}</Tag>
            </Flex>
            {server.tool_count && (
              <Flex justify="space-between">
                <Text type="secondary">Tool Count:</Text>
                <Text>{server.tool_count}</Text>
              </Flex>
            )}
            {server.version && (
              <Flex justify="space-between">
                <Text type="secondary">Version:</Text>
                <Text>{server.version}</Text>
              </Flex>
            )}
            {server.license && (
              <Flex justify="space-between">
                <Text type="secondary">License:</Text>
                <Text>{server.license}</Text>
              </Flex>
            )}
          </Flex>
        </div>

        {/* Tags */}
        {server.tags.length > 0 && (
          <div>
            <Title level={5}>Tags</Title>
            <Flex wrap className="gap-1">
              {server.tags.map((tag: string) => (
                <Tag key={tag}>{tag}</Tag>
              ))}
            </Flex>
          </div>
        )}

        {/* Tool Categories */}
        {server.tool_categories && server.tool_categories.length > 0 && (
          <div>
            <Title level={5}>Tool Categories</Title>
            <Flex wrap className="gap-1">
              {server.tool_categories.map((cat: string) => (
                <Tag key={cat} color="cyan">
                  {cat}
                </Tag>
              ))}
            </Flex>
          </div>
        )}

        {/* Example Tools */}
        {server.example_tools && server.example_tools.length > 0 && (
          <div>
            <Title level={5}>Example Tools</Title>
            <Flex wrap className="gap-1">
              {server.example_tools.map((tool: string) => (
                <Tag key={tool} icon={<CodeOutlined />}>
                  {tool}
                </Tag>
              ))}
            </Flex>
          </div>
        )}

        {/* Use Cases */}
        {server.use_cases && server.use_cases.length > 0 && (
          <div>
            <Title level={5}>Use Cases</Title>
            <Card size="small">
              <ul className="m-0 pl-4">
                {server.use_cases.map((useCase: string) => (
                  <li key={useCase}>{useCase}</li>
                ))}
              </ul>
            </Card>
          </div>
        )}

        {/* Platform Support */}
        {server.platform_support && server.platform_support.length > 0 && (
          <div>
            <Title level={5}>Platform Support</Title>
            <Flex wrap className="gap-1">
              {server.platform_support.map((platform: string) => (
                <Tag key={platform} color="green">
                  {platform}
                </Tag>
              ))}
            </Flex>
          </div>
        )}

        {/* Links */}
        {(server.homepage || server.repository_url || server.documentation_url) && (
          <div>
            <Title level={5}>Links</Title>
            <Flex wrap className="gap-2">
              {server.homepage && (
                <Button
                  size="small"
                  icon={<GlobalOutlined />}
                  onClick={() => window.open(server.homepage, '_blank')}
                >
                  Homepage
                </Button>
              )}
              {server.repository_url && (
                <Button
                  size="small"
                  icon={<GithubOutlined />}
                  onClick={() => window.open(server.repository_url, '_blank')}
                >
                  Repository
                </Button>
              )}
              {server.documentation_url && (
                <Button
                  size="small"
                  icon={<FileTextOutlined />}
                  onClick={() => window.open(server.documentation_url, '_blank')}
                >
                  Documentation
                </Button>
              )}
            </Flex>
          </div>
        )}

        {/* Platform Warning */}
        {!canInstall && (
          <Card size="small" style={{ background: '#fff7e6', borderColor: '#ffd591' }}>
            <Text type="warning">
              This server requires desktop application and cannot be installed in web mode.
            </Text>
          </Card>
        )}
      </Flex>
    </Drawer>
  )
}
