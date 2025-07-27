import { ArrowUpOutlined } from '@ant-design/icons'
import { App, Button, Flex, Input, Select, Typography } from 'antd'
import React, { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import {
  clearProjectsStoreError,
  loadProjectWithDetails,
  Stores,
} from '../../../store'
import { PageContainer } from '../../common/PageContainer.tsx'
import { ProjectFormDrawer } from './ProjectFormDrawer.tsx'
import { ProjectKnowledgeCard } from './ProjectKnowledgeCard.tsx'
import { RecentConversationsSection } from './RecentConversationsSection.tsx'

const { Title } = Typography
const { TextArea } = Input

// Mock data
const mockAssistants = [
  { label: 'Claude Sonnet 4', value: 'Claude Sonnet 4' },
  { label: 'GPT-4', value: 'GPT-4' },
  { label: 'Gemini Pro', value: 'Gemini Pro' },
]

export const ProjectDetailsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()

  // Projects store
  const { currentProject, loading, error } = Stores.Projects

  // Chat state
  const [chatInput, setChatInput] = useState('')
  const [selectedAssistant, setSelectedAssistant] = useState('Claude Sonnet 4')

  useEffect(() => {
    if (projectId) {
      loadProjectWithDetails(projectId).catch((error: any) => {
        message.error(error?.message || t('common.failedToUpdate'))
        navigate('/projects')
      })
    }
  }, [projectId])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProjectsStoreError()
    }
  }, [error, message])

  const handleSendMessage = () => {
    if (!chatInput.trim()) return

    // TODO: Start new conversation with the input
    console.log('Starting new conversation with:', chatInput)
    setChatInput('')
  }

  if (loading || !currentProject) {
    return <Typography.Text>Loading...</Typography.Text>
  }

  return (
    <PageContainer>
      <Flex className={'w-full h-full gap-8'}>
        {/* Left Side - Chat Input and Conversations */}
        <Flex vertical className={'flex-1 h-full'}>
          {/* Header */}
          <Flex className="justify-between">
            <Title level={2}>{currentProject.name}</Title>
          </Flex>

          {/* Chat Input */}
          <Flex className={'min-h-62'}>
            <Flex className={'flex-col w-full self-center'}>
              <Flex gap="small">
                <Select
                  value={selectedAssistant}
                  onChange={setSelectedAssistant}
                  options={mockAssistants}
                  style={{ width: 150 }}
                />
                <Button
                  type="primary"
                  icon={<ArrowUpOutlined />}
                  onClick={handleSendMessage}
                  disabled={!chatInput.trim()}
                />
              </Flex>
              <TextArea
                value={chatInput}
                onChange={e => setChatInput(e.target.value)}
                placeholder="How can I help you today?"
                autoSize={{ minRows: 2, maxRows: 4 }}
                onPressEnter={e => {
                  if (!e.shiftKey) {
                    e.preventDefault()
                    handleSendMessage()
                  }
                }}
              />
            </Flex>
          </Flex>

          {/* Recent Conversations */}
          <RecentConversationsSection />
        </Flex>

        {/* Right Side - Project Knowledge */}
        <ProjectKnowledgeCard />
      </Flex>
      <ProjectFormDrawer />
    </PageContainer>
  )
}
