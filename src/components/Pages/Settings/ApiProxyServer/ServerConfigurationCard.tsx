import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Card,
  Form,
  Input,
  InputNumber,
  Switch,
  Button,
  Typography,
  App,
  Select,
} from 'antd'
import { LockOutlined } from '@ant-design/icons'
import { Permission, usePermissions } from '../../../../permissions'
import { Stores } from '../../../../store'
import { updateApiProxyServerConfig } from '../../../../store/admin/apiProxyServer.ts'

const { Text } = Typography

export function ServerConfigurationCard() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const { hasPermission } = usePermissions()

  // Permission check
  const canEdit = hasPermission(Permission.config.apiProxyServer?.edit)

  // Store data
  const { config, loadingConfig } = Stores.AdminApiProxyServer

  const handleConfigSave = async (values: any) => {
    try {
      console.log('Saving proxy config with values:', values)
      await updateApiProxyServerConfig(values)
      message.success(t('apiProxyServer.configurationSaved'))
    } catch (error) {
      console.error('Failed to save proxy config:', error)
      message.error(t('apiProxyServer.configurationError'))
    }
  }

  // Update form when config changes
  useEffect(() => {
    if (config) {
      form.setFieldsValue({
        address: config.address,
        port: config.port,
        prefix: config.prefix,
        api_key: config.api_key,
        allow_cors: config.allow_cors,
        log_level: config.log_level,
        autostart_on_startup: config.autostart_on_startup,
      })
    }
  }, [config, form])

  return (
    <Card title={t('apiProxyServer.configuration')}>
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          address: '127.0.0.1',
          port: 8080,
          prefix: '/v1',
          api_key: '',
          allow_cors: false,
          log_level: 'info',
          autostart_on_startup: false,
          ...config,
        }}
        onFinish={handleConfigSave}
        disabled={!canEdit}
      >
        {/* Server Address */}
        <Form.Item
          name="address"
          label={t('apiProxyServer.address')}
          tooltip={t('apiProxyServer.addressTooltip')}
          rules={[
            { required: true, message: t('apiProxyServer.addressRequired') },
          ]}
        >
          <Select
            placeholder={t('apiProxyServer.addressPlaceholder')}
            options={[
              { label: '127.0.0.1 (localhost only)', value: '127.0.0.1' },
              { label: '0.0.0.0 (all interfaces)', value: '0.0.0.0' },
            ]}
          />
        </Form.Item>

        {/* Server Port */}
        <Form.Item
          name="port"
          label={t('apiProxyServer.port')}
          tooltip={t('apiProxyServer.portTooltip')}
          rules={[
            { required: true, message: t('apiProxyServer.portRequired') },
            {
              type: 'number',
              min: 1,
              max: 65535,
              message: t('apiProxyServer.portRange'),
            },
          ]}
        >
          <InputNumber
            placeholder="8080"
            min={1}
            max={65535}
            style={{ width: '100%' }}
          />
        </Form.Item>

        {/* URL Prefix */}
        <Form.Item
          name="prefix"
          label={t('apiProxyServer.prefix')}
          tooltip={t('apiProxyServer.prefixTooltip')}
          rules={[
            { required: true, message: t('apiProxyServer.prefixRequired') },
          ]}
        >
          <Input placeholder="/v1" />
        </Form.Item>

        {/* API Key */}
        <Form.Item
          name="api_key"
          label={t('apiProxyServer.apiKey')}
          tooltip={t('apiProxyServer.apiKeyTooltip')}
        >
          <Input.Password
            placeholder={t('apiProxyServer.apiKeyPlaceholder')}
            prefix={<LockOutlined />}
          />
        </Form.Item>

        {/* CORS Toggle */}
        <div className="flex justify-between items-center mb-4">
          <div style={{ flex: 1, marginRight: 16 }}>
            <Text strong>{t('apiProxyServer.allowCors')}</Text>
            <br />
            <Text type="secondary">{t('apiProxyServer.allowCorsDesc')}</Text>
          </div>
          <Form.Item
            name="allow_cors"
            valuePropName="checked"
            style={{ margin: 0 }}
          >
            <Switch disabled={!canEdit} />
          </Form.Item>
        </div>

        {/* Autostart Toggle */}
        <div className="flex justify-between items-center mb-4">
          <div style={{ flex: 1, marginRight: 16 }}>
            <Text strong>{t('apiProxyServer.autostartOnStartup')}</Text>
            <br />
            <Text type="secondary">
              {t('apiProxyServer.autostartOnStartupDesc')}
            </Text>
          </div>
          <Form.Item
            name="autostart_on_startup"
            valuePropName="checked"
            style={{ margin: 0 }}
          >
            <Switch disabled={!canEdit} />
          </Form.Item>
        </div>

        {/* Log Level */}
        <Form.Item
          name="log_level"
          label={t('apiProxyServer.logLevel')}
          tooltip={t('apiProxyServer.logLevelTooltip')}
        >
          <Select
            placeholder={t('apiProxyServer.logLevelPlaceholder')}
            options={[
              { label: 'Error', value: 'error' },
              { label: 'Warn', value: 'warn' },
              { label: 'Info', value: 'info' },
              { label: 'Debug', value: 'debug' },
              { label: 'Trace', value: 'trace' },
            ]}
          />
        </Form.Item>

        {canEdit && (
          <Form.Item>
            <Button type="primary" htmlType="submit" loading={loadingConfig}>
              {t('common.save')}
            </Button>
          </Form.Item>
        )}
      </Form>
    </Card>
  )
}
