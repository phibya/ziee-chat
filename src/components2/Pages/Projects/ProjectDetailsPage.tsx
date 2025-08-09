import { App, Button, Card, Flex, theme, Typography } from 'antd'
import React, { useEffect, useRef } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { openProjectDrawer, useProjectStore } from '../../../store'
import { ChatInput } from '../Chat/ChatInput.tsx'
import { ConversationHistory } from '../../Common/ConversationHistory'
import { ProjectFormDrawer } from './ProjectFormDrawer.tsx'
import { ProjectKnowledgeCard } from './ProjectKnowledgeCard.tsx'
import { TauriDragRegion } from '../../Common/TauriDragRegion.tsx'
import { TitleBarWrapper } from '../../Common/TitleBarWrapper.tsx'
import { IoIosArrowBack, IoIosArrowForward } from 'react-icons/io'
import { FiEdit } from 'react-icons/fi'
import { useWindowMinSize } from '../../hooks/useWindowMinSize.ts'
import { PiFiles } from 'react-icons/pi'
import { Drawer } from '../../Common/Drawer.tsx'

export const ProjectDetailsPage: React.FC = () => {
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()
  const searchBoxContainerRef = useRef<HTMLDivElement>(null)
  const navigate = useNavigate()
  const windowMinSize = useWindowMinSize()
  const [isKnowledgeCardOpen, setIsKnowledgeCardOpen] = React.useState(false)
  const { token } = theme.useToken()

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
    <div className="h-full flex flex-col w-full overflow-hidden">
      <div className="w-full h-[50px]">
        <TitleBarWrapper>
          <div className="h-full flex items-center justify-between w-full">
            <TauriDragRegion
              className={'h-full w-full absolute top-0 left-0'}
            />
            {!windowMinSize.xs ? (
              <div
                className={'h-full flex items-center flex-1 overflow-hidden'}
              >
                <Button
                  type={'text'}
                  className={'!px-1'}
                  onClick={() => navigate('/projects')}
                >
                  All projects
                </Button>
                <IoIosArrowForward className={'mx-2 text-md'} />
                <Typography.Title
                  level={5}
                  className="!m-0 !leading-tight px-1 flex-1 !font-semibold"
                  ellipsis={true}
                >
                  {project.name}
                </Typography.Title>
              </div>
            ) : (
              <div className={'h-full flex items-center'}>
                <Button
                  type={'text'}
                  className={'!pl-0 !pr-2'}
                  onClick={() => navigate('/projects')}
                >
                  <IoIosArrowBack />
                  All projects
                </Button>
              </div>
            )}
            <div className={'flex items-center justify-between gap-1'}>
              {windowMinSize.md && (
                <Button
                  type={'text'}
                  icon={<PiFiles />}
                  style={{
                    fontSize: '20px',
                  }}
                  onClick={() => setIsKnowledgeCardOpen(true)}
                />
              )}
              <Button
                type={'text'}
                icon={<FiEdit />}
                style={{
                  fontSize: '20px',
                }}
                onClick={() => openProjectDrawer(project)}
              />
            </div>
          </div>
        </TitleBarWrapper>
      </div>
      <div
        className={
          'w-full h-full overflow-y-scroll max-w-6xl self-center flex-wrap flex'
        }
      >
        {/*Left Side - Chat Input and Conversations*/}
        <div className={'flex flex-col flex-1 overflow-y-hidden'}>
          {windowMinSize.xs && (
            <div className={'w-full py-6'}>
              <Typography.Title
                level={4}
                className="!m-0 !leading-tight px-3 !font-semibold"
              >
                {project.name}
              </Typography.Title>
            </div>
          )}
          <div
            className={
              'flex flex-col w-full px-3 flex-1 justify-center min-h-72'
            }
          >
            <div className={'w-full'}>
              <div className="text-3xl font-light mb-4 text-center">
                Hi! How can I assist you with your project?
              </div>
              <Flex className={'flex-col w-full self-center'}>
                <ChatInput />
              </Flex>
            </div>
          </div>
          {/* Recent Conversations */}
          <div
            className={
              'flex flex-col gap-3 overflow-y-hidden overflow-x-visible min-h-1/2'
            }
          >
            <Flex
              justify="space-between"
              align="center"
              className={'w-full flex-wrap !px-3 gap-x-4'}
            >
              <Typography.Title level={5}>
                Recent Conversations
              </Typography.Title>
              <div
                className={'flex-1 max-w-sm min-w-[200px]'}
                ref={searchBoxContainerRef}
              />
            </Flex>
            <div className={'flex flex-1 overflow-auto'}>
              <ConversationHistory
                getSearchBoxContainer={() => searchBoxContainerRef.current}
              />
            </div>
          </div>
        </div>
        {/* Right Side - Project Knowledge */}
        {!windowMinSize.md ? (
          <div className={`p-3 w-96`}>
            <Card
              className="overflow-y-hidden flex flex-col w-full h-full"
              classNames={{
                body: 'flex flex-col relative overflow-y-hidden flex-1',
              }}
              styles={{
                body: {
                  backgroundColor: token.colorBgContainer,
                },
              }}
            >
              <ProjectKnowledgeCard />
            </Card>
          </div>
        ) : (
          <Drawer
            open={isKnowledgeCardOpen}
            onClose={() => setIsKnowledgeCardOpen(false)}
            maskClosable
          >
            <div className={'h-full w-full flex flex-col gap-2'}>
              <div
                className={'flex flex-col relative overflow-y-hidden flex-1'}
              >
                <ProjectKnowledgeCard />
              </div>
              <div>
                <Button onClick={() => setIsKnowledgeCardOpen(false)}>
                  Close
                </Button>
              </div>
            </div>
          </Drawer>
        )}
      </div>

      <ProjectFormDrawer />
    </div>
  )
}
