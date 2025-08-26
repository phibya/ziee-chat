import { App, Button, Card, Result, theme, Typography } from 'antd'
import React, { useEffect, useRef } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { openRAGInstanceDrawer, useRAGInstanceStore } from '../../../store'
import { RagFormDrawer } from './RagFormDrawer.tsx'
import { RagKnowledgeCard } from './RagKnowledgeCard.tsx'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'
import { TitleBarWrapper } from '../../common/TitleBarWrapper.tsx'
import { IoIosArrowBack, IoIosArrowForward } from 'react-icons/io'
import { FiEdit } from 'react-icons/fi'
import { useWindowMinSize } from '../../hooks/useWindowMinSize.ts'
import { PiFiles, PiSmileySadLight } from 'react-icons/pi'
import { Drawer } from '../../common/Drawer.tsx'

export const RagDetailsPage: React.FC = () => {
  const { message } = App.useApp()
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const navigate = useNavigate()
  const windowMinSize = useWindowMinSize()
  const [isKnowledgeCardOpen, setIsKnowledgeCardOpen] = React.useState(false)
  const { token } = theme.useToken()
  const knowledgeCardHeaderRef = useRef<HTMLDivElement>(null)

  // RAG instance store
  const { ragInstance, loading, error, clearError } = useRAGInstanceStore(ragInstanceId)

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, message])

  if (!loading && !ragInstance) {
    return (
      <div className={'w-full h-full flex items-center justify-center'}>
        <Result
          icon={
            <div className={'w-full flex items-center justify-center text-8xl'}>
              <PiSmileySadLight />
            </div>
          }
          title="RAG Instance Not Found"
          subTitle="The RAG instance you are looking for does not exist or has been deleted."
          extra={
            <Button type="primary" onClick={() => navigate('/rags')}>
              Go to RAG Instances
            </Button>
          }
        />
      </div>
    )
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
                  onClick={() => navigate('/rags')}
                >
                  All RAG Instances
                </Button>
                <IoIosArrowForward className={'mx-2 text-md'} />
                <Typography.Title
                  level={5}
                  className="!m-0 !leading-tight px-1 flex-1 !font-semibold"
                  ellipsis={true}
                >
                  {ragInstance?.name}
                </Typography.Title>
              </div>
            ) : (
              <div className={'h-full flex items-center'}>
                <Button
                  type={'text'}
                  className={'!pl-0 !pr-2'}
                  onClick={() => navigate('/rags')}
                >
                  <IoIosArrowBack />
                  All RAG Instances
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
                onClick={() => openRAGInstanceDrawer(ragInstance!)}
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
        {/*Left Side - Query Interface and Settings*/}
        <div className={'flex flex-col flex-1 overflow-y-auto h-full'}>
          {windowMinSize.xs && (
            <div className={'w-full pt-3 pb-6'}>
              <Typography.Title
                level={4}
                className="!m-0 !leading-tight px-3 !font-semibold"
              >
                {ragInstance?.name}
              </Typography.Title>
            </div>
          )}
          <div
            className={
              'flex flex-col w-full px-3 flex-1 justify-center min-h-72'
            }
          >
            <div className={'w-full flex flex-col justify-center'}>
              <div className="text-3xl font-light mb-4 text-center">
                RAG Query Interface
              </div>
              <div className="text-center text-gray-500 mb-4">
                Query interface will be implemented in Phase 4
              </div>
            </div>
          </div>
        </div>
        {/* Right Side - RAG Knowledge/Files */}
        {!windowMinSize.md ? (
          <div className={`p-3 w-96 h-full`}>
            <Card
              className="overflow-y-hidden flex flex-col w-full h-full"
              classNames={{
                body: '!p-0 flex flex-col relative overflow-y-hidden flex-1',
              }}
              styles={{
                body: {
                  backgroundColor: token.colorBgContainer,
                },
              }}
            >
              <RagKnowledgeCard />
            </Card>
          </div>
        ) : (
          <Drawer
            title={
              <div
                className={'flex-1 pr-2 pt-0'}
                ref={knowledgeCardHeaderRef}
              />
            }
            open={isKnowledgeCardOpen}
            onClose={() => setIsKnowledgeCardOpen(false)}
            maskClosable
            classNames={{
              body: '!px-0 !pt-0',
            }}
          >
            <div className={'h-full w-full flex flex-col gap-2'}>
              <div
                className={'flex flex-col relative overflow-y-hidden flex-1'}
              >
                <RagKnowledgeCard
                  getHeaderRef={() => knowledgeCardHeaderRef.current}
                />
              </div>
            </div>
          </Drawer>
        )}
      </div>

      <RagFormDrawer />
    </div>
  )
}