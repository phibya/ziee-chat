import {
  AppstoreOutlined,
  ReloadOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import { App, Button, Dropdown, Flex, Segmented, theme, Typography } from 'antd'
import React, { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { TitleBarWrapper } from '../../common/TitleBarWrapper'
import { TauriDragRegion } from '../../common/TauriDragRegion'
import {
  refreshHubAssistants,
  refreshHubModels,
  setHubActiveTab,
} from '../../../store/hub'
import { ModelsTab } from './ModelsTab'
import { AssistantsTab } from './AssistantsTab'
import { Stores } from '../../../store'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { IoIosArrowDown, IoIosArrowForward } from 'react-icons/io'
import { hasPermission } from '../../../permissions/utils.ts'
import { Permission } from '../../../types'
import {
  PagePermissionGuard403,
  PermissionGuard,
} from '../../Auth/PermissionGuard.tsx'
import { DivScrollY } from '../../common/DivScrollY.tsx'

export function HubPage() {
  const { message } = App.useApp()
  const navigate = useNavigate()
  const { activeTab: urlActiveTab } = useParams<{ activeTab?: string }>()
  const mainContentMinSize = useMainContentMinSize()
  const { token } = theme.useToken()

  // Hub store state
  const { modelsLoading, assistantsLoading, lastActiveTab } = Stores.Hub

  // Valid tab names
  const validTabs = ['models', 'assistants']

  // Default to lastActiveTab from store if no URL tab, otherwise use URL tab or 'models'
  const activeTab =
    urlActiveTab && validTabs.includes(urlActiveTab)
      ? urlActiveTab
      : !urlActiveTab
        ? lastActiveTab || 'models'
        : 'models'

  // Redirect to valid tab if current tab is invalid, or redirect to last active tab if no tab in URL
  useEffect(() => {
    const availabelTabs = [
      hasPermission([Permission.HubModelsRead]) && 'models',
      hasPermission([Permission.HubAssistantsRead]) && 'assistants',
    ].filter(Boolean) as string[]

    if (!availabelTabs.includes(activeTab)) {
      const newTab = availabelTabs[0]
      if (!newTab) return
      navigate(`/hub/${newTab}`, {
        replace: true,
      })
      return
    }

    if (activeTab !== urlActiveTab) {
      navigate(`/hub/${activeTab}`, {
        replace: true,
      })
    }
  }, [urlActiveTab, activeTab])

  useEffect(() => {
    setHubActiveTab(activeTab)
  }, [activeTab])

  const handleRefresh = async () => {
    try {
      if (activeTab === 'models') {
        await refreshHubModels()
        message.success('Hub models refreshed successfully')
      } else if (activeTab === 'assistants') {
        await refreshHubAssistants()
        message.success('Hub assistants refreshed successfully')
      }
    } catch (err) {
      console.error('Failed to refresh hub data:', err)
      message.error('Failed to refresh hub data')
    }
  }

  return (
    <Flex className="flex flex-col w-full h-full overflow-hidden">
      <TitleBarWrapper>
        <TauriDragRegion className={'h-full w-full absolute top-0 left-0'} />
        <div className="flex items-center justify-between w-full h-[50px]">
          <Typography.Title
            level={3}
            ellipsis
            className={'!m-0 !leading-tight'}
          >
            Hub
          </Typography.Title>

          {/* Desktop: Show segmented control in title bar center */}
          {!mainContentMinSize.xs && (
            <div className="flex-1 flex h-full justify-center items-center">
              <Segmented
                value={activeTab}
                onChange={(value: string) => {
                  setHubActiveTab(value)
                  navigate(`/hub/${value}`)
                }}
                className={`
                [&_.ant-segmented-item-label]:!px-4
                [&_.ant-segmented-item-label]:!py-1
                `}
                style={{
                  backgroundColor: token.colorBgMask,
                }}
                shape="round"
                options={
                  [
                    hasPermission([Permission.HubModelsRead]) && {
                      value: 'models',
                      label: (
                        <Flex align="center" gap={4}>
                          <AppstoreOutlined />
                          Models
                        </Flex>
                      ),
                    },
                    hasPermission([Permission.HubAssistantsRead]) && {
                      value: 'assistants',
                      label: (
                        <Flex align="center" gap={4}>
                          <RobotOutlined />
                          Assistants
                        </Flex>
                      ),
                    },
                  ].filter(e => !!e) as {
                    value: string
                    label: React.ReactNode
                  }[]
                }
              />
            </div>
          )}

          {/* Mobile: Show dropdown in title bar */}
          {mainContentMinSize.xs && (
            <div className="flex flex-1 items-center px-2">
              <IoIosArrowForward />
              <Dropdown
                menu={{
                  items: [
                    hasPermission([Permission.HubModelsRead]) && {
                      key: 'models',
                      label: (
                        <Flex className={'gap-2'}>
                          <AppstoreOutlined />
                          Models
                        </Flex>
                      ),
                    },
                    hasPermission([Permission.HubAssistantsRead]) && {
                      key: 'assistants',
                      label: (
                        <Flex className={'gap-2'}>
                          <RobotOutlined />
                          Assistants
                        </Flex>
                      ),
                    },
                  ].filter(e => !!e) as {
                    key: string
                    label: React.ReactNode
                  }[],
                  onClick: ({ key }) => {
                    setHubActiveTab(key)
                    navigate(`/hub/${key}`)
                  },
                  selectedKeys: [activeTab],
                }}
                trigger={['click']}
              >
                <Button type="text" className={'!pt-1'}>
                  {activeTab === 'models' ? (
                    <PermissionGuard permissions={[Permission.HubModelsRead]}>
                      <AppstoreOutlined /> Models
                    </PermissionGuard>
                  ) : (
                    <PermissionGuard
                      permissions={[Permission.HubAssistantsRead]}
                    >
                      <RobotOutlined /> Assistants
                    </PermissionGuard>
                  )}{' '}
                  <IoIosArrowDown />
                </Button>
              </Dropdown>
            </div>
          )}

          <Button
            icon={<ReloadOutlined />}
            onClick={handleRefresh}
            loading={modelsLoading || assistantsLoading}
            type="text"
          >
            {mainContentMinSize.xs ? null : 'Refresh'}
          </Button>
        </div>
      </TitleBarWrapper>
      <div className="flex flex-col w-full h-full overflow-hidden">
        <DivScrollY className="flex flex-1 w-full flex-col">
          <div className="max-w-4xl w-full flex flex-col self-center">
            <DivScrollY className={`flex-1 h-full w-full`}>
              <div className={'flex flex-col py-3'}>
                {activeTab === 'models' ? (
                  <PagePermissionGuard403
                    permissions={[Permission.HubModelsRead]}
                  >
                    <ModelsTab />
                  </PagePermissionGuard403>
                ) : (
                  <PagePermissionGuard403
                    permissions={[Permission.HubAssistantsRead]}
                  >
                    <AssistantsTab />
                  </PagePermissionGuard403>
                )}
              </div>
            </DivScrollY>
          </div>
        </DivScrollY>
      </div>
    </Flex>
  )
}
