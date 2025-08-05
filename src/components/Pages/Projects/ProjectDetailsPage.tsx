import { App, Flex, Typography } from 'antd'
import React, { useEffect, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { useProjectStore } from '../../../store'
import { PageContainer } from '../../common/PageContainer.tsx'
import { ChatInput } from '../../Chat/ChatInput.tsx'
import { ConversationHistory } from '../../common/ConversationHistory'
import { ProjectFormDrawer } from './ProjectFormDrawer.tsx'
import { ProjectKnowledgeCard } from './ProjectKnowledgeCard.tsx'

const { Title } = Typography

export const ProjectDetailsPage: React.FC = () => {
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()
  const searchBoxContainerRef = useRef<HTMLDivElement>(null)

  // Project store
  const { project, loading, error, clearError } = useProjectStore(projectId)

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, message])

  if (loading || !project) {
    return <Typography.Text>Loading...</Typography.Text>
  }

  return (
    <PageContainer>
      <Flex className={'w-full h-full gap-8 overflow-y-hidden'}>
        {/* Left Side - Chat Input and Conversations */}
        <Flex vertical className={'flex-1 h-full'}>
          {/* Header */}
          <Flex className="justify-between">
            <Title level={2}>{project.name}</Title>
          </Flex>

          {/* Chat Input */}
          <Flex className={'min-h-62'}>
            <Flex className={'flex-col w-full self-center'}>
              <ChatInput />
            </Flex>
          </Flex>

          {/* Recent Conversations */}
          <Flex className={'flex-col gap-3 flex-1'}>
            <Flex justify="space-between" align="center">
              <Typography.Title level={5}>
                Recent Conversations
              </Typography.Title>
              <div ref={searchBoxContainerRef} />
            </Flex>
            <ConversationHistory
              getSearchBoxContainer={() => searchBoxContainerRef.current}
            />
          </Flex>
        </Flex>

        {/* Right Side - Project Knowledge */}
        <ProjectKnowledgeCard />
      </Flex>
      <ProjectFormDrawer />
    </PageContainer>
  )
}
