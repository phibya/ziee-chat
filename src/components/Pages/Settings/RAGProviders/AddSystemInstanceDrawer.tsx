import { Button, Form, Input, Select, Switch } from 'antd'
import { Drawer } from '../../../common/Drawer'
import { useTranslation } from 'react-i18next'
import {
  RAG_ENGINE_TYPES,
  RAG_ENGINE_DEFAULTS,
} from '../../../../constants/ragProviders'
import {
  closeAddSystemInstanceDrawer,
  createSystemRAGInstance,
  setAddSystemInstanceDrawerLoading,
  Stores,
} from '../../../../store'
import { CreateSystemRAGInstanceRequest, RAGEngineType } from '../../../../types/api'
import { useState } from 'react'

export function AddSystemInstanceDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [engineType, setEngineType] = useState<RAGEngineType>('rag_simple_vector')

  const { open, loading, providerId } = Stores.UI.AddSystemInstanceDrawer

  const handleSubmit = async () => {
    if (!providerId) return

    try {
      const values = await form.validateFields()
      setAddSystemInstanceDrawerLoading(true)
      
      const requestData: CreateSystemRAGInstanceRequest = {
        provider_id: providerId,
        name: values.name,
        alias: values.name, // Use name as alias
        description: values.description,
        engine_type: values.engine_type,
        embedding_model_id: 'default', // Required field - use default
        llm_model_id: undefined,
        parameters: {},
        engine_settings: values.settings || getDefaultEngineSettings(values.engine_type),
      }

      await createSystemRAGInstance(providerId, requestData)
      closeAddSystemInstanceDrawer()
      form.resetFields()
    } catch (error) {
      console.error('Failed to create system RAG instance:', error)
    } finally {
      setAddSystemInstanceDrawerLoading(false)
    }
  }

  const handleEngineTypeChange = (type: RAGEngineType) => {
    setEngineType(type)
    const defaultSettings = getDefaultEngineSettings(type)
    form.setFieldValue('settings', defaultSettings)
  }

  const getDefaultEngineSettings = (type: RAGEngineType) => {
    return RAG_ENGINE_DEFAULTS[type] || {}
  }

  const handleClose = () => {
    form.resetFields()
    closeAddSystemInstanceDrawer()
  }

  return (
    <Drawer
      title="Add Instance"
      open={open}
      onClose={handleClose}
      footer={[
        <Button key="cancel" onClick={handleClose}>
          {t('buttons.cancel')}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          {t('buttons.ok')}
        </Button>,
      ]}
      width={500}
      maskClosable={false}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          engine_type: 'rag_simple_vector',
          enabled: true,
          settings: getDefaultEngineSettings('rag_simple_vector'),
        }}
      >
        <Form.Item
          name="name"
          label="Instance Name"
          rules={[
            {
              required: true,
              message: 'Instance name is required',
            },
          ]}
        >
          <Input placeholder="Enter instance name" />
        </Form.Item>

        <Form.Item
          name="engine_type"
          label="Engine Type"
          rules={[
            {
              required: true,
              message: 'Engine type is required',
            },
          ]}
        >
          <Select
            options={RAG_ENGINE_TYPES.map(engine => ({
              value: engine.value,
              label: engine.label,
              engine: engine,
            }))}
            optionRender={(option) => (
              <div className="flex flex-col gap-1 py-1">
                <div className="font-medium">{option.data.engine.label}</div>
                <div className="text-xs text-gray-500">{option.data.engine.description}</div>
              </div>
            )}
            onChange={handleEngineTypeChange}
            placeholder="Select engine type"
          />
        </Form.Item>

        <Form.Item
          name="enabled"
          label="Enabled"
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        <Form.Item
          name="description"
          label="Description"
        >
          <Input.TextArea 
            placeholder="Optional description for this instance"
            rows={3}
          />
        </Form.Item>

        <Form.Item
          name="settings"
          label="Engine Settings (JSON)"
          help="Advanced settings for the RAG engine in JSON format"
        >
          <Input.TextArea
            placeholder={JSON.stringify(getDefaultEngineSettings(engineType), null, 2)}
            rows={6}
          />
        </Form.Item>
      </Form>
    </Drawer>
  )
}