import {
  App,
  Button,
  Card,
  Divider,
  Flex,
  Form,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FileTextOutlined, FolderOpenOutlined } from '@ant-design/icons'
import { Permission, usePermissions } from '../../../../permissions'
import { isDesktopApp } from '../../../../api/core'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'

const { Text } = Typography

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
  }, [experimentalFeatures]) // Removed form from dependencies to prevent infinite rerenders

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
      <SettingsPageContainer title={t('admin.title')}>
        <Card>
          <Text type="secondary">{t('admin.notAvailableDesktop')}</Text>
        </Card>
      </SettingsPageContainer>
    )
  }

  return (
    <SettingsPageContainer title={t('admin.title')}>
      <Flex className={'flex-col gap-3 w-full'}>
        <Card title={t('admin.application')}>
          <Flex vertical className="gap-2 w-full">
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
                    <Text type="secondary">
                      {t('admin.checkForUpdatesDesc')}
                    </Text>
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
          </Flex>
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
                    disabled={
                      !hasPermission(Permission.config.experimental.edit)
                    }
                  />
                </Form.Item>
              </Flex>
            </Form>
          </Card>
        )}

        {hasPermission(Permission.config.dataFolder.read) && (
          <Card title={t('admin.dataFolder')}>
            <Flex vertical className="gap-2 w-full">
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
                <Flex
                  vertical={isMobile}
                  className={isMobile ? 'gap-2 w-full' : 'gap-2'}
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
                </Flex>
              </Flex>
            </Flex>
          </Card>
        )}
      </Flex>
    </SettingsPageContainer>
  )
}
