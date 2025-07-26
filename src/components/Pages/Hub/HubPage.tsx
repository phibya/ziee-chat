import {
  AppstoreOutlined,
  ReloadOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import { App, Button, Flex, Spin, Tabs, Typography } from 'antd'
import { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { PageContainer } from '../../common/PageContainer'
import { initializeHub, refreshHub, setHubActiveTab } from '../../../store/hub'
import { ModelsTab } from './ModelsTab'
import { AssistantsTab } from './AssistantsTab'
import { Stores } from '../../../store'

const { Title, Text } = Typography

export function HubPage() {
  const { message } = App.useApp()
  const navigate = useNavigate()
  const { activeTab: urlActiveTab } = useParams<{ activeTab?: string }>()

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

  if (loading && !initialized) {
    return (
      <PageContainer>
        <div className="flex justify-center items-center h-64">
          <Spin size="large" />
          <Text className="ml-4">Loading hub data...</Text>
        </div>
      </PageContainer>
    )
  }

  if (error && !initialized) {
    return (
      <PageContainer>
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
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Flex className="flex h-full w-full flex-col gap-3">
        {/* Header */}
        <div>
          <Flex justify="space-between" align="center">
            <Title level={2} style={{ margin: 0 }}>
              Hub
            </Title>
            <Flex align="center" gap={16}>
              {/*<Text type="secondary" className="text-sm">*/}
              {/*  Version: {hubVersion} • Updated:{" "}*/}
              {/*  {new Date(lastUpdated).toLocaleDateString()}*/}
              {/*</Text>*/}
              <Button
                icon={<ReloadOutlined />}
                onClick={handleRefresh}
                loading={loading}
                type="text"
              >
                Refresh
              </Button>
            </Flex>
          </Flex>
          <Text type="secondary">
            Discover and download models and assistants
          </Text>
        </div>

        {/* Tabs */}
        <Tabs
          activeKey={activeTab}
          onChange={key => {
            setHubActiveTab(key)
            navigate(`/hub/${key}`)
          }}
          className="mb-6"
          items={[
            {
              key: 'models',
              label: (
                <span>
                  <AppstoreOutlined />
                  Models ({models.length})
                </span>
              ),
              children: <ModelsTab />,
            },
            {
              key: 'assistants',
              label: (
                <span>
                  <RobotOutlined />
                  Assistants ({assistants.length})
                </span>
              ),
              children: <AssistantsTab />,
            },
          ]}
        />
      </Flex>
    </PageContainer>
  )
}
