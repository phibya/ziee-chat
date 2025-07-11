import {
  App,
  Button,
  Card,
  Divider,
  Flex,
  Form,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { FileTextOutlined, FolderOpenOutlined } from '@ant-design/icons'
import { Permission, usePermissions } from '../../../permissions'
import { isDesktopApp } from '../../../api/core'

const { Title, Text } = Typography

export function GeneralSettings() {
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)
  const [experimentalFeatures, setExperimentalFeatures] = useState(false)
  const [spellCheck, setSpellCheck] = useState(true)
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
      spellCheck,
    })
  }, [experimentalFeatures, spellCheck, form])

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
      if ('spellCheck' in changedValues) {
        setSpellCheck(changedValues.spellCheck)
        message.success(
          `Spell check ${changedValues.spellCheck ? 'enabled' : 'disabled'}`,
        )
      }
    } catch {
      message.error('Failed to update settings')
      form.setFieldsValue({
        experimentalFeatures,
        spellCheck,
      })
    }
  }

  return (
    <Flex className={'flex-col gap-3 h-full pb-2'}>
      <Title level={3}>General</Title>

      <Card title="Application">
        <Flex className="flex-col gap-3">
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
          {isDesktopApp && hasPermission(Permission.config.updates.read) && (
            <>
              <Divider style={{ margin: 0 }} />
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
                      Check if a newer version of Jan is available.
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
            </>
          )}
        </Flex>
      </Card>

      {isDesktopApp && hasPermission(Permission.config.experimental.read) && (
        <Card title="Advanced">
          <Form
            form={form}
            onValuesChange={handleFormChange}
            initialValues={{
              experimentalFeatures,
              spellCheck,
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

      {isDesktopApp && hasPermission(Permission.config.dataFolder.read) && (
        <Card title="Data Folder">
          <Flex className="flex-col gap-3">
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
                    /Users/royal/Library/Application Support/Jan/data
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
          </Flex>
        </Card>
      )}

      <Card title="Other">
        <Flex className="flex-col gap-3">
          <Form
            form={form}
            onValuesChange={handleFormChange}
            initialValues={{
              experimentalFeatures,
              spellCheck,
            }}
          >
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>Spell Check</Text>
                <div>
                  <Text type="secondary">
                    Enable spell check for your threads.
                  </Text>
                </div>
              </div>
              <Form.Item
                name="spellCheck"
                valuePropName="checked"
                style={{ margin: 0 }}
              >
                <Switch size="small" />
              </Form.Item>
            </Flex>
          </Form>
          <Divider style={{ margin: 0 }} />
          {hasPermission(Permission.config.factoryReset.read) && (
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>Reset To Factory Settings</Text>
                <div>
                  <Text type="secondary">
                    Restore application to its initial state, erasing all models
                    and chat history. This action is irreversible and
                    recommended only if the application is corrupted.
                  </Text>
                </div>
              </div>
              <Button
                type="primary"
                danger
                disabled={!hasPermission(Permission.config.factoryReset.edit)}
              >
                Reset
              </Button>
            </Flex>
          )}
        </Flex>
      </Card>

      <Card title="Resources">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Documentation</Text>
              <div>
                <Text type="secondary">
                  Learn how to use Jan and explore its features.
                </Text>
              </div>
            </div>
            <Button type="link">View Docs</Button>
          </Flex>
          <Divider style={{ margin: 0 }} />
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>Release Notes</Text>
              <div>
                <Text type="secondary">
                  See what's new in the latest version of Jan.
                </Text>
              </div>
            </div>
            <Button type="link">View Releases</Button>
          </Flex>
        </Space>
      </Card>

      <Card title="Community">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>GitHub</Text>
              <div>
                <Text type="secondary">Contribute to Jan's development.</Text>
              </div>
            </div>
            <Button type="text">
              <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="currentColor"
              >
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
              </svg>
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
              <Text strong>Discord</Text>
              <div>
                <Text type="secondary">
                  Join our community for support and discussions.
                </Text>
              </div>
            </div>
            <Button type="text">
              <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="currentColor"
              >
                <path d="M13.545 2.907a13.227 13.227 0 0 0-3.257-1.011.05.05 0 0 0-.052.025c-.141.25-.297.577-.406.833a12.19 12.19 0 0 0-3.658 0 8.258 8.258 0 0 0-.412-.833.051.051 0 0 0-.052-.025c-1.125.194-2.22.534-3.257 1.011a.041.041 0 0 0-.021.018C.356 6.024-.213 9.047.066 12.032c.001.014.01.028.021.037a13.276 13.276 0 0 0 3.995 2.02.05.05 0 0 0 .056-.019c.308-.42.582-.863.818-1.329a.05.05 0 0 0-.01-.059.051.051 0 0 0-.018-.011 8.875 8.875 0 0 1-1.248-.595.05.05 0 0 1-.02-.066.051.051 0 0 1 .015-.019c.084-.063.168-.129.248-.195a.05.05 0 0 1 .051-.007c2.619 1.196 5.454 1.196 8.041 0a.052.052 0 0 1 .053.007c.08.066.164.132.248.195a.051.051 0 0 1-.004.085 8.254 8.254 0 0 1-1.249.594.05.05 0 0 0-.03.03.052.052 0 0 0 .003.041c.24.465.515.909.817 1.329a.05.05 0 0 0 .056.019 13.235 13.235 0 0 0 4.001-2.02.049.049 0 0 0 .021-.037c.334-3.451-.559-6.449-2.366-9.106a.034.034 0 0 0-.02-.019z" />
              </svg>
            </Button>
          </Flex>
        </Space>
      </Card>

      <Card title="Support">
        <Flex justify="space-between" align="center">
          <div>
            <Text strong>Report an Issue</Text>
            <div>
              <Text type="secondary">
                Found a bug? Help us out by filing an issue on GitHub.
              </Text>
            </div>
          </div>
          <Button type="link">Report Issue</Button>
        </Flex>
      </Card>
    </Flex>
  )
}
