import {
  CalendarOutlined,
  DeleteOutlined,
  EditOutlined,
} from '@ant-design/icons'
import { App, Button, Card, Dropdown, Flex, Typography } from 'antd'
import { useNavigate } from 'react-router-dom'
import type { RAGInstance } from '../../../types'
import { deleteRAGInstance, openRAGInstanceDrawer } from '../../../store'
import { CgMenuRightAlt } from 'react-icons/cg'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { Permission } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'

dayjs.extend(relativeTime)

const { Text } = Typography

interface RagCardProps {
  ragInstance: RAGInstance
}

export function RagCard({ ragInstance }: RagCardProps) {
  const navigate = useNavigate()
  const { message, modal } = App.useApp()
  const isSystemInstance = ragInstance.is_system

  const handleCardClick = () => {
    navigate(`/rags/${ragInstance.id}`)
  }

  const handleDeleteInstance = async (instance: RAGInstance) => {
    try {
      await deleteRAGInstance(instance.id)
      message.success('RAG instance deleted successfully')
    } catch (error) {
      console.error('Failed to delete RAG instance:', error)
    }
  }


  return (
    <Card
      className={`cursor-pointer relative group hover:!shadow-md transition-shadow h-full ${
        isSystemInstance ? 'border-blue-200 bg-blue-50/20' : ''
      }`}
      classNames={{
        body: '!px-3 !pb-0 !py-2 flex gap-2 flex-col',
      }}
      hoverable
      onClick={handleCardClick}
    >
      <Flex className="h-full flex-col flex-1">
        {/* Header with name */}
        <div className="mb-2">
          <Typography.Text strong className="m-0">
            {ragInstance.name}
          </Typography.Text>
        </div>

        {/* Description */}
        {ragInstance.description && (
          <div className="mb-3">
            <Text type="secondary" className="text-sm line-clamp-2">
              {ragInstance.description}
            </Text>
          </div>
        )}

        {/* Stats and date - pushed to bottom */}
        <div
          style={{
            marginTop: ragInstance.description ? 'auto' : '12px',
          }}
        >
          {/* Last updated */}
          <div className="mb-2">
            <Flex align="center" gap="small">
              <CalendarOutlined className="text-gray-400" />
              <Text type="secondary" className="text-xs">
                Updated {dayjs(ragInstance.updated_at).fromNow()}
              </Text>
            </Flex>
          </div>
        </div>

        {/* Show actions based on permissions - both personal instances and system instances with admin permission */}
        <PermissionGuard 
          permissions={isSystemInstance ? [Permission.RagAdminInstancesEdit] : [Permission.RagInstancesEdit]}
          type="hidden"
        >
          <div className="absolute top-2 right-2">
            <Dropdown
              menu={{
                items: [
                  {
                    key: 'edit',
                    icon: <EditOutlined />,
                    label: 'Edit',
                    onClick: e => {
                      e.domEvent.stopPropagation()
                      e.domEvent.preventDefault()
                      openRAGInstanceDrawer(ragInstance)
                    },
                  },
                  {
                    key: 'delete',
                    icon: <DeleteOutlined />,
                    label: 'Delete',
                    danger: true,
                    onClick: e => {
                      e.domEvent.stopPropagation()
                      e.domEvent.preventDefault()
                      modal.confirm({
                        title: 'Delete RAG Instance',
                        content: `Are you sure you want to delete "${ragInstance.name}"?`,
                        okText: 'Delete',
                        okType: 'danger',
                        onOk: () => handleDeleteInstance(ragInstance),
                      })
                    },
                  },
                ],
              }}
              trigger={['click']}
            >
              <Button
                type="text"
                icon={<CgMenuRightAlt />}
                onClick={e => e.stopPropagation()}
                size="small"
              />
            </Dropdown>
          </div>
        </PermissionGuard>
      </Flex>
    </Card>
  )
}