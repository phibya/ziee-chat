import { Button, Form, Input, Switch } from 'antd'
import { Drawer } from '../../../common/Drawer'
import { useTranslation } from 'react-i18next'
import { RAG_ENGINE_DEFAULTS } from '../../../../constants/ragProviders'
import {
  closeAddSystemInstanceDrawer,
  createSystemRAGInstance,
  setAddSystemInstanceDrawerLoading,
  Stores,
} from '../../../../store'
import {
  CreateSystemRAGInstanceRequest,
  RAGEngineType,
} from '../../../../types/api'

export function AddSystemInstanceDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()

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
        llm_model_id: undefined,
        parameters: {},
        engine_type: 'simple_vector',
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
          engine_type: 'simple_vector',
          enabled: true,
          settings: getDefaultEngineSettings('simple_vector'),
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

        <Form.Item name="enabled" label="Enabled" valuePropName="checked">
          <Switch />
        </Form.Item>

        <Form.Item name="description" label="Description">
          <Input.TextArea
            placeholder="Optional description for this instance"
            rows={3}
          />
        </Form.Item>
      </Form>
    </Drawer>
  )
}
