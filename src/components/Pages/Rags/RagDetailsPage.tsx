import {
  App,
  Button,
  Dropdown,
  Flex,
  Result,
  Segmented,
  theme,
  Typography,
} from 'antd'
import React, { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { openRAGInstanceDrawer, useRAGInstanceStore } from '../../../store'
import { RagFormDrawer } from './RagFormDrawer.tsx'
import { RagQueryTab } from './RagQueryTab.tsx'
import { RagInstanceSettingsTab } from './RagInstanceSettingsTab.tsx'
import { RagDocumentsTab } from './RagDocumentsTab.tsx'
import { RagStatusTab } from './RagStatusTab.tsx'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'
import { TitleBarWrapper } from '../../common/TitleBarWrapper.tsx'
import { IoIosArrowForward } from 'react-icons/io'
import { FiEdit } from 'react-icons/fi'
import { PiSmileySadLight } from 'react-icons/pi'
import { Permission } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import {
  FileOutlined,
  SearchOutlined,
  SettingOutlined,
  DashboardOutlined,
  DownOutlined,
} from '@ant-design/icons'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'

export const RagDetailsPage: React.FC = () => {
  const { message } = App.useApp()
  const { ragInstanceId, tab } = useParams<{
    ragInstanceId: string
    tab?: string
  }>()
  const navigate = useNavigate()
  const { token } = theme.useToken()
  const mainContentMinSize = useMainContentMinSize()

  // Set default tab if none specified
  const activeTab = tab || 'query'

  // Tab options
  const tabOptions = [
    {
      value: 'query',
      label: (
        <Flex align="center" gap={4}>
          <SearchOutlined />
          Query
        </Flex>
      ),
      key: 'query',
    },
    {
      value: 'documents',
      label: (
        <Flex align="center" gap={4}>
          <FileOutlined />
          Documents
        </Flex>
      ),
      key: 'documents',
    },
    {
      value: 'status',
      label: (
        <Flex align="center" gap={4}>
          <DashboardOutlined />
          Status
        </Flex>
      ),
      key: 'status',
    },
    {
      value: 'settings',
      label: (
        <Flex align="center" gap={4}>
          <SettingOutlined />
          Settings
        </Flex>
      ),
      key: 'settings',
    },
  ]

  // Get current tab label for dropdown
  const currentTabOption = tabOptions.find(option => option.value === activeTab)

  // RAG instance store
  const { ragInstance, loading, error, clearError } =
    useRAGInstanceStore(ragInstanceId)

  // Check permissions for system instances
  const isSystemInstance = ragInstance?.is_system

  // Redirect to default tab if no tab specified
  useEffect(() => {
    if (ragInstanceId && !tab) {
      navigate(`/rags/${ragInstanceId}/query`, { replace: true })
    }
  }, [ragInstanceId, tab, navigate])

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
            <div className={'h-full flex items-center flex-1 overflow-hidden'}>
              <Button
                type={'text'}
                className={'!px-1'}
                onClick={() => navigate('/rags')}
              >
                RAGs
              </Button>
              <IoIosArrowForward className={'mx-2 text-md'} />
              <Typography.Title
                level={5}
                className="!m-0 !leading-tight px-1 flex-1 !font-semibold"
                ellipsis={true}
              >
                {ragInstance?.display_name}
              </Typography.Title>
            </div>
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
            <div className={'h-full flex items-center justify-center'}>
              {mainContentMinSize.xxs ? (
                <Dropdown
                  className={'w-full'}
                  menu={{
                    items: tabOptions,
                    onClick: ({ key }) => {
                      navigate(`/rags/${ragInstance?.id}/${key}`)
                    },
                  }}
                >
                  <Button>
                    <div className={'flex items-center w-full'}>
                      <div
                        className={'flex-1 flex items-center justify-center'}
                      >
                        {' '}
                        {currentTabOption?.label}
                      </div>
                      <div>
                        <DownOutlined />
                      </div>
                    </div>
                  </Button>
                </Dropdown>
              ) : (
                <Segmented
                  value={activeTab}
                  onChange={(value: string) => {
                    navigate(`/rags/${ragInstance?.id}/${value}`)
                  }}
                  className={`
                  [&_.ant-segmented-item-label]:!px-4
                  [&_.ant-segmented-item-label]:!py-1
                  w-auto`}
                  style={{
                    backgroundColor: token.colorBgMask,
                  }}
                  shape="round"
                  options={tabOptions.map(option => ({
                    value: option.value,
                    label: option.label,
                  }))}
                />
              )}
            </div>

            {/* Render cards based on active tab */}
            {activeTab === 'query' && <RagQueryTab />}
            {activeTab === 'documents' && <RagDocumentsTab />}
            {activeTab === 'status' && <RagStatusTab />}
            {activeTab === 'settings' && <RagInstanceSettingsTab />}
          </div>
        </div>
      </div>
      <RagFormDrawer />
    </div>
  )
}
