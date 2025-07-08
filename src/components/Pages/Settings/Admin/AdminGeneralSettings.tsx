import {
  Button,
  Card,
  Divider,
  Flex,
  Form,
  message,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { FileTextOutlined, FolderOpenOutlined } from '@ant-design/icons'
import { Permission, usePermissions } from '../../../../permissions'
import { isDesktopApp } from '../../../../api/core'

const { Title, Text } = Typography

export function AdminGeneralSettings() {
  const [form] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)
  const [experimentalFeatures, setExperimentalFeatures] = useState(false)
  const { hasPermission } = usePermissions()

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  useEffect(() => {
    form.setFieldsValue({
      experimentalFeatures,
    })
  }, [experimentalFeatures, form])

  const handleFormChange = async (changedValues: any) => {
    try {
      if ('experimentalFeatures' in changedValues) {
        if (!hasPermission(Permission.config.experimental.edit)) {
          message.error(
            'You do not have permission to change experimental features',
          )
          form.setFieldsValue({ experimentalFeatures })
          return
        }
        setExperimentalFeatures(changedValues.experimentalFeatures)
        message.success(
          `Experimental features ${changedValues.experimentalFeatures ? 'enabled' : 'disabled'}`,
        )
      }
    } catch {
      message.error('Failed to update settings')
      form.setFieldsValue({
        experimentalFeatures,
      })
    }
  }

  // Only show these settings for web app (not desktop)
  if (isDesktopApp) {
    return (
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <Title level={3}>Admin General</Title>
        <Card>
          <Text type="secondary">
            Admin General settings are not available in desktop mode.
          </Text>
        </Card>
      </Space>
    )
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Admin General</Title>

      <Card title="Application">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>App Version</Text>
              <div>
                <Text type="secondary">v0.6.4</Text>
              </div>
            </div>
            <Text type="secondary">v0.6.4</Text>
          </Flex>
          <Divider style={{ margin: 0 }} />
          {hasPermission(Permission.config.updates.read) && (
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>Check for Updates</Text>
                <div>
                  <Text type="secondary">
                    Check if a newer version of the application is available.
                  </Text>
                </div>
              </div>
              <Button
                type="default"
                disabled={!hasPermission(Permission.config.updates.edit)}
              >
                Check for Updates
              </Button>
            </Flex>
          )}
        </Space>
      </Card>

      {hasPermission(Permission.config.experimental.read) && (
        <Card title="Advanced">
          <Form
            form={form}
            onValuesChange={handleFormChange}
            initialValues={{
              experimentalFeatures,
            }}
          >
            <Flex justify="space-between" align="center">
              <div>
                <Text strong>Experimental Features</Text>
                <div>
                  <Text type="secondary">
                    Enable experimental features. They may be unstable or change
                    at any time.
                  </Text>
                </div>
              </div>
              <Form.Item
                name="experimentalFeatures"
                valuePropName="checked"
                style={{ margin: 0 }}
              >
                <Switch
                  size="small"
                  disabled={!hasPermission(Permission.config.experimental.edit)}
                />
              </Form.Item>
            </Flex>
          </Form>
        </Card>
      )}

      {hasPermission(Permission.config.dataFolder.read) && (
        <Card title="Data Folder">
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>App Data</Text>
                <div>
                  <Text type="secondary">
                    Default location for messages and other user data.
                  </Text>
                </div>
                <div>
                  <Text type="secondary" style={{ fontSize: '12px' }}>
                    /var/lib/app/data
                  </Text>
                </div>
              </div>
              <Button
                type="default"
                icon={<FolderOpenOutlined />}
                disabled={!hasPermission(Permission.config.dataFolder.edit)}
              >
                Change Location
              </Button>
            </Flex>
            <Divider style={{ margin: 0 }} />
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>App Logs</Text>
                <div>
                  <Text type="secondary">View detailed logs of the App.</Text>
                </div>
              </div>
              <Space
                direction={isMobile ? 'vertical' : 'horizontal'}
                style={{ width: isMobile ? '100%' : 'auto' }}
              >
                <Button
                  type="default"
                  icon={<FileTextOutlined />}
                  block={isMobile}
                  disabled={!hasPermission(Permission.config.dataFolder.edit)}
                >
                  Open Logs
                </Button>
                <Button
                  type="default"
                  icon={<FolderOpenOutlined />}
                  block={isMobile}
                  disabled={!hasPermission(Permission.config.dataFolder.edit)}
                >
                  Show in Finder
                </Button>
              </Space>
            </Flex>
          </Space>
        </Card>
      )}
    </Space>
  )
}
