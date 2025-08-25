import { Button, Form, Input, Select, Switch } from 'antd'
import { Drawer } from '../../../common/Drawer'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  RAG_ENGINE_TYPES,
  RAG_ENGINE_DEFAULTS,
} from '../../../../constants/ragProviders'
import {
  closeEditSystemInstanceDrawer,
  findRAGInstanceById,
  setEditSystemInstanceDrawerLoading,
  Stores,
  updateSystemRAGInstance,
} from '../../../../store'
import { UpdateRAGInstanceRequest, RAGEngineType } from '../../../../types/api'

export function EditSystemInstanceDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [engineType, setEngineType] = useState<RAGEngineType>('rag_simple_vector')

  const { open, loading, instanceId } = Stores.UI.EditSystemInstanceDrawer

  // Find the current instance from the store
  const instance = instanceId ? findRAGInstanceById(instanceId) : null

  useEffect(() => {
    if (instance && open) {
      setEngineType(instance.engine_type)
      
      // Get engine settings based on engine type
      let engineSettings = {}
      if (instance.engine_type === 'rag_simple_vector' && instance.engine_settings_rag_simple_vector) {
        engineSettings = instance.engine_settings_rag_simple_vector
      } else if (instance.engine_type === 'rag_simple_graph' && instance.engine_settings_rag_simple_graph) {
        engineSettings = instance.engine_settings_rag_simple_graph
      }
      
      form.setFieldsValue({
        name: instance.name,
        description: instance.description,
        engine_type: instance.engine_type,
        enabled: instance.enabled,
        settings: JSON.stringify(engineSettings, null, 2),
      })
    }
  }, [instance, open, form])

  const handleSubmit = async () => {
    if (!instance) return

    try {
      setEditSystemInstanceDrawerLoading(true)
      const values = await form.validateFields()
      
      let parsedSettings = values.settings
      if (typeof values.settings === 'string') {
        try {
          parsedSettings = JSON.parse(values.settings)
        } catch (error) {
          console.error('Invalid JSON settings:', error)
          parsedSettings = getDefaultEngineSettings(values.engine_type)
        }
      }

      const requestData: UpdateRAGInstanceRequest = {
        name: values.name,
        description: values.description,
        enabled: values.enabled,
        engine_settings: parsedSettings,
      }

      await updateSystemRAGInstance(instance.id, requestData)
      closeEditSystemInstanceDrawer()
    } catch (error) {
      console.error('Failed to update system RAG instance:', error)
    } finally {
      setEditSystemInstanceDrawerLoading(false)
    }
  }

  const handleEngineTypeChange = (type: RAGEngineType) => {
    setEngineType(type)
    const defaultSettings = getDefaultEngineSettings(type)
    form.setFieldValue('settings', JSON.stringify(defaultSettings, null, 2))
  }

  const getDefaultEngineSettings = (type: RAGEngineType) => {
    return RAG_ENGINE_DEFAULTS[type] || {}
  }

  if (!instance) return null

  return (
    <Drawer
      title={`Edit Instance: ${instance.name}`}
      open={open}
      onClose={closeEditSystemInstanceDrawer}
      footer={[
        <Button key="cancel" onClick={closeEditSystemInstanceDrawer}>
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
      <Form form={form} layout="vertical">
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