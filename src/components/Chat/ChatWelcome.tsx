import { memo } from 'react'
import { Button, Col, Input, Row, Select, Space, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { RobotOutlined, SendOutlined } from '@ant-design/icons'
import { Assistant } from '../../types/api/assistant'
import { ModelProvider } from '../../types/api/modelProvider'

const { TextArea } = Input
const { Text } = Typography
const { Option } = Select

interface ChatWelcomeProps {
  inputValue: string
  selectedAssistant: string | null
  selectedModel: string | null
  assistants: Assistant[]
  modelProviders: ModelProvider[]
  onInputChange: (value: string) => void
  onAssistantChange: (assistantId: string) => void
  onModelChange: (modelId: string) => void
  onSend: () => void
  onKeyPress: (e: React.KeyboardEvent) => void
}

export const ChatWelcome = memo(function ChatWelcome({
  inputValue,
  selectedAssistant,
  selectedModel,
  assistants,
  modelProviders,
  onInputChange,
  onAssistantChange,
  onModelChange,
  onSend,
  onKeyPress,
}: ChatWelcomeProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col h-full">
      {/* Header with model selection */}
      <div className="px-4 sm:px-6 py-4">
        <Row gutter={16} align="middle">
          <Col xs={24} sm={12} md={8}>
            <Select
              value={selectedAssistant}
              onChange={onAssistantChange}
              placeholder="Select your assistant"
              className="w-full"
              showSearch
              optionFilterProp="children"
            >
              {assistants.map(assistant => (
                <Option key={assistant.id} value={assistant.id}>
                  <Space>
                    <RobotOutlined />
                    {assistant.name}
                  </Space>
                </Option>
              ))}
            </Select>
          </Col>
          <Col xs={24} sm={12} md={8}>
            <Select
              value={selectedModel}
              onChange={onModelChange}
              placeholder="Select a model"
              className="w-full"
              showSearch
              optionFilterProp="children"
            >
              {modelProviders.map(provider => (
                <Select.OptGroup key={provider.id} label={provider.name}>
                  {provider.models.map(model => (
                    <Option
                      key={`${provider.id}:${model.id}`}
                      value={`${provider.id}:${model.id}`}
                    >
                      {model.alias}
                    </Option>
                  ))}
                </Select.OptGroup>
              ))}
            </Select>
          </Col>
        </Row>
      </div>

      {/* Welcome message */}
      <div className="flex flex-col items-center justify-center flex-1 text-center p-8">
        <div className="mb-8">
          <div className="text-3xl font-light mb-4">
            {t('chat.placeholderWelcome')}
          </div>
        </div>

        <div className="w-full max-w-2xl">
          <div className="flex items-end gap-3">
            <div className="flex-1">
              <TextArea
                value={inputValue}
                onChange={e => onInputChange(e.target.value)}
                onKeyPress={onKeyPress}
                placeholder={t('chat.placeholder')}
                autoSize={{ minRows: 1, maxRows: 6 }}
                disabled={!selectedAssistant || !selectedModel}
                className="resize-none"
              />
            </div>
            <Button
              type="primary"
              icon={<SendOutlined />}
              onClick={onSend}
              disabled={
                !inputValue.trim() || !selectedAssistant || !selectedModel
              }
              className="h-10 rounded-lg"
            >
              {t('chat.send')}
            </Button>
          </div>

          {(!selectedAssistant || !selectedModel) && (
            <div className="mt-4">
              <Text type="secondary" className="text-sm">
                {t('chat.noAssistantSelected')}
              </Text>
            </div>
          )}
        </div>
      </div>
    </div>
  )
})