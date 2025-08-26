import {
  CopyOutlined,
  EyeInvisibleOutlined,
  EyeTwoTone,
} from '@ant-design/icons'
import { App, Button, Card, Flex, Form, Input, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import {
  clearProvidersError,
  Stores,
  updateModelProvider,
} from '../../../../store'
import { ProviderProxySettingsForm } from './ProviderProxySettings'
import { ModelsSection } from './common/ModelsSection'
import { ProviderHeader } from './common/ProviderHeader'
import { ProxySettings } from '../../../../types'

const { Title, Text } = Typography

export function RemoteProviderSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()

  const [form] = Form.useForm()
  const [nameForm] = Form.useForm()
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  const [pendingSettings, setPendingSettings] = useState<any>(null)

  // Store data
  const { error } = Stores.AdminProviders

  // Get current provider and its models
  const currentProvider = Stores.AdminProviders.providers.find(
    p => p.id === providerId,
  )

  const copyToClipboard = (text: string) => {
    if (typeof window !== 'undefined' && window.navigator?.clipboard) {
      window.navigator.clipboard.writeText(text)
      message.success(t('providers.copiedToClipboard'))
    } else {
      message.error(t('providers.clipboardNotAvailable'))
    }
  }

  const handleFormChange = (changedValues: any) => {
    if (!currentProvider) return

    setHasUnsavedChanges(true)
    setPendingSettings((prev: any) => ({ ...prev, ...changedValues }))
  }



  const handleSaveSettings = async () => {
    if (!currentProvider || !pendingSettings) return

    try {
      await updateModelProvider(currentProvider.id, pendingSettings)

      setHasUnsavedChanges(false)
      setPendingSettings(null)
      message.success(t('providers.settingsSaved'))
    } catch (error) {
      console.error('Failed to save settings:', error)
      // Error is handled by the store
    }
  }

  const handleProxySettingsSave = async (proxySettings: any) => {
    if (!currentProvider) return

    try {
      await updateModelProvider(currentProvider.id, {
        proxy_settings: proxySettings,
      })
      message.success(t('providers.proxySettingsSaved'))
    } catch (error) {
      console.error('Failed to save proxy settings:', error)
      // Error is handled by the store
    }
  }


  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProvidersError()
    }
  }, [error]) // Removed message from dependencies to prevent infinite rerenders

  // Update forms when provider changes
  useEffect(() => {
    if (currentProvider) {
      form.setFieldsValue({
        api_key: currentProvider.api_key,
        base_url: currentProvider.base_url,
      })
      nameForm.setFieldsValue({
        name: currentProvider.name,
      })
      // Clear unsaved changes when switching providers
      setHasUnsavedChanges(false)
      setPendingSettings(null)
    }
  }, [currentProvider, form, nameForm])

  // Return early if no provider or not remote
  if (!currentProvider || currentProvider.type === 'local') {
    return null
  }

  return (
    <Flex className={'flex-col gap-3'}>
      <ProviderHeader />

      {/* API Configuration */}
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          api_key: currentProvider.api_key,
          base_url: currentProvider.base_url,
        }}
        onValuesChange={handleFormChange}
      >
        <Card
          title={t('providers.apiConfiguration')}
          extra={
            <Button
              type="primary"
              onClick={handleSaveSettings}
              disabled={!hasUnsavedChanges}
            >
              Save
            </Button>
          }
        >
          <Flex className={'flex-col gap-3'}>
            <div>
              <Title level={5}>API Key</Title>
              <Text type="secondary">
                The {currentProvider.name} API uses API keys for authentication.
                Visit your <Text type="danger">API Keys</Text> page to retrieve
                the API key you'll use in your requests.
              </Text>
              <Form.Item
                name="api_key"
                style={{ marginBottom: 0, marginTop: 16 }}
              >
                <Input.Password
                  placeholder={t('providers.insertApiKey')}
                  iconRender={visible =>
                    visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                  }
                  suffix={
                    <Button
                      type="text"
                      icon={<CopyOutlined />}
                      onClick={() =>
                        copyToClipboard(currentProvider.api_key || '')
                      }
                    />
                  }
                />
              </Form.Item>
            </div>

            <div>
              <Title level={5}>Base URL</Title>
              <Text type="secondary">
                The base{' '}
                {currentProvider.type === 'gemini' ? 'OpenAI-compatible' : ''}{' '}
                endpoint to use. See the{' '}
                <Text type="danger">{currentProvider.name} documentation</Text>{' '}
                for more information.
              </Text>
              <Form.Item
                name="base_url"
                style={{ marginBottom: 0, marginTop: 16 }}
              >
                <Input placeholder={t('providers.baseUrl')} />
              </Form.Item>
            </div>
          </Flex>
        </Card>
      </Form>

      {/* Models Section */}
      <ModelsSection />

      {/* Proxy Settings - For non-Local providers */}
      <ProviderProxySettingsForm
        initialSettings={
          currentProvider.proxy_settings || ({} as ProxySettings)
        }
        onSave={handleProxySettingsSave}
      />
    </Flex>
  )
}
