import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Card,
  Form,
  Input,
  Checkbox,
  Button,
  Typography,
  Alert,
  Flex,
  App,
} from 'antd'
import {
  GlobalOutlined,
  LockOutlined,
  PlayCircleOutlined,
  StopOutlined,
} from '@ant-design/icons'
import { SettingsPageContainer } from './common/SettingsPageContainer'
import { isTauriView } from '../../../api/core'
import { Permission, usePermissions } from '../../../permissions'
import { Stores } from '../../../store'
import {
  loadNgrokSettings,
  updateNgrokSettings,
  startNgrokTunnel,
  stopNgrokTunnel,
  refreshNgrokStatus,
  updateAccountPassword,
} from '../../../store/admin/ngrokSettings'

const { Text } = Typography

export function NgrokSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [configForm] = Form.useForm()
  const [passwordForm] = Form.useForm()
  const { hasPermission } = usePermissions()

  // Desktop-only feature check
  if (!isTauriView) {
    return (
      <SettingsPageContainer title={t('ngrok.title')}>
        <Card>
          <Text type="secondary">{t('ngrok.desktopOnly')}</Text>
        </Card>
      </SettingsPageContainer>
    )
  }

  // Permission check
  const canRead = hasPermission(Permission.config.ngrok.read)
  const canEdit = hasPermission(Permission.config.ngrok.edit)

  if (!canRead) {
    return (
      <SettingsPageContainer title={t('ngrok.title')}>
        <Card>
          <Text type="secondary">{t('permissions.insufficient')}</Text>
        </Card>
      </SettingsPageContainer>
    )
  }

  // Store data
  const { ngrokSettings, ngrokStatus, loadingSettings, loadingStatus, error } =
    Stores.AdminNgrokSettings

  // Load data on mount
  useEffect(() => {
    loadNgrokSettings()
    refreshNgrokStatus()

    // Refresh status every 10 seconds when tunnel is active
    const interval = setInterval(() => {
      if (ngrokStatus?.tunnel_active) {
        refreshNgrokStatus()
      }
    }, 10000)

    return () => clearInterval(interval)
  }, [ngrokStatus?.tunnel_active])

  // Form handlers
  const handleConfigSave = async (values: any) => {
    try {
      await updateNgrokSettings(values)
      message.success(t('ngrok.settingsSaved'))
    } catch (_error) {
      message.error(t('ngrok.settingsError'))
    }
  }

  const handlePasswordSave = async (values: any) => {
    try {
      await updateAccountPassword({
        ...(values.current_password && {
          current_password: values.current_password,
        }),
        new_password: values.new_password,
      })
      message.success(t('ngrok.passwordUpdated'))

      // Clear password fields
      passwordForm.setFieldsValue({
        current_password: '',
        new_password: '',
      })
    } catch (_error) {
      message.error(t('ngrok.settingsError'))
    }
  }

  const handleStartTunnel = async () => {
    try {
      await startNgrokTunnel()
      message.success(t('ngrok.tunnelStarted'))
    } catch (_error) {
      console.log({ _error })
      message.error(t('ngrok.tunnelStartError'))
    }
  }

  const handleStopTunnel = async () => {
    try {
      await stopNgrokTunnel()
      message.success(t('ngrok.tunnelStopped'))
    } catch (_error) {
      console.log({ _error })
      message.error(t('ngrok.tunnelStopError'))
    }
  }

  const handleCopyUrl = () => {
    if (ngrokSettings?.tunnel_url) {
      navigator.clipboard.writeText(ngrokSettings.tunnel_url)
      message.success(t('ngrok.urlCopied'))
    }
  }

  // Check if API key is present (form value or saved settings)
  const apiKeyValue = ngrokSettings?.api_key
  const hasApiKey = Boolean(apiKeyValue?.trim())

  return (
    <SettingsPageContainer title={t('ngrok.title')}>
      <div className="flex flex-col gap-3 flex-wrap w-full">
        {/* Configuration Card */}
        <Card title={t('ngrok.configuration')} loading={loadingSettings}>
          <Form
            form={configForm}
            layout="vertical"
            initialValues={ngrokSettings || {}}
            onFinish={handleConfigSave}
            disabled={!canEdit}
          >
            <Form.Item
              name="api_key"
              label={t('ngrok.apiKey')}
              tooltip={t('ngrok.apiKeyTooltip')}
              rules={[{ required: true, message: t('ngrok.apiKeyRequired') }]}
            >
              <Input.Password
                placeholder={t('ngrok.apiKeyPlaceholder')}
                prefix={<LockOutlined />}
              />
            </Form.Item>

            <Form.Item name="auto_start" valuePropName="checked">
              <Checkbox>{t('ngrok.autoStart')}</Checkbox>
            </Form.Item>

            {canEdit && (
              <Form.Item>
                <Button
                  type="primary"
                  htmlType="submit"
                  loading={loadingSettings}
                >
                  {t('common.save')}
                </Button>
              </Form.Item>
            )}
          </Form>
        </Card>

        {/* Password Change Card */}
        <Card title={t('ngrok.accountSecurity')}>
          <Form
            form={passwordForm}
            layout="vertical"
            onFinish={handlePasswordSave}
            disabled={!canEdit}
          >
            {!isTauriView && (
              <Form.Item
                name="current_password"
                label={t('ngrok.currentPassword')}
                tooltip={t('ngrok.currentPasswordTooltip')}
                rules={[
                  {
                    required: true,
                    message: t('ngrok.currentPasswordRequired'),
                  },
                ]}
              >
                <Input.Password
                  placeholder={t('ngrok.currentPasswordPlaceholder')}
                  prefix={<LockOutlined />}
                />
              </Form.Item>
            )}

            <Form.Item
              name="new_password"
              label={t('ngrok.newPassword')}
              tooltip={t('ngrok.newPasswordTooltip')}
              rules={[{ min: 8, message: t('ngrok.passwordMinLength') }]}
            >
              <Input.Password
                placeholder={t('ngrok.newPasswordPlaceholder')}
                prefix={<LockOutlined />}
              />
            </Form.Item>

            {canEdit && (
              <Form.Item>
                <Button
                  type="primary"
                  htmlType="submit"
                  loading={loadingSettings}
                >
                  {t('ngrok.updatePassword')}
                </Button>
              </Form.Item>
            )}
          </Form>
        </Card>

        {/* Tunnel Control Card */}
        {ngrokSettings && (
          <Card title={t('ngrok.tunnelControl')}>
            <div className="flex flex-col gap-3 flex-wrap w-full">
              {/* Status Display */}
              <Flex justify="space-between" align="center">
                <Text strong>{t('ngrok.status')}:</Text>
                <Text
                  type={ngrokStatus?.tunnel_active ? 'success' : 'secondary'}
                >
                  {ngrokStatus?.tunnel_status || t('ngrok.inactive')}
                </Text>
              </Flex>

              {/* Tunnel URL */}
              {ngrokSettings.tunnel_url && (
                <Flex justify="space-between" align="center">
                  <Text strong>{t('ngrok.tunnelUrl')}:</Text>
                  <div className="flex gap-3 flex-wrap">
                    <Text code copyable={{ onCopy: handleCopyUrl }}>
                      {ngrokSettings.tunnel_url}
                    </Text>
                  </div>
                </Flex>
              )}

              {/* Control Buttons */}
              {canEdit && (
                <div className="flex gap-3 flex-wrap">
                  {!ngrokStatus?.tunnel_active ? (
                    <Button
                      type="primary"
                      icon={<PlayCircleOutlined />}
                      onClick={handleStartTunnel}
                      loading={loadingStatus}
                      disabled={!hasApiKey}
                    >
                      {t('ngrok.startTunnel')}
                    </Button>
                  ) : (
                    <Button
                      danger
                      icon={<StopOutlined />}
                      onClick={handleStopTunnel}
                      loading={loadingStatus}
                    >
                      {t('ngrok.stopTunnel')}
                    </Button>
                  )}

                  <Button
                    icon={<GlobalOutlined />}
                    onClick={() => refreshNgrokStatus()}
                  >
                    {t('ngrok.refreshStatus')}
                  </Button>
                </div>
              )}

              {/* Error Display */}
              {error && <Alert message={error} type="error" showIcon />}
            </div>
          </Card>
        )}
      </div>
    </SettingsPageContainer>
  )
}
