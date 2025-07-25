import {
  AppstoreOutlined,
  ReloadOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Flex,
  Spin,
  Tabs,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { PageContainer } from '../../common/PageContainer'
import {
  useHubStore,
  initializeHub,
  refreshHub,
} from '../../../store/hub'
import { ModelsTab } from './ModelsTab'
import { AssistantsTab } from './AssistantsTab'

const { Title, Text } = Typography

export function HubPage() {
  const { message } = App.useApp()
  const [activeTab, setActiveTab] = useState('models')

  // Hub store state
  const {
    models,
    assistants,
    hubVersion,
    lastUpdated,
    initialized,
    loading,
    error,
  } = useHubStore()

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
                // Clear error and retry
                useHubStore.setState({ error: null })
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
      <div style={{ height: '100%', overflow: 'auto' }}>
        {/* Header */}
        <div className="mb-6">
          <Flex justify="space-between" align="center" className="mb-2">
            <Title level={2} style={{ margin: 0 }}>
              Hub
            </Title>
            <Flex align="center" gap={16}>
              <Text type="secondary" className="text-sm">
                Version: {hubVersion} • Updated:{' '}
                {new Date(lastUpdated).toLocaleDateString()}
              </Text>
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
          onChange={setActiveTab}
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
      </div>
    </PageContainer>
  )
}