import { App, Button, Result, Typography } from 'antd'
import React, { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { openRAGInstanceDrawer, useRAGInstanceStore } from '../../../store'
import { RagFormDrawer } from './RagFormDrawer.tsx'
import { RagQueryCard } from './RagQueryCard.tsx'
import { RagInstanceInfoCard } from './RagInstanceInfoCard.tsx'
import { RagDocumentsCard } from './RagDocumentsCard.tsx'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'
import { TitleBarWrapper } from '../../common/TitleBarWrapper.tsx'
import { IoIosArrowBack, IoIosArrowForward } from 'react-icons/io'
import { FiEdit } from 'react-icons/fi'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { PiSmileySadLight } from 'react-icons/pi'
import { Permission } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'

export const RagDetailsPage: React.FC = () => {
  const { message } = App.useApp()
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const navigate = useNavigate()
  const pageMinSize = useMainContentMinSize()

  // RAG instance store
  const { ragInstance, loading, error, clearError } =
    useRAGInstanceStore(ragInstanceId)

  // Check permissions for system instances
  const isSystemInstance = ragInstance?.is_system

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
            {!pageMinSize.xs ? (
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
              <PermissionGuard
                permissions={
                  isSystemInstance
                    ? [Permission.RagAdminInstancesEdit]
                    : [Permission.RagInstancesEdit]
                }
                type="hidden"
              >
                <Button
                  type={'text'}
                  icon={<FiEdit />}
                  style={{
                    fontSize: '20px',
                  }}
                  onClick={() => openRAGInstanceDrawer(ragInstance!)}
                />
              </PermissionGuard>
            </div>
          </div>
        </TitleBarWrapper>
      </div>
      <div className="w-full overflow-y-auto">
        <div className="w-full flex-1 p-3 max-w-4xl mx-auto">
          <div className="flex flex-col gap-3">
            {/* Instance name when in xs size */}
            {pageMinSize.xs && ragInstance && (
              <Typography.Title
                level={4}
                className="!m-0 !mb-1"
                ellipsis={true}
              >
                {ragInstance.name}
              </Typography.Title>
            )}

            {/* 1. Query Interface Card */}
            <RagQueryCard />

            {/* 2. Instance Information Card */}
            <RagInstanceInfoCard />

            {/* 3. Documents Card */}
            <RagDocumentsCard />
          </div>
        </div>
      </div>
      <RagFormDrawer />
    </div>
  )
}
