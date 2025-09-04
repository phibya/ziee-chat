import {
  Button,
  Card,
  Form,
  Select,
  Divider,
  Tag,
  Typography,
  Input,
  Switch,
  App,
} from 'antd'
import React, { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useRAGInstanceStore } from '../../../store/ragInstance'
import { toggleRAGInstanceActivate } from '../../../store/rag'
import {
  useUserProvidersStore,
  loadUserProviders,
  loadUserProvidersWithAllModels,
} from '../../../store/providers'
import { Permission, UpdateRAGInstanceRequest } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { RagSimpleVectorEngineSettings } from './RagSimpleVectorEngineSettings.tsx'
import { RagSimpleGraphEngineSettings } from './RagSimpleGraphEngineSettings.tsx'
import { RagInstanceStatus } from './RagInstanceStatus.tsx'

const { Text } = Typography

export const RagInstanceSettingsTab: React.FC = () => {
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [activationForm] = Form.useForm()
  const [updatingInstance, setUpdatingInstance] = useState(false)
  const engineType = Form.useWatch('engine_type', form)

  useEffect(() => {
    loadUserProvidersWithAllModels()
  }, [])

  // RAG instance store
  const { ragInstance, updateRAGInstance } = useRAGInstanceStore(ragInstanceId)

  // Providers store for models
  const {
    providers,
    modelsByProvider,
    isInitialized: providersInitialized,
  } = useUserProvidersStore()

  // Initialize providers
  useEffect(() => {
    if (!providersInitialized) {
      loadUserProviders().catch(console.error)
    }
  }, [providersInitialized])

  // Initialize form with RAG instance data
  useEffect(() => {
    if (ragInstance) {
      form.setFieldsValue({
        enabled: ragInstance.enabled,
        engine_type: ragInstance.engine_type,
        embedding_model_id: ragInstance.embedding_model_id,
        llm_model_id: ragInstance.llm_model_id,
        engine_settings: ragInstance.engine_settings,
      })

      // Initialize activation form separately
      activationForm.setFieldsValue({
        is_active: ragInstance.is_active,
      })
    }
  }, [ragInstance, form, activationForm])

  useEffect(() => {
    // Sync activation form if ragInstance.is_active changes externally
    if (ragInstance) {
      activationForm.setFieldsValue({
        is_active: ragInstance.is_active,
      })
    }
  }, [ragInstance?.is_active])

  // Handle configuration form submission
  const handleConfigurationSubmit = async (values: any) => {
    if (!ragInstance) return

    try {
      setUpdatingInstance(true)

      const updateData: UpdateRAGInstanceRequest = {
        enabled:
          values.enabled !== ragInstance.enabled ? values.enabled : undefined,
        embedding_model_id:
          values.embedding_model_id !== ragInstance.embedding_model_id
            ? values.embedding_model_id
            : undefined,
        llm_model_id:
          values.llm_model_id !== ragInstance.llm_model_id
            ? values.llm_model_id
            : undefined,
        engine_settings: values.engine_settings,
      }

      // Remove undefined values
      Object.keys(updateData).forEach(key => {
        if (updateData[key as keyof UpdateRAGInstanceRequest] === undefined) {
          delete updateData[key as keyof UpdateRAGInstanceRequest]
        }
      })

      await updateRAGInstance(updateData)
    } catch (error) {
      console.error('Failed to update RAG instance:', error)
    } finally {
      setUpdatingInstance(false)
    }
  }

  // Handle activation toggle
  const handleActivationToggle = async (checked: boolean) => {
    if (!ragInstance || !ragInstanceId) return

    const previousValue = ragInstance.is_active

    try {
      // Update activation form immediately for UI feedback
      activationForm.setFieldValue('is_active', checked)

      await toggleRAGInstanceActivate(ragInstanceId)

      message.success(
        `RAG instance ${previousValue ? 'deactivated' : 'activated'} successfully`,
      )
    } catch (error) {
      console.error('Failed to toggle activation:', error)
      message.error('Failed to toggle activation status')
      // Revert the activation form value on error
      activationForm.setFieldValue('is_active', previousValue)
    }
  }

  // Get available models grouped by provider, filtered by capability
  const getAvailableModels = (capability?: 'text_embedding' | 'chat') => {
    const options: Array<{
      label: string
      options: Array<{
        label: string
        value: string
        description?: string
      }>
    }> = []

    providers.forEach(provider => {
      const providerModels = modelsByProvider[provider.id] || []

      // Filter models by capability if specified
      const filteredModels = capability
        ? providerModels.filter(model => model.capabilities?.[capability])
        : providerModels

      if (filteredModels.length > 0) {
        options.push({
          label: provider.name,
          options: filteredModels.map(model => ({
            label: model.alias || model.name,
            value: model.id,
            description: model.description || '',
          })),
        })
      }
    })

    return options
  }

  return (
    <div className="flex flex-col gap-3">
      {/* Real-time Status Card */}
      <RagInstanceStatus />

      {/* Instance Information Card */}
      <Card
        title={
          <Typography.Title level={5} className={'!m-0 !pt-[2px]'}>
            Instance Configuration
          </Typography.Title>
        }
      >
        <div className="flex flex-col gap-2">
          <div className={'flex items-center justify-between'}>
            <Text type="secondary">Active:</Text>
            <PermissionGuard
              permissions={[Permission.RagInstancesEdit]}
              type="disabled"
            >
              <Form form={activationForm}>
                <Form.Item name="is_active" className="mb-0">
                  <Switch onChange={handleActivationToggle} />
                </Form.Item>
              </Form>
            </PermissionGuard>
          </div>
          {ragInstance?.embedding_model_id && (
            <div className={'flex items-center justify-between'}>
              <Text type="secondary">Embedding Model:</Text>
              <Text style={{ fontSize: '12px' }}>
                {ragInstance.embedding_model_id.substring(0, 20)}...
              </Text>
            </div>
          )}
          {ragInstance?.llm_model_id && (
            <div className={'flex items-center justify-between'}>
              <Text type="secondary">LLM Model:</Text>
              <Text style={{ fontSize: '12px' }}>
                {ragInstance.llm_model_id.substring(0, 20)}...
              </Text>
            </div>
          )}
        </div>

        {/* RAG Configuration Form */}
        <div className="mt-4">
          <PermissionGuard
            permissions={[Permission.RagInstancesEdit]}
            type="disabled"
          >
            <Form
              form={form}
              layout="vertical"
              onFinish={handleConfigurationSubmit}
              disabled={updatingInstance}
            >
            <Form.Item
              label="Engine Type"
              name="engine_type"
              rules={[{ required: true, message: 'Please select engine type' }]}
            >
              <Select placeholder="Select engine type">
                <Select.Option value="simple_vector">
                  Simple Vector Search
                </Select.Option>
                <Select.Option value="simple_graph">
                  Simple Graph Search
                </Select.Option>
              </Select>
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Model Configuration</Text>
            </Divider>

            <Form.Item label="Embedding Model" name="embedding_model_id">
              <Select
                placeholder="Select embedding model"
                allowClear
                showSearch
                filterOption={(input, option) => {
                  if (!option) return false
                  if ('options' in option && Array.isArray(option.options)) {
                    // This is a group option - search in children
                    return option.options.some((child: any) =>
                      child?.label?.toLowerCase().includes(input.toLowerCase()),
                    )
                  }
                  // This is a regular option
                  return (option.label ?? '')
                    .toLowerCase()
                    .includes(input.toLowerCase())
                }}
                options={getAvailableModels('text_embedding')}
              />
            </Form.Item>

            <Form.Item label="LLM Model" name="llm_model_id">
              <Select
                placeholder="Select LLM model"
                allowClear
                showSearch
                filterOption={(input, option) => {
                  if (!option) return false
                  if ('options' in option && Array.isArray(option.options)) {
                    // This is a group option - search in children
                    return option.options.some((child: any) =>
                      child?.label?.toLowerCase().includes(input.toLowerCase()),
                    )
                  }
                  // This is a regular option
                  return (option.label ?? '')
                    .toLowerCase()
                    .includes(input.toLowerCase())
                }}
                options={getAvailableModels('chat')}
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Engine Settings</Text>
            </Divider>

            <div
              style={{
                display: engineType === 'simple_vector' ? 'block' : 'none',
              }}
            >
              <RagSimpleVectorEngineSettings />
            </div>

            <div
              style={{
                display: engineType === 'simple_graph' ? 'block' : 'none',
              }}
            >
              <RagSimpleGraphEngineSettings />
            </div>

            <Form.Item className="mb-0 !mt-3">
              <Button
                type="primary"
                htmlType="submit"
                loading={updatingInstance}
              >
                Save
              </Button>
            </Form.Item>
          </Form>
          </PermissionGuard>
        </div>
      </Card>
    </div>
  )
}
