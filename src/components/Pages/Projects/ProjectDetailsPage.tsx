import { App, Flex, Typography } from 'antd'
import React, { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import {
  clearProjectsStoreError,
  loadProjectWithDetails,
  Stores,
} from '../../../store'
import { PageContainer } from '../../common/PageContainer.tsx'
import { ChatInput } from '../../Chat/ChatInput.tsx'
import { ProjectFormDrawer } from './ProjectFormDrawer.tsx'
import { ProjectKnowledgeCard } from './ProjectKnowledgeCard.tsx'
import { RecentConversationsSection } from './RecentConversationsSection.tsx'

const { Title } = Typography

export const ProjectDetailsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()

  // Projects store
  const { currentProject, loading, error } = Stores.Projects


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


  if (loading || !currentProject) {
    return <Typography.Text>Loading...</Typography.Text>
  }

  return (
    <PageContainer>
      <Flex className={'w-full h-full gap-8 overflow-y-hidden'}>
        {/* Left Side - Chat Input and Conversations */}
        <Flex vertical className={'flex-1 h-full'}>
          {/* Header */}
          <Flex className="justify-between">
            <Title level={2}>{currentProject.name}</Title>
          </Flex>

          {/* Chat Input */}
          <Flex className={'min-h-62'}>
            <Flex className={'flex-col w-full self-center'}>
              <ChatInput projectId={projectId} />
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
