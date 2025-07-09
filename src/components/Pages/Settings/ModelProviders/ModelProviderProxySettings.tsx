import {
  Button,
  Card,
  Col,
  Divider,
  Form,
  Input,
  Row,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { ModelProviderProxySettings } from '../../../../types/api/modelProvider'
import { ApiClient } from '../../../../api/client'

const { Title, Text } = Typography

interface ModelProviderProxySettingsProps {
  providerId: string
  initialSettings: ModelProviderProxySettings
  onSave: (settings: ModelProviderProxySettings) => void
  loading?: boolean
  disabled?: boolean
}

export function ModelProviderProxySettingsForm({
  providerId,
  initialSettings,
  onSave,
  loading = false,
  disabled = false,
}: ModelProviderProxySettingsProps) {
  const [form] = Form.useForm()
  const [testingProxy, setTestingProxy] = useState(false)
  const [lastTestedConfig, setLastTestedConfig] = useState<string | null>(null)
  const [isProxyTested, setIsProxyTested] = useState(false)

  useEffect(() => {
    form.setFieldsValue(initialSettings)
    // Reset test status when settings change
    setIsProxyTested(false)
    setLastTestedConfig(null)
  }, [initialSettings, form])

  const handleTestProxy = async () => {
    try {
      setTestingProxy(true)
      const values = form.getFieldsValue()

      // Only test if URL is provided
      if (!values.url || values.url.trim() === '') {
        return
      }

      await ApiClient.ModelProviders.testProxy({
        provider_id: providerId,
        enabled: values.enabled,
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
    } catch (error) {
      console.error('Proxy test failed:', error)
      setIsProxyTested(false)
      setLastTestedConfig(null)
      throw error
    } finally {
      setTestingProxy(false)
    }
  }

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
        values.url &&
        values.url.trim() !== '' &&
        (!isProxyTested || currentConfig !== lastTestedConfig)
      ) {
        // Force disable proxy and inform user
        values.enabled = false
        form.setFieldValue('enabled', false)
      }

      onSave(values)

      // Reset test status if config changed
      if (currentConfig !== lastTestedConfig) {
        setIsProxyTested(false)
        setLastTestedConfig(null)
      }
    } catch (error) {
      console.error('Failed to save proxy settings:', error)
      throw error
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
    if (!values.url || values.url.trim() === '') return true // Can enable/disable if no URL

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

  return (
    <Card title="Proxy Settings">
      <Form form={form} layout="vertical">
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
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'flex-start',
                  marginBottom: 24,
                }}
              >
                <div style={{ flex: 1 }}>
                  <Title level={5} style={{ margin: 0 }}>
                    Enable Proxy
                  </Title>
                  <Text type="secondary">
                    Use a proxy server for this model provider's API calls
                  </Text>
                  {values.enabled && values.url && !canEnable && (
                    <Text
                      type="warning"
                      style={{
                        fontSize: '12px',
                        display: 'block',
                        marginTop: '4px',
                      }}
                    >
                      ⚠️ Proxy must be tested before enabling
                    </Text>
                  )}
                </div>
                <Form.Item
                  name="enabled"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            )
          }}
        </Form.Item>

        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          <div>
            <Title level={5}>Proxy URL</Title>
            <Text type="secondary">The URL and port of your proxy server.</Text>
            <Form.Item
              name="url"
              style={{ marginTop: 8 }}
              rules={[
                ({ getFieldValue }) => ({
                  validator(_, value) {
                    if (!getFieldValue('enabled')) {
                      return Promise.resolve()
                    }
                    if (!value || value.trim() === '') {
                      return Promise.reject(new Error('Please enter proxy URL'))
                    }
                    try {
                      // eslint-disable-next-line no-undef
                      new URL(value)
                      return Promise.resolve()
                    } catch {
                      return Promise.reject(new Error('Invalid URL format'))
                    }
                  },
                }),
              ]}
            >
              <Input
                placeholder="http://<user>:<password>@<domain or IP>:<port>"
                disabled={disabled}
              />
            </Form.Item>
          </div>

          <div>
            <Title level={5}>Authentication</Title>
            <Text type="secondary">
              Credentials for the proxy server, if required.
            </Text>
            <Row gutter={8} style={{ marginTop: 8 }}>
              <Col span={12}>
                <Form.Item name="username">
                  <Input placeholder="Username" disabled={disabled} />
                </Form.Item>
              </Col>
              <Col span={12}>
                <Form.Item name="password">
                  <Input.Password placeholder="Password" disabled={disabled} />
                </Form.Item>
              </Col>
            </Row>
          </div>

          <div>
            <Title level={5}>No Proxy</Title>
            <Text type="secondary">
              A comma-separated list of hosts to bypass the proxy.
            </Text>
            <Form.Item name="no_proxy" style={{ marginTop: 8 }}>
              <Input placeholder="localhost, 127.0.0.1" disabled={disabled} />
            </Form.Item>
          </div>

          <div>
            <Title level={5} style={{ marginBottom: 16 }}>
              SSL Verification
            </Title>

            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'flex-start',
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    Ignore SSL Certificates
                  </Title>
                  <Text type="secondary">
                    Allow self-signed or unverified certificates. This may be
                    required for some proxies but reduces security.
                  </Text>
                </div>
                <Form.Item
                  name="ignore_ssl_certificates"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    Proxy SSL
                  </Title>
                  <Text type="secondary">
                    Validate the SSL certificate when connecting to the proxy.
                  </Text>
                </div>
                <Form.Item
                  name="proxy_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    Proxy Host SSL
                  </Title>
                  <Text type="secondary">
                    Validate the SSL certificate of the proxy's host.
                  </Text>
                </div>
                <Form.Item
                  name="proxy_host_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            <div style={{ marginBottom: 24 }}>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    Peer SSL
                  </Title>
                  <Text type="secondary">
                    Validate the SSL certificates of peer connections.
                  </Text>
                </div>
                <Form.Item
                  name="peer_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>

            <div>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div style={{ flex: 1, marginRight: 16 }}>
                  <Title level={5} style={{ margin: 0, marginBottom: 4 }}>
                    Host SSL
                  </Title>
                  <Text type="secondary">
                    Validate the SSL certificates of destination hosts.
                  </Text>
                </div>
                <Form.Item
                  name="host_ssl"
                  valuePropName="checked"
                  style={{ margin: 0 }}
                >
                  <Switch disabled={disabled} />
                </Form.Item>
              </div>
            </div>
          </div>
        </Space>

        <Divider />

        <div style={{ textAlign: 'right' }}>
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
              const enabledButNotTested =
                values.enabled && values.url && !canEnableProxy(values)

              return (
                <Space>
                  <Button
                    onClick={handleTestProxy}
                    loading={testingProxy}
                    disabled={!canTest || disabled}
                  >
                    Test Proxy
                  </Button>
                  <Button
                    type="primary"
                    onClick={handleSave}
                    loading={loading}
                    disabled={disabled}
                  >
                    Save
                  </Button>
                  {enabledButNotTested && (
                    <Text type="warning" style={{ fontSize: '12px' }}>
                      ⚠️ Proxy will be disabled on save - test required
                    </Text>
                  )}
                </Space>
              )
            }}
          </Form.Item>
        </div>
      </Form>
    </Card>
  )
}
