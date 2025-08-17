import { InfoCircleOutlined, RobotOutlined } from '@ant-design/icons'
import { App, Button, Card, Flex, Tag, Typography } from 'antd'
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import type { HubAssistant } from '../../../types/api/hub'
import { createUserAssistant } from '../../../store/assistants'
import { AssistantDetailsDrawer } from './AssistantDetailsDrawer'

const { Text } = Typography

interface AssistantCardProps {
  assistant: HubAssistant
}

export function AssistantCard({ assistant }: AssistantCardProps) {
  const { message } = App.useApp()
  const [showDetails, setShowDetails] = useState(false)
  const [isCreating, setIsCreating] = useState(false)
  const navigate = useNavigate()

  const handleUseAssistant = async (assistant: HubAssistant) => {
    setIsCreating(true)
    try {
      // Create a user assistant based on the hub assistant
      await createUserAssistant({
        name: assistant.name,
        description: assistant.description,
        instructions: assistant.instructions,
        parameters: assistant.parameters || { stream: true },
        is_active: true,
      })

      message.success(`Assistant "${assistant.name}" created successfully!`)

      // Navigate to assistants page to show the newly created assistant
      navigate('/assistants')
    } catch (error: any) {
      console.error('Failed to create assistant:', error)
      message.error(
        `Failed to create assistant: ${error.message || 'Unknown error'}`,
      )
    } finally {
      setIsCreating(false)
    }
  }

  const handleShowDetails = () => {
    setShowDetails(true)
  }

  const handleCloseDetails = () => {
    setShowDetails(false)
  }

  return (
    <>
      <Card
        hoverable
        className="cursor-pointer relative group hover:!shadow-md transition-shadow h-full"
        onClick={handleShowDetails}
      >
        <div className="flex items-start gap-3 flex-wrap">
          {/* Assistant Info */}
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-2 flex-wrap">
              <div className="flex-1 min-w-48">
                <Flex className="gap-2 items-center">
                  <RobotOutlined />
                  <Text
                    className="font-medium cursor-pointer"
                    onClick={handleShowDetails}
                  >
                    {assistant.name}
                  </Text>
                  <Tag color="geekblue" className="text-xs">
                    {assistant.category}
                  </Tag>
                  {isCreating && <Tag color="blue">Creating...</Tag>}
                </Flex>
              </div>
              <div className="flex gap-1 items-center justify-end">
                <Button
                  icon={<InfoCircleOutlined />}
                  onClick={e => {
                    e.stopPropagation()
                    handleShowDetails()
                  }}
                >
                  Details
                </Button>
                <Button
                  type="primary"
                  icon={<RobotOutlined />}
                  onClick={e => {
                    e.stopPropagation()
                    handleUseAssistant(assistant)
                  }}
                  loading={isCreating}
                  disabled={isCreating}
                >
                  Use Assistant
                </Button>
              </div>
            </div>

            <div>
              <Text type="secondary" className="text-sm mb-2 block">
                {assistant.description}
              </Text>

              {/* Tags */}
              {assistant.tags.length > 0 && (
                <div className="mb-2">
                  <Text type="secondary" className="text-xs mr-2">
                    Tags:
                  </Text>
                  <Flex
                    wrap
                    className="gap-1"
                    style={{ display: 'inline-flex' }}
                  >
                    {assistant.tags.map(tag => (
                      <Tag key={tag} color="default" className="text-xs">
                        {tag}
                      </Tag>
                    ))}
                  </Flex>
                </div>
              )}

              {/* Metadata */}
              <div className="mb-2">
                <Flex wrap className="gap-4 text-xs">
                  {assistant.author && (
                    <span>
                      <Text type="secondary" className="text-xs">
                        Author:
                      </Text>{' '}
                      {assistant.author}
                    </span>
                  )}
                  {assistant.recommended_models.length > 0 && (
                    <span>
                      <Text type="secondary" className="text-xs">
                        Models:
                      </Text>{' '}
                      {assistant.recommended_models.slice(0, 2).join(', ')}
                      {assistant.recommended_models.length > 2 && '...'}
                    </span>
                  )}
                </Flex>
              </div>
            </div>
          </div>
        </div>
      </Card>

      <AssistantDetailsDrawer
        assistant={assistant}
        open={showDetails}
        onClose={handleCloseDetails}
      />
    </>
  )
}
