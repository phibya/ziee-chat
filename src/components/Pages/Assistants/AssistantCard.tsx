import { DeleteOutlined, EditOutlined, MoreOutlined } from '@ant-design/icons'
import { App, Button, Card, Dropdown, Flex, Tag, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { deleteUserAssistant, openAssistantDrawer } from '../../../store'
import { Assistant } from '../../../types/api/assistant'

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
      hoverable
      className="cursor-pointer h-full flex flex-col"
      onClick={handleCardClick}
      classNames={{
        body: 'h-full flex flex-col',
      }}
    >
      <Flex className="h-full flex-col justify-between">
        {/* Content Area - Grows to fill space */}
        <div className="flex-1">
          <Card.Meta
            title={
              <div className="flex flex-col gap-1 pr-4">
                <div className={'text-ellipsis overflow-hidden'}>
                  <Typography.Title level={4} className={'whitespace-nowrap'}>
                    {assistant.name}
                  </Typography.Title>
                </div>
                <Flex className="gap-2">
                  {assistant.is_default && (
                    <Tag color="blue">{t('assistants.default')}</Tag>
                  )}
                  {!assistant.is_active && (
                    <Tag color="red">{t('assistants.inactive')}</Tag>
                  )}
                </Flex>
              </div>
            }
            description={
              <div>
                <Text type="secondary" className="block mb-2">
                  {assistant.description || 'No description'}
                </Text>
              </div>
            }
          />
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
              icon={<MoreOutlined />}
              onClick={e => e.stopPropagation()}
              size="small"
            />
          </Dropdown>
        </div>
      </Flex>
    </Card>
  )
}
