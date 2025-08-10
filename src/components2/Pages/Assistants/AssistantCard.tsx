import {
  CalendarOutlined,
  DeleteOutlined,
  EditOutlined,
} from '@ant-design/icons'
import { App, Button, Card, Dropdown, Flex, Tag, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { deleteUserAssistant, openAssistantDrawer } from '../../../store'
import { Assistant } from '../../../types/api/assistant'
import { CgMenuRightAlt } from 'react-icons/cg'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

const { Text } = Typography

interface AssistantCardProps {
  assistant: Assistant
}

export function AssistantCard({ assistant }: AssistantCardProps) {
  const { t } = useTranslation()
  const { message, modal } = App.useApp()

  const handleDelete = async () => {
    try {
      await deleteUserAssistant(assistant.id)
      message.success(t('assistants.assistantDeleted'))
    } catch (error) {
      console.error('Failed to delete assistant:', error)
    }
  }

  const handleEdit = () => {
    openAssistantDrawer(assistant)
  }

  const handleCardClick = () => {
    openAssistantDrawer(assistant)
  }

  return (
    <Card
      className="cursor-pointer relative group hover:!shadow-md transition-shadow h-full"
      classNames={{
        body: '!px-3 !pb-0 !py-2 flex gap-2 flex-col',
      }}
      hoverable
      onClick={handleCardClick}
    >
      <Flex className="h-full flex-col flex-1">
        {/* Header with name and tags */}
        <Typography.Text strong className="m-0 pr-2">
          {assistant.name}
        </Typography.Text>

        {/* Tags */}
        {(assistant.is_default || !assistant.is_active) && (
          <div className="mb-2">
            <Flex className="gap-1">
              {assistant.is_default && (
                <Tag color="blue" className="text-xs">
                  {t('assistants.default')}
                </Tag>
              )}
              {!assistant.is_active && (
                <Tag color="red" className="text-xs">
                  {t('assistants.inactive')}
                </Tag>
              )}
            </Flex>
          </div>
        )}

        {/* Description */}
        {assistant.description && (
          <div className="mb-3">
            <Text type="secondary" className="text-sm line-clamp-2">
              {assistant.description}
            </Text>
          </div>
        )}

        {/* Stats and date - pushed to bottom */}
        <div
          style={{
            marginTop: assistant.description ? 'auto' : '12px',
          }}
        >
          {/* Last updated */}
          <div className="mb-2">
            <Flex align="center" gap="small">
              <CalendarOutlined className="text-gray-400" />
              <Text type="secondary" className="text-xs">
                Updated {dayjs(assistant.updated_at).fromNow()}
              </Text>
            </Flex>
          </div>
        </div>

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
                    handleEdit()
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
                      title: 'Delete Assistant',
                      content: `Are you sure?`,
                      okText: 'Delete',
                      okType: 'danger',
                      onOk: handleDelete,
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
      </Flex>
    </Card>
  )
}
