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
import { useTranslation } from 'react-i18next'
import { FileTextOutlined, FolderOpenOutlined } from '@ant-design/icons'
import { Permission, usePermissions } from '../../../../permissions'
import { isDesktopApp } from '../../../../api/core'

const { Title, Text } = Typography

export function AdminGeneralSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
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
          message.error(t('admin.noPermissionExperimental'))
          form.setFieldsValue({ experimentalFeatures })
          return
        }
        setExperimentalFeatures(changedValues.experimentalFeatures)
        message.success(
          changedValues.experimentalFeatures
            ? t('admin.experimentalEnabled')
            : t('admin.experimentalDisabled'),
        )
      }
    } catch (error: any) {
      message.error(error?.message || t('common.failedToUpdate'))
      form.setFieldsValue({
        experimentalFeatures,
      })
    }
  }

  // Only show these settings for web app (not desktop)
  if (isDesktopApp) {
    return (
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <Title level={3}>{t('admin.title')}</Title>
        <Card>
          <Text type="secondary">{t('admin.notAvailableDesktop')}</Text>
        </Card>
      </Space>
    )
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>{t('admin.title')}</Title>

      <Card title={t('admin.application')}>
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Flex
            justify="space-between"
            align={isMobile ? 'flex-start' : 'center'}
            vertical={isMobile}
            gap={isMobile ? 'small' : 0}
          >
            <div>
              <Text strong>{t('admin.appVersion')}</Text>
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
                <Text strong>{t('admin.checkForUpdates')}</Text>
                <div>
                  <Text type="secondary">{t('admin.checkForUpdatesDesc')}</Text>
                </div>
              </div>
              <Button
                type="default"
                disabled={!hasPermission(Permission.config.updates.edit)}
              >
                {t('admin.checkForUpdates')}
              </Button>
            </Flex>
          )}
        </Space>
      </Card>

      {hasPermission(Permission.config.experimental.read) && (
        <Card title={t('admin.advanced')}>
          <Form
            form={form}
            onValuesChange={handleFormChange}
            initialValues={{
              experimentalFeatures,
            }}
          >
            <Flex justify="space-between" align="center">
              <div>
                <Text strong>{t('admin.experimentalFeatures')}</Text>
                <div>
                  <Text type="secondary">
                    {t('admin.experimentalFeaturesDesc')}
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
        <Card title={t('admin.dataFolder')}>
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>{t('admin.appData')}</Text>
                <div>
                  <Text type="secondary">{t('admin.appDataDesc')}</Text>
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
                {t('admin.changeLocation')}
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
                <Text strong>{t('admin.appLogs')}</Text>
                <div>
                  <Text type="secondary">{t('admin.appLogsDesc')}</Text>
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
                  {t('admin.openLogs')}
                </Button>
                <Button
                  type="default"
                  icon={<FolderOpenOutlined />}
                  block={isMobile}
                  disabled={!hasPermission(Permission.config.dataFolder.edit)}
                >
                  {t('admin.showInFinder')}
                </Button>
              </Space>
            </Flex>
          </Space>
        </Card>
      )}
    </Space>
  )
}
