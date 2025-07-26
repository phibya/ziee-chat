import {
  CopyOutlined,
  DownOutlined,
  RobotOutlined,
  UpOutlined,
} from '@ant-design/icons'
import { Button, Card, Flex, Typography } from 'antd'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Drawer } from '../../common/Drawer.tsx'
import { Assistant } from '../../../types/api/assistant.ts'
import { openAssistantDrawer } from '../../../store'

const { Text } = Typography

interface TemplateAssistantCardProps {
  assistant: Assistant
}

function TemplateAssistantCard({ assistant }: TemplateAssistantCardProps) {
  const { t } = useTranslation()
  const [expanded, setExpanded] = useState(false)

  const onSelectTemplate = () => {
    // This function should handle the logic to clone the assistant template
    // For now, we just log the assistant ID
    console.log('Cloning assistant template:', assistant.id)
    openAssistantDrawer(assistant, false, true)
  }

  return (
    <Card hoverable className="w-full mb-3 h-full flex flex-col">
      <Flex className="h-full flex-col justify-between">
        {/* Content Area - Grows to fill space */}
        <div className="flex-1">
          <Flex className="flex-col gap-1 w-full mb-3">
            <Flex align="center" className="gap-2">
              <RobotOutlined />
              <Text strong>{assistant.name}</Text>
            </Flex>
            <Text type="secondary" className="text-sm">
              {assistant.description || 'No description'}
            </Text>
            {assistant.instructions && !expanded && (
              <Text
                type="secondary"
                className="text-xs"
                ellipsis={{ tooltip: assistant.instructions }}
              >
                {assistant.instructions.substring(0, 150)}
                {assistant.instructions.length > 150 ? '...' : ''}
              </Text>
            )}
          </Flex>

          {/* Expanded Details */}
          {expanded && (
            <Flex vertical className="gap-3 p-3 rounded">
              <div>
                <Text strong className="block mb-1">
                  {t('labels.instructions')}
                </Text>
                <Card size="small">
                  <div style={{ whiteSpace: 'pre-wrap' }}>
                    {assistant.instructions || 'No instructions'}
                  </div>
                </Card>
              </div>

              <div>
                <Text strong className="block mb-1">
                  {t('labels.parameters')}
                </Text>
                <Card size="small">
                  <pre style={{ overflow: 'auto', margin: 0 }}>
                    {assistant.parameters
                      ? JSON.stringify(assistant.parameters, null, 2)
                      : 'No parameters'}
                  </pre>
                </Card>
              </div>
            </Flex>
          )}
        </div>

        {/* Actions Area - Always at bottom */}
        <div className="mt-4 pt-3 border-t border-gray-100">
          <Flex align="center" className="gap-2 justify-end">
            <Button
              type="default"
              size="small"
              icon={expanded ? <UpOutlined /> : <DownOutlined />}
              onClick={() => setExpanded(!expanded)}
            >
              {expanded ? 'Hide Details' : 'Show Details'}
            </Button>
            <Button
              type="primary"
              size="small"
              icon={<CopyOutlined />}
              onClick={onSelectTemplate}
            >
              Clone
            </Button>
          </Flex>
        </div>
      </Flex>
    </Card>
  )
}

interface TemplateAssistantDrawerProps {
  open: boolean
  onClose: () => void
  templateAssistants: Assistant[]
}

export function TemplateAssistantDrawer({
  open,
  onClose,
  templateAssistants,
}: TemplateAssistantDrawerProps) {
  const { t } = useTranslation()

  return (
    <Drawer
      title={t('assistants.cloneFromTemplateAssistants')}
      open={open}
      onClose={onClose}
      footer={[
        <Button key="close" onClick={onClose}>
          {t('buttons.close')}
        </Button>,
      ]}
      width={500}
      maskClosable={true}
    >
      <div className="mb-4">
        <Text type="secondary">
          Select a template assistant to clone and customize for your use
        </Text>
      </div>
      <Flex className="flex-col gap-2">
        {templateAssistants.map(assistant => (
          <TemplateAssistantCard key={assistant.id} assistant={assistant} />
        ))}
      </Flex>
    </Drawer>
  )
}
