import { App, Button, Card, Flex, Form, Input, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import {
  clearRAGProvidersError,
  Stores,
  updateRAGProvider,
} from '../../../../store'
import { RAGProviderHeader } from './RAGProviderHeader'
import { SystemInstancesSection } from './SystemInstancesSection'

const { Title, Text } = Typography

export function RAGProviderSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()

  const [form] = Form.useForm()
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  const [pendingSettings, setPendingSettings] = useState<any>(null)

  // Store data
  const { error } = Stores.AdminRAGProviders

  // Get current provider
  const currentProvider = Stores.AdminRAGProviders.providers.find(
    p => p.id === providerId,
  )


  const handleFormChange = (changedValues: any) => {
    if (!currentProvider) return

    setHasUnsavedChanges(true)
    setPendingSettings((prev: any) => ({ ...prev, ...changedValues }))
  }


  const handleSaveSettings = async () => {
    if (!currentProvider || !pendingSettings) return

    try {
      await updateRAGProvider(currentProvider.id, pendingSettings)

      setHasUnsavedChanges(false)
      setPendingSettings(null)
      message.success(t('providers.settingsSaved'))
    } catch (error) {
      console.error('Failed to save settings:', error)
      // Error is handled by the store
    }
  }


  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearRAGProvidersError()
    }
  }, [error])

  // Update forms when provider changes
  useEffect(() => {
    if (currentProvider) {
      form.setFieldsValue({
        api_key: currentProvider.api_key,
        base_url: currentProvider.base_url,
      })
      // Clear unsaved changes when switching providers
      setHasUnsavedChanges(false)
      setPendingSettings(null)
    }
  }, [currentProvider])

  // Return early if no provider
  if (!currentProvider) {
    return null
  }

  return (
    <Flex className={'flex-col gap-3 w-full overflow-x-hidden'}>
      <RAGProviderHeader />

      {/* API Configuration - Only for non-local providers */}
      {currentProvider.type !== 'local' && (
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
                  The {currentProvider.name} API uses API keys for
                  authentication. Visit your <Text type="danger">API Keys</Text>{' '}
                  page to retrieve the API key you'll use in your requests.
                </Text>
                <Form.Item
                  name="api_key"
                  style={{ marginBottom: 0, marginTop: 16 }}
                >
                  <Input.Password placeholder={t('providers.insertApiKey')} />
                </Form.Item>
              </div>

              <div>
                <Title level={5}>Base URL</Title>
                <Text type="secondary">
                  The base endpoint to use. See the{' '}
                  <Text type="danger">
                    {currentProvider.name} documentation
                  </Text>{' '}
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
      )}

      {/* Instances Section */}
      <SystemInstancesSection />
    </Flex>
  )
}
