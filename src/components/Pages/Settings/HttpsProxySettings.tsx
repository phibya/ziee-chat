import {
  Button,
  Card,
  Col,
  Flex,
  Form,
  Input,
  message,
  Row,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useShallow } from 'zustand/react/shallow'
import { useAdminStore } from '../../../store/admin'

const { Title, Text } = Typography

// interface ProxySettings {
//   enabled: boolean
//   url: string
//   username: string
//   password: string
//   no_proxy: string
//   ignore_ssl_certificates: boolean
//   proxy_ssl: boolean
//   proxy_host_ssl: boolean
//   peer_ssl: boolean
//   host_ssl: boolean
// }

export function HttpsProxySettings() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [lastTestedConfig, setLastTestedConfig] = useState<string | null>(null)
  const [isProxyTested, setIsProxyTested] = useState(false)

  // Admin store
  const {
    proxySettings,
    loading,
    testingProxy,
    error,
    loadProxySettings,
    updateProxySettings,
    testProxyConnection,
    clearError,
  } = useAdminStore(
    useShallow(state => ({
      proxySettings: state.proxySettings,
      loading: state.loading,
      testingProxy: state.testingProxy,
      updating: state.updating,
      error: state.error,
      loadProxySettings: state.loadProxySettings,
      updateProxySettings: state.updateProxySettings,
      testProxyConnection: state.testProxyConnection,
      clearError: state.clearError,
    })),
  )

  useEffect(() => {
    loadProxySettings()
  }, [loadProxySettings])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, clearError])

  // Update form when proxy settings change
  useEffect(() => {
    if (proxySettings) {
      form.setFieldsValue(proxySettings)
      // Reset test status when loading settings
      setIsProxyTested(false)
      setLastTestedConfig(null)
    }
  }, [proxySettings, form])

  const handleSave = async () => {
    try {
      const values = await form.validateFields()

      // Check if proxy settings have changed and if enabling without testing
      const currentConfig = JSON.stringify({
        url: values.url,
        username: values.username,
        password: values.password,
        no_proxy: values.no_proxy,
        ignore_ssl_certificates: values.ignore_ssl_certificates,
        proxy_ssl: values.proxy_ssl,
        proxy_host_ssl: values.proxy_host_ssl,
        peer_ssl: values.peer_ssl,
        host_ssl: values.host_ssl,
      })

      // If proxy is being enabled but hasn't been tested, or config changed since last test
      if (
        values.enabled &&
        (!isProxyTested || currentConfig !== lastTestedConfig)
      ) {
        // Force disable proxy and inform user
        values.enabled = false
        message.warning(t('proxy.disabledTestRequired'))
      }

      await updateProxySettings(values)
      form.setFieldsValue(values) // Update form with potentially modified enabled state

      // Reset test status if config changed
      if (currentConfig !== lastTestedConfig) {
        setIsProxyTested(false)
        setLastTestedConfig(null)
      }

      message.success(t('proxy.settingsSaved'))
    } catch (error) {
      console.error('Failed to save proxy settings:', error)
      // Error is handled by the store
    }
  }

  const handleReset = () => {
    if (proxySettings) {
      form.setFieldsValue(proxySettings)
    }
  }

  const handleTestProxy = async () => {
    try {
      const values = form.getFieldsValue()

      // Only test if URL is provided (no need to be enabled)
      if (!values.url || values.url.trim() === '') {
        message.warning(t('proxy.enterValidUrl'))
        return
      }

      const success = await testProxyConnection()

      if (success) {
        // Store the tested configuration
        const currentConfig = JSON.stringify({
          url: values.url,
          username: values.username,
          password: values.password,
          no_proxy: values.no_proxy,
          ignore_ssl_certificates: values.ignore_ssl_certificates,
          proxy_ssl: values.proxy_ssl,
          proxy_host_ssl: values.proxy_host_ssl,
          peer_ssl: values.peer_ssl,
          host_ssl: values.host_ssl,
        })

        setLastTestedConfig(currentConfig)
        setIsProxyTested(true)
        message.success(t('proxy.testSuccessful'))
      } else {
        setIsProxyTested(false)
        setLastTestedConfig(null)
        message.error(t('proxy.testFailed'))
      }
    } catch (error) {
      console.error('Proxy test failed:', error)
      setIsProxyTested(false)
      setLastTestedConfig(null)
      // Error is handled by the store
    }
  }

  const isProxyValid = (values: any): boolean => {
    if (!values || typeof values !== 'object') {
      return false
    }

    if (
      !values.url ||
      typeof values.url !== 'string' ||
      values.url.trim() === ''
    ) {
      return false
    }

    try {
      // eslint-disable-next-line no-undef
      new URL(values.url)
      return true
    } catch {
      return false
    }
  }

  const canEnableProxy = (values: any) => {
    const currentConfig = JSON.stringify({
      url: values.url,
      username: values.username,
      password: values.password,
      no_proxy: values.no_proxy,
      ignore_ssl_certificates: values.ignore_ssl_certificates,
      proxy_ssl: values.proxy_ssl,
      proxy_host_ssl: values.proxy_host_ssl,
      peer_ssl: values.peer_ssl,
      host_ssl: values.host_ssl,
    })

    return isProxyTested && currentConfig === lastTestedConfig
  }

  if (loading) {
    return (
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <Title level={3}>{t('proxy.title')}</Title>
        <Card>
          <Text type="secondary">{t('proxy.loadingSettings')}</Text>
        </Card>
      </Space>
    )
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>{t('proxy.title')}</Title>
      <Card>
        <Form form={form} layout="vertical" onFinish={handleSave}>
          <Form.Item
            noStyle
            shouldUpdate={(prevValues, currentValues) =>
              prevValues.enabled !== currentValues.enabled ||
              prevValues.url !== currentValues.url ||
              prevValues.username !== currentValues.username ||
              prevValues.password !== currentValues.password ||
              prevValues.no_proxy !== currentValues.no_proxy ||
              prevValues.ignore_ssl_certificates !==
                currentValues.ignore_ssl_certificates ||
              prevValues.proxy_ssl !== currentValues.proxy_ssl ||
              prevValues.proxy_host_ssl !== currentValues.proxy_host_ssl ||
              prevValues.peer_ssl !== currentValues.peer_ssl ||
              prevValues.host_ssl !== currentValues.host_ssl
            }
          >
            {({ getFieldsValue }) => {
              const values = getFieldsValue()
              const canEnable = canEnableProxy(values)

              return (
                <Flex className={'w-full'}>
                  <div className={'flex-1'}>
                    <Title level={5} style={{ margin: 0 }}>
                      {t('proxy.proxy')}
                    </Title>
                    {values.enabled && !canEnable && (
                      <Text
                        type="warning"
                        style={{
                          fontSize: '12px',
                          display: 'block',
                          marginTop: '4px',
                        }}
                      >
                        {t('proxy.proxyMustBeTested')}
                      </Text>
                    )}
                  </div>
                  <Form.Item
                    name="enabled"
                    valuePropName="checked"
                    style={{ margin: 0 }}
                  >
                    <Switch />
                  </Form.Item>
                </Flex>
              )
            }}
          </Form.Item>

          <Space
            direction="vertical"
            size="large"
            style={{ width: '100%', marginTop: 24 }}
          >
            <div>
              <Title level={5}>{t('proxy.proxyUrl')}</Title>
              <Text type="secondary">{t('proxy.proxyUrlDesc')}</Text>
              <Form.Item
                noStyle
                shouldUpdate={(prevValues, currentValues) =>
                  prevValues.enabled !== currentValues.enabled
                }
              >
                {({ getFieldValue }) => {
                  const enabled = getFieldValue('enabled')
                  return (
                    <Form.Item
                      name="url"
                      rules={
                        enabled
                          ? [
                              {
                                required: true,
                                message: t('proxy.enterProxyUrl'),
                              },
                            ]
                          : []
                      }
                      style={{ marginTop: 8 }}
                    >
                      <Input placeholder={t('proxy.proxyUrlPlaceholder')} />
                    </Form.Item>
                  )
                }}
              </Form.Item>
            </div>

            <div>
              <Title level={5}>{t('proxy.authentication')}</Title>
              <Text type="secondary">{t('proxy.authDesc')}</Text>
              <Row gutter={8} style={{ marginTop: 8 }}>
                <Col span={12}>
                  <Form.Item name="username">
                    <Input placeholder={t('proxy.usernamePlaceholder')} />
                  </Form.Item>
                </Col>
                <Col span={12}>
                  <Form.Item name="password">
                    <Input.Password
                      placeholder={t('proxy.passwordPlaceholder')}
                    />
                  </Form.Item>
                </Col>
              </Row>
            </div>

            <div>
              <Title level={5}>{t('proxy.noProxy')}</Title>
              <Text type="secondary">{t('proxy.noProxyDesc')}</Text>
              <Form.Item name="no_proxy" style={{ marginTop: 8 }}>
                <Input placeholder={t('proxy.noProxyPlaceholder')} />
              </Form.Item>
            </div>

            <div>
              <Title level={4} style={{ marginTop: 32, marginBottom: 16 }}>
                {t('proxy.sslVerification')}
              </Title>

              <div>
                <div style={{ marginBottom: 24 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    {t('proxy.ignoreSslCerts')}
                  </Title>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'flex-start',
                    }}
                  >
                    <Text type="secondary" style={{ flex: 1, marginRight: 16 }}>
                      {t('proxy.ignoreSslCertsDesc')}
                    </Text>
                    <Form.Item
                      name="ignore_ssl_certificates"
                      valuePropName="checked"
                      style={{ margin: 0 }}
                    >
                      <Switch />
                    </Form.Item>
                  </div>
                </div>

                <div style={{ marginBottom: 24 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    {t('proxy.proxySsl')}
                  </Title>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                    }}
                  >
                    <Text type="secondary" style={{ flex: 1, marginRight: 16 }}>
                      {t('proxy.proxySslDesc')}
                    </Text>
                    <Form.Item
                      name="proxy_ssl"
                      valuePropName="checked"
                      style={{ margin: 0 }}
                    >
                      <Switch />
                    </Form.Item>
                  </div>
                </div>

                <div style={{ marginBottom: 24 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    {t('proxy.proxyHostSsl')}
                  </Title>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                    }}
                  >
                    <Text type="secondary" style={{ flex: 1, marginRight: 16 }}>
                      {t('proxy.proxyHostSslDesc')}
                    </Text>
                    <Form.Item
                      name="proxy_host_ssl"
                      valuePropName="checked"
                      style={{ margin: 0 }}
                    >
                      <Switch />
                    </Form.Item>
                  </div>
                </div>

                <div style={{ marginBottom: 24 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    {t('proxy.peerSsl')}
                  </Title>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                    }}
                  >
                    <Text type="secondary" style={{ flex: 1, marginRight: 16 }}>
                      {t('proxy.peerSslDesc')}
                    </Text>
                    <Form.Item
                      name="peer_ssl"
                      valuePropName="checked"
                      style={{ margin: 0 }}
                    >
                      <Switch />
                    </Form.Item>
                  </div>
                </div>

                <div>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    {t('proxy.hostSsl')}
                  </Title>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                    }}
                  >
                    <Text type="secondary" style={{ flex: 1, marginRight: 16 }}>
                      {t('proxy.hostSslDesc')}
                    </Text>
                    <Form.Item
                      name="host_ssl"
                      valuePropName="checked"
                      style={{ margin: 0 }}
                    >
                      <Switch />
                    </Form.Item>
                  </div>
                </div>
              </div>
            </div>
          </Space>

          <div style={{ marginTop: 24, textAlign: 'right' }}>
            <Space>
              <Button onClick={handleReset}>{t('proxy.reset')}</Button>
              <Form.Item
                noStyle
                shouldUpdate={(prevValues, currentValues) =>
                  prevValues.enabled !== currentValues.enabled ||
                  prevValues.url !== currentValues.url
                }
              >
                {({ getFieldsValue }) => {
                  const values = getFieldsValue()
                  const canTest = isProxyValid(values)
                  const canSave = isProxyValid(values)
                  const enabledButNotTested =
                    values.enabled && !canEnableProxy(values)

                  return (
                    <Space>
                      <Button
                        onClick={handleTestProxy}
                        loading={testingProxy}
                        disabled={!canTest}
                      >
                        {t('proxy.testProxy')}
                      </Button>
                      <Button
                        type="primary"
                        htmlType="submit"
                        loading={loading}
                        disabled={!canSave}
                      >
                        {t('common.save')}
                      </Button>
                      {enabledButNotTested && (
                        <Text type="warning" style={{ fontSize: '12px' }}>
                          {t('proxy.willBeDisabled')}
                        </Text>
                      )}
                    </Space>
                  )
                }}
              </Form.Item>
            </Space>
          </div>
        </Form>
      </Card>
    </Space>
  )
}
