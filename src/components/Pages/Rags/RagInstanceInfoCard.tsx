import {
  Button,
  Card,
  Flex,
  Form,
  Select,
  Divider,
  InputNumber,
  Tag,
  Typography,
  Input,
  Space,
} from 'antd'
import { SettingOutlined } from '@ant-design/icons'
import React, { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useRAGInstanceStore } from '../../../store/ragInstance'
import { useUserProvidersStore, loadUserProviders } from '../../../store/providers'
import { Permission, UpdateRAGInstanceRequest } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'

const { Text } = Typography

export const RagInstanceInfoCard: React.FC = () => {
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const [form] = Form.useForm()
  const [configurationVisible, setConfigurationVisible] = useState(false)
  const [updatingInstance, setUpdatingInstance] = useState(false)

  // RAG instance store
  const {
    ragInstance,
    updateRAGInstance,
  } = useRAGInstanceStore(ragInstanceId)

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
        name: ragInstance.name,
        description: ragInstance.description,
        enabled: ragInstance.enabled,
        engine_type: ragInstance.engine_type,
        embedding_model_id: ragInstance.embedding_model_id,
        llm_model_id: ragInstance.llm_model_id,
        engine_settings: ragInstance.engine_settings,
      })
    }
  }, [ragInstance, form])

  // Handle configuration form submission
  const handleConfigurationSubmit = async (values: any) => {
    if (!ragInstance) return

    try {
      setUpdatingInstance(true)
      
      const updateData: UpdateRAGInstanceRequest = {
        name: values.name !== ragInstance.name ? values.name : undefined,
        description: values.description !== ragInstance.description ? values.description : undefined,
        enabled: values.enabled !== ragInstance.enabled ? values.enabled : undefined,
        embedding_model_id: values.embedding_model_id !== ragInstance.embedding_model_id ? values.embedding_model_id : undefined,
        llm_model_id: values.llm_model_id !== ragInstance.llm_model_id ? values.llm_model_id : undefined,
        engine_settings: values.engine_settings,
      }

      // Remove undefined values
      Object.keys(updateData).forEach(key => {
        if (updateData[key as keyof UpdateRAGInstanceRequest] === undefined) {
          delete updateData[key as keyof UpdateRAGInstanceRequest]
        }
      })

      await updateRAGInstance(updateData)
      setConfigurationVisible(false)
    } catch (error) {
      console.error('Failed to update RAG instance:', error)
    } finally {
      setUpdatingInstance(false)
    }
  }

  // Get available models for model selectors
  const getAvailableModels = () => {
    const allModels: Array<{id: string, name: string, providerId: string}> = []
    
    providers.forEach(provider => {
      const providerModels = modelsByProvider[provider.id] || []
      providerModels.forEach(model => {
        allModels.push({
          id: model.id,
          name: `${provider.name} - ${model.name}`,
          providerId: provider.id
        })
      })
    })
    
    return allModels
  }

  return (
    <Card
      title={
        <Flex className="items-center justify-between">
          <Text strong>Instance Information</Text>
          <PermissionGuard permissions={[Permission.RagInstancesEdit]} type="disabled">
            <Button
              type="text"
              icon={<SettingOutlined />}
              size="small"
              onClick={() => setConfigurationVisible(!configurationVisible)}
            />
          </PermissionGuard>
        </Flex>
      }
    >
      <div className="flex flex-col gap-2">
        <div className={'flex items-center justify-between'}>
          <Text type="secondary">Engine Type:</Text>
          <Tag color="blue">
            {ragInstance?.engine_type === 'simple_vector' ? 'Vector' : 'Graph'}
          </Tag>
        </div>
        <div className={'flex items-center justify-between'}>
          <Text type="secondary">Status:</Text>
          <Tag color={ragInstance?.is_active ? 'green' : 'red'}>
            {ragInstance?.is_active ? 'Active' : 'Inactive'}
          </Tag>
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
      {configurationVisible && (
        <div className="mt-4">
          <Form
            form={form}
            layout="vertical"
            onFinish={handleConfigurationSubmit}
            disabled={updatingInstance}
          >
            <Form.Item
              label="Instance Name"
              name="name"
              rules={[{ required: true, message: 'Please enter instance name' }]}
            >
              <Input placeholder="Enter instance name" />
            </Form.Item>

            <Form.Item
              label="Description"
              name="description"
            >
              <Input.TextArea 
                placeholder="Enter instance description" 
                rows={2}
              />
            </Form.Item>

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

            <Form.Item
              label="Embedding Model"
              name="embedding_model_id"
            >
              <Select
                placeholder="Select embedding model"
                allowClear
                showSearch
                filterOption={(input, option) =>
                  (option?.label ?? '').toLowerCase().includes(input.toLowerCase())
                }
                onDropdownVisibleChange={(open) => {
                  if (open && !providersInitialized) {
                    loadUserProviders().catch(console.error)
                  }
                }}
                options={getAvailableModels().map(model => ({
                  label: model.name,
                  value: model.id,
                }))}
              />
            </Form.Item>

            <Form.Item
              label="LLM Model"
              name="llm_model_id"
            >
              <Select
                placeholder="Select LLM model"
                allowClear
                showSearch
                filterOption={(input, option) =>
                  (option?.label ?? '').toLowerCase().includes(input.toLowerCase())
                }
                onDropdownVisibleChange={(open) => {
                  if (open && !providersInitialized) {
                    loadUserProviders().catch(console.error)
                  }
                }}
                options={getAvailableModels().map(model => ({
                  label: model.name,
                  value: model.id,
                }))}
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Engine Settings</Text>
            </Divider>

            <Form.Item shouldUpdate={(prevValues, currentValues) => 
              prevValues.engine_type !== currentValues.engine_type
            }>
              {({ getFieldValue }) => {
                const engineType = getFieldValue('engine_type')
                return (
                  <>
                    {engineType === 'simple_vector' && (
                      <Card size="small" className="bg-blue-50">
                        <Text strong>Vector Engine Settings</Text>
                        <Form.Item
                          label="Chunk Size"
                          name={['engine_settings', 'simple_vector', 'chunk_size']}
                          className="mb-2 mt-2"
                        >
                          <InputNumber
                            placeholder="1000"
                            min={100}
                            max={10000}
                            style={{ width: '100%' }}
                          />
                        </Form.Item>
                        <Form.Item
                          label="Overlap"
                          name={['engine_settings', 'simple_vector', 'overlap']}
                          className="mb-0"
                        >
                          <InputNumber
                            placeholder="200"
                            min={0}
                            max={1000}
                            style={{ width: '100%' }}
                          />
                        </Form.Item>
                      </Card>
                    )}
                    
                    {engineType === 'simple_graph' && (
                      <Card size="small" className="bg-green-50">
                        <Text strong>Graph Engine Settings</Text>
                        <Form.Item
                          label="Max Depth"
                          name={['engine_settings', 'simple_graph', 'max_depth']}
                          className="mb-2 mt-2"
                        >
                          <InputNumber
                            placeholder="3"
                            min={1}
                            max={10}
                            style={{ width: '100%' }}
                          />
                        </Form.Item>
                        <Form.Item
                          label="Min Score"
                          name={['engine_settings', 'simple_graph', 'min_score']}
                          className="mb-0"
                        >
                          <InputNumber
                            placeholder="0.5"
                            min={0}
                            max={1}
                            step={0.1}
                            style={{ width: '100%' }}
                          />
                        </Form.Item>
                      </Card>
                    )}
                  </>
                )
              }}
            </Form.Item>

            <Form.Item className="mb-0 mt-3">
              <Space>
                <Button
                  type="primary"
                  htmlType="submit"
                  loading={updatingInstance}
                >
                  Save
                </Button>
                <Button onClick={() => setConfigurationVisible(false)}>
                  Cancel
                </Button>
              </Space>
            </Form.Item>
          </Form>
        </div>
      )}
    </Card>
  )
}