import { Button, Card, Form, Select, Typography, Switch, App } from 'antd'
import React, { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useRAGInstanceStore } from '../../../store/ragInstance'
import { toggleRAGInstanceActivate } from '../../../store/rag'
import { Permission, UpdateRAGInstanceRequest } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { RagSimpleVectorEngineSettings } from './RagSimpleVectorEngineSettings.tsx'
import { RagSimpleGraphEngineSettings } from './RagSimpleGraphEngineSettings.tsx'
import { disconnectRAGStatus, subscribeToRAGStatus } from '../../../store'

const { Text } = Typography

export const RagInstanceSettingsTab: React.FC = () => {
  const { ragInstanceId } = useParams<{ ragInstanceId: string }>()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [activationForm] = Form.useForm()
  const [updatingInstance, setUpdatingInstance] = useState(false)
  const engineType = Form.useWatch('engine_type', form)

  // RAG instance store
  const { ragInstance, updateRAGInstance } = useRAGInstanceStore(ragInstanceId)

  useEffect(() => {
    if (ragInstanceId) {
      subscribeToRAGStatus(ragInstanceId).catch(console.error)
    }

    return () => {
      disconnectRAGStatus()
    }
  }, [ragInstanceId])

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

  return (
    <div className="flex flex-col gap-3 w-full">
      {/* Instance Information Card */}
      <Card
        title={
          <Typography.Title level={5} className={'!m-0 !pt-[2px]'}>
            Instance Configuration
          </Typography.Title>
        }
      >
        <div className="flex flex-col gap-2">
          {' '}
          <PermissionGuard
            permissions={[Permission.RagInstancesEdit]}
            type="disabled"
          >
            <div className={'flex items-center justify-between pb-2'}>
              <Text type="secondary">Active:</Text>
              <Form form={activationForm}>
                <Form.Item name="is_active" noStyle>
                  <Switch onChange={handleActivationToggle} />
                </Form.Item>
              </Form>
            </div>
          </PermissionGuard>
        </div>

        {/* RAG Configuration Form */}
        <div className="mt-1">
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
                rules={[
                  { required: true, message: 'Please select engine type' },
                ]}
                help="Changing the engine type will require reprocessing all documents"
                className={'!pb-3'}
              >
                <Select
                  placeholder="Select engine type"
                  options={[
                    { value: 'simple_vector', label: 'Simple Vector Search' },
                    { value: 'simple_graph', label: 'Simple Graph Search' },
                  ]}
                />
              </Form.Item>

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
