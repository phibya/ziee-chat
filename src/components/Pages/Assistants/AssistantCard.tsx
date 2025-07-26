import { DeleteOutlined, EditOutlined } from '@ant-design/icons'
import { App, Button, Card, Flex, Popconfirm, Tag, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { deleteUserAssistant, openAssistantDrawer } from '../../../store'
import { Assistant } from '../../../types/api/assistant'

const { Text } = Typography

interface AssistantCardProps {
  assistant: Assistant
}

export function AssistantCard({ assistant }: AssistantCardProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()

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
              <div className="flex flex-col gap-1">
                <Text strong>{assistant.name}</Text>
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

        {/* Actions Area - Always at bottom */}
        <Flex justify="flex-end" className="gap-2">
          <Button
            size="small"
            icon={<EditOutlined />}
            onClick={e => {
              e.stopPropagation()
              handleEdit()
            }}
          >
            {t('buttons.edit')}
          </Button>

          <Popconfirm
            title={t('assistants.deleteAssistant')}
            description={t('assistants.deleteConfirm')}
            onConfirm={() => handleDelete()}
            okText="Yes"
            cancelText="No"
          >
            <Button
              danger
              size="small"
              icon={<DeleteOutlined />}
              onClick={e => e.stopPropagation()}
            >
              {t('buttons.delete')}
            </Button>
          </Popconfirm>
        </Flex>
      </Flex>
    </Card>
  )
}
