import { Card, Tag, Typography, Button, Flex } from 'antd'
import {
  DownloadOutlined,
  StarOutlined,
  GlobalOutlined,
  GithubOutlined,
} from '@ant-design/icons'
import type { HubMCPServer } from '../../../types'
import { useState } from 'react'
import { MCPServerDetailsDrawer } from './MCPServerDetailsDrawer'

const { Text } = Typography

interface MCPServerCardProps {
  server: HubMCPServer
}

export function MCPServerCard({ server }: MCPServerCardProps) {
  const [selectedServer, setSelectedServer] = useState<HubMCPServer | null>(null)

  return (
    <>
      <Card
        hoverable
        className="cursor-pointer relative group hover:!shadow-md transition-shadow h-full"
        onClick={() => setSelectedServer(server)}
      >
        <div className="flex items-start gap-3 flex-wrap">
          {/* Server Info */}
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-2 flex-wrap">
              <div className="flex-1 min-w-48">
                <Flex className="gap-2 items-center">
                  {server.icon_url && (
                    <img
                      src={server.icon_url}
                      alt={server.display_name}
                      className="w-6 h-6 rounded"
                    />
                  )}
                  <Text className="font-medium cursor-pointer">
                    {server.display_name}
                  </Text>
                  <Tag color="blue" className="text-xs">
                    {server.category}
                  </Tag>
                  <Tag className="text-xs">{server.transport_type.toUpperCase()}</Tag>
                </Flex>
              </div>
              <div className="flex gap-1 items-center justify-end">
                {server.homepage && (
                  <Button
                    icon={<GlobalOutlined />}
                    onClick={e => {
                      e.stopPropagation()
                      window.open(server.homepage, '_blank')
                    }}
                  />
                )}
                {server.repository_url && (
                  <Button
                    icon={<GithubOutlined />}
                    onClick={e => {
                      e.stopPropagation()
                      window.open(server.repository_url, '_blank')
                    }}
                  />
                )}
                <Button
                  type="primary"
                  icon={<DownloadOutlined />}
                  onClick={e => {
                    e.stopPropagation()
                    setSelectedServer(server)
                  }}
                >
                  Install
                </Button>
              </div>
            </div>

            <div>
              {server.description && (
                <Text type="secondary" className="text-sm mb-2 block">
                  {server.description}
                </Text>
              )}

              {/* Tags */}
              {server.tags.length > 0 && (
                <div className="mb-2">
                  <Text type="secondary" className="text-xs mr-2">
                    Tags:
                  </Text>
                  <Flex wrap className="gap-1" style={{ display: 'inline-flex' }}>
                    {server.tags.slice(0, 3).map((tag: string) => (
                      <Tag key={tag} color="default" className="text-xs">
                        {tag}
                      </Tag>
                    ))}
                    {server.tags.length > 3 && (
                      <Tag color="default" className="text-xs">
                        +{server.tags.length - 3}
                      </Tag>
                    )}
                  </Flex>
                </div>
              )}

              {/* Metadata */}
              <div className="mb-2">
                <Flex wrap className="gap-4 text-xs">
                  {server.author && (
                    <span>
                      <Text type="secondary" className="text-xs">
                        Author:
                      </Text>{' '}
                      {server.author}
                    </span>
                  )}
                  {server.tool_count && (
                    <span>
                      <Text type="secondary" className="text-xs">
                        Tools:
                      </Text>{' '}
                      {server.tool_count}
                    </span>
                  )}
                  {server.download_count && (
                    <span>
                      <Text type="secondary" className="text-xs">
                        Downloads:
                      </Text>{' '}
                      {server.download_count.toLocaleString()}
                    </span>
                  )}
                  {server.rating && (
                    <span>
                      <Text type="secondary" className="text-xs">
                        Rating:
                      </Text>{' '}
                      <StarOutlined /> {server.rating.toFixed(1)}
                    </span>
                  )}
                </Flex>
              </div>
            </div>
          </div>
        </div>
      </Card>

      <MCPServerDetailsDrawer
        server={selectedServer}
        open={!!selectedServer}
        onClose={() => setSelectedServer(null)}
      />
    </>
  )
}
