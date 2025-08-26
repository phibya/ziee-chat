import {
  App,
  Button,
  Flex,
  Form,
  Input,
  Switch,
  Tooltip,
  Typography,
} from 'antd'
import {
  CheckOutlined,
  CloseOutlined,
  DeleteOutlined,
  EditOutlined,
} from '@ant-design/icons'
import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { deleteModelProvider, updateModelProvider, Stores } from '../../../../../store'
import { PROVIDER_ICONS } from '../../../../../constants/providers'
import { Provider } from '../../../../../types'

export function ProviderHeader() {
  const [isEditingName, setIsEditingName] = useState(false)
  const [form] = Form.useForm()
  const navigate = useNavigate()
  const { message, modal } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()

  // Get current provider from store
  const currentProvider = Stores.AdminProviders.providers.find(
    p => p.id === providerId,
  )
  const models = currentProvider?.models || []

  useEffect(() => {
    setIsEditingName(false)
  }, [currentProvider?.id])

  useEffect(() => {
    if (isEditingName && currentProvider) {
      form.setFieldsValue({ name: currentProvider.name })
    }
  }, [isEditingName, currentProvider])

  // Helper functions for provider validation
  const canEnableProvider = (provider: Provider): boolean => {
    if (provider.enabled) return true // Already enabled
    const providerModels = provider.id === providerId ? models : []
    if (providerModels.length === 0) return false
    if (provider.type === 'local') return true
    if (!provider.api_key || provider.api_key.trim() === '') return false
    if (!provider.base_url || provider.base_url.trim() === '') return false
    try {
      new globalThis.URL(provider.base_url)
      return true
    } catch {
      return false
    }
  }

  const getEnableDisabledReason = (provider: Provider): string | null => {
    if (provider.enabled) return null
    const providerModels = provider.id === providerId ? models : []
    if (providerModels.length === 0)
      return 'No models available. Add at least one model first.'
    if (provider.type === 'local') return null
    if (!provider.api_key || provider.api_key.trim() === '')
      return 'API key is required'
    if (!provider.base_url || provider.base_url.trim() === '')
      return 'Base URL is required'
    try {
      new globalThis.URL(provider.base_url)
      return null
    } catch {
      return 'Invalid base URL format'
    }
  }

  const handleProviderToggle = async (providerId: string, enabled: boolean) => {
    if (!currentProvider) return

    try {
      await updateModelProvider(providerId, {
        enabled: enabled,
      })
      message.success(
        `${currentProvider?.name || 'Provider'} ${enabled ? 'enabled' : 'disabled'}`,
      )
    } catch (error: any) {
      console.error('Failed to update provider:', error)
      // Handle error similar to original implementation
      if (error.response?.status === 400) {
        if (currentProvider) {
          if (models.length === 0) {
            message.error(
              `Cannot enable "${currentProvider.name}" - No models available`,
            )
          } else if (
            currentProvider.type !== 'local' &&
            (!currentProvider.api_key || currentProvider.api_key.trim() === '')
          ) {
            message.error(
              `Cannot enable "${currentProvider.name}" - API key is required`,
            )
          } else if (
            currentProvider.type !== 'local' &&
            (!currentProvider.base_url ||
              currentProvider.base_url.trim() === '')
          ) {
            message.error(
              `Cannot enable "${currentProvider.name}" - Base URL is required`,
            )
          } else {
            message.error(
              `Cannot enable "${currentProvider.name}" - Invalid base URL format`,
            )
          }
        } else {
          message.error(error?.message || 'Failed to update provider')
        }
      } else {
        message.error(error?.message || 'Failed to update provider')
      }
    }
  }

  const handleDeleteProvider = async () => {
    if (!currentProvider) return

    modal.confirm({
      title: 'Confirm Deletion',
      content: `Are you sure you want to delete "${currentProvider.name}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await deleteModelProvider(currentProvider.id)
          navigate(`/settings/providers`, {
            replace: true,
          })
          message.success('Provider deleted successfully')
        } catch (error: any) {
          console.error('Failed to delete provider:', error)
          // Error is handled by the store
        }
      },
    })
  }

  // Return early if no provider
  if (!currentProvider) {
    return null
  }

  return (
    <Flex justify="space-between" align="center">
      <Flex align="center" gap="middle">
        {(() => {
          const IconComponent = PROVIDER_ICONS[currentProvider.type]
          return <IconComponent className="text-2xl" />
        })()}
        <Form
          style={{
            display: isEditingName ? 'block' : 'none',
          }}
          form={form}
          layout="inline"
          initialValues={{ name: currentProvider.name }}
        >
          <div className={'flex items-center gap-2 w-full flex-wrap'}>
            <Form.Item
              name="name"
              style={{ margin: 0 }}
              rules={[{ required: true, message: 'Name is required' }]}
            >
              <Input className={'!text-lg'} />
            </Form.Item>
            <div className={'flex items-center gap-2'}>
              <Button
                type={'primary'}
                onClick={() => {
                  form.validateFields().then(async values => {
                    await updateModelProvider(currentProvider.id, {
                      name: values.name,
                    })
                    setIsEditingName(false)
                  })
                }}
              >
                <CheckOutlined />
              </Button>
              <Button onClick={() => setIsEditingName(false)}>
                <CloseOutlined />
              </Button>
            </div>
          </div>
        </Form>
        <div
          className={'flex items-center gap-2 overflow-x-hidden w-full'}
          style={{
            display: isEditingName ? 'none' : 'flex',
          }}
        >
          <Typography.Title
            level={4}
            ellipsis
            className={'!m-0 flex-1 overflow-x-hidden'}
          >
            {currentProvider.name}
          </Typography.Title>
          <div className={'flex items-center'}>
            <Button
              type={'text'}
              onClick={() => {
                setIsEditingName(!isEditingName)
              }}
            >
              <EditOutlined />
            </Button>
            <Button type={'text'} danger onClick={handleDeleteProvider}>
              <DeleteOutlined />
            </Button>
          </div>
        </div>
      </Flex>
      {(() => {
        const disabledReason = getEnableDisabledReason(currentProvider)
        const switchElement = (
          <Switch
            checked={currentProvider.enabled}
            disabled={
              !currentProvider.enabled && !canEnableProvider(currentProvider)
            }
            onChange={enabled => handleProviderToggle(currentProvider.id, enabled)}
          />
        )

        if (disabledReason && !currentProvider.enabled) {
          return <Tooltip title={disabledReason}>{switchElement}</Tooltip>
        }
        return switchElement
      })()}
    </Flex>
  )
}
