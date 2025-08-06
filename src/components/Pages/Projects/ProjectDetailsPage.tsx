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
      <Flex
        className={
          'w-full h-full gap-3 overflow-y-hidden max-w-6xl self-center !py-3'
        }
      >
        {/* Left Side - Chat Input and Conversations */}
        <Flex vertical className={'flex-1 h-full flex-col overflow-y-hidden'}>
          {/* Header */}
          <Flex className={'!px-3'}>
            <Title level={2}>{project.name}</Title>
          </Flex>

          {/* Chat Input */}
          <Flex className={'min-h-62 !px-3 flex-col content-center'}>
            <div className={'my-auto'}>
              <div className="text-3xl font-light mb-4 text-center">
                Hi! How can I assist you with your project?
              </div>
              <Flex className={'flex-col w-full self-center'}>
                <ChatInput />
              </Flex>
            </div>
          </Flex>

          {/* Recent Conversations */}
          <Flex
            className={
              'flex-col gap-3 flex-1 overflow-y-hidden overflow-x-visible'
            }
          >
            <Flex
              justify="space-between"
              align="center"
              className={'w-full flex-wrap !px-3'}
            >
              <Typography.Title level={5}>
                Recent Conversations
              </Typography.Title>
              <div ref={searchBoxContainerRef} />
            </Flex>
            <div className={'flex flex-1 overflow-auto'}>
              <ConversationHistory
                getSearchBoxContainer={() => searchBoxContainerRef.current}
              />
            </div>
          </Flex>
        </Flex>

        {/* Right Side - Project Knowledge */}
        <div className={'h-full !pr-3 flex'}>
          <ProjectKnowledgeCard />
        </div>
      </Flex>

      <ProjectFormDrawer />
    </PageContainer>
  )
}
