import {
  AppstoreOutlined,
  ReloadOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import { App, Button, Dropdown, Flex, Spin, Tabs, Typography } from 'antd'
import { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { TitleBarWrapper } from '../../Common/TitleBarWrapper'
import { TauriDragRegion } from '../../Common/TauriDragRegion'
import { initializeHub, refreshHub, setHubActiveTab } from '../../../store/hub'
import { ModelsTab } from './ModelsTab'
import { AssistantsTab } from './AssistantsTab'
import { Stores } from '../../../store'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { IoIosArrowDown, IoIosArrowForward } from 'react-icons/io'

const { Text } = Typography

export function HubPage() {
  const { message } = App.useApp()
  const navigate = useNavigate()
  const { activeTab: urlActiveTab } = useParams<{ activeTab?: string }>()
  const mainContentMinSize = useMainContentMinSize()

  // Hub store state
  const { models, assistants, initialized, loading, error, lastActiveTab } =
    Stores.Hub

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
    if (activeTab !== urlActiveTab) {
      navigate(`/hub/${activeTab}`, {
        replace: true,
      })
    }
  }, [urlActiveTab, activeTab])

  useEffect(() => {
    setHubActiveTab(activeTab)
  }, [activeTab])

  useEffect(() => {
    if (!initialized && !loading && !error) {
      initializeHub().catch(err => {
        console.error('Failed to initialize hub:', err)
        message.error('Failed to load hub data')
      })
    }
  }, [initialized, loading, error, message])

  const handleRefresh = async () => {
    try {
      await refreshHub()
      message.success('Hub data refreshed successfully')
    } catch (err) {
      console.error('Failed to refresh hub:', err)
      message.error('Failed to refresh hub data')
    }
  }

  const renderContent = () => {
    if (loading && !initialized) {
      return (
        <div className="flex justify-center items-center h-full">
          <Spin size="large" />
          <Text className="ml-4">Loading hub data...</Text>
        </div>
      )
    }

    if (error && !initialized) {
      return (
        <div className="text-center py-12">
          <Text type="danger">Failed to load hub data: {error}</Text>
          <div className="mt-4">
            <Button
              onClick={() => {
                initializeHub()
              }}
            >
              Retry
            </Button>
          </div>
        </div>
      )
    }

    const TabWrapper = ({ children }: { children: React.ReactNode }) => (
      <div
        className={`flex-1 h-full w-full overflow-y-auto`}
        style={{
          paddingLeft: mainContentMinSize.xs ? 12 : 0,
          paddingRight: mainContentMinSize.xs ? 12 : 0,
        }}
      >
        <div className={'flex flex-col py-3'}>{children}</div>
      </div>
    )

    if (mainContentMinSize.xs) {
      // Mobile: Simple content switching (tabs are controlled via titlebar dropdown)
      return (
        <Flex className="flex h-full w-full flex-col gap-2">
          <div className="flex-1 h-full w-full overflow-hidden">
            <TabWrapper>
              {activeTab === 'models' ? <ModelsTab /> : <AssistantsTab />}
            </TabWrapper>
          </div>
        </Flex>
      )
    }

    // Desktop: Show tabs for navigation only, content rendered separately
    return (
      <Flex className="flex h-full w-full flex-col">
        <div className="pt-1 px-3 max-w-6xl self-center w-full">
          <Tabs
            activeKey={activeTab}
            onChange={(key: string) => {
              setHubActiveTab(key)
              navigate(`/hub/${key}`)
            }}
            items={[
              {
                key: 'models',
                label: (
                  <Flex className={'gap-1'}>
                    <AppstoreOutlined />
                    Models ({models.length})
                  </Flex>
                ),
                children: null, // No content in tabs
              },
              {
                key: 'assistants',
                label: (
                  <Flex className={'gap-1'}>
                    <RobotOutlined />
                    Assistants ({assistants.length})
                  </Flex>
                ),
                children: null, // No content in tabs
              },
            ]}
            tabBarStyle={{
              marginBottom: 0,
            }}
          />
        </div>
        <div className="flex flex-1 w-full overflow-y-auto flex-col">
          <div className="max-w-6xl w-full flex flex-col self-center px-3">
            <TabWrapper>
              {activeTab === 'models' ? <ModelsTab /> : <AssistantsTab />}
            </TabWrapper>
          </div>
        </div>
      </Flex>
    )
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
          {mainContentMinSize.xs && (
            <div className="flex flex-1 items-center px-2">
              <IoIosArrowForward />
              <Dropdown
                menu={{
                  items: [
                    {
                      key: 'models',
                      label: (
                        <Flex className={'gap-2'}>
                          <AppstoreOutlined />
                          Models ({models.length})
                        </Flex>
                      ),
                    },
                    {
                      key: 'assistants',
                      label: (
                        <Flex className={'gap-2'}>
                          <RobotOutlined />
                          Assistants ({assistants.length})
                        </Flex>
                      ),
                    },
                  ],
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
                    <>
                      <AppstoreOutlined /> Models
                    </>
                  ) : (
                    <>
                      <RobotOutlined /> Assistants
                    </>
                  )}{' '}
                  <IoIosArrowDown />
                </Button>
              </Dropdown>
            </div>
          )}
          <Button
            icon={<ReloadOutlined />}
            onClick={handleRefresh}
            loading={loading}
            type="text"
          >
            {mainContentMinSize.xs ? null : 'Refresh'}
          </Button>
        </div>
      </TitleBarWrapper>
      <div className="flex flex-col w-full h-full overflow-hidden">
        {renderContent()}
      </div>
    </Flex>
  )
}
