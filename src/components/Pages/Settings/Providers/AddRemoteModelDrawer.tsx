import { App, Button, Form } from 'antd'
import { Drawer } from '../../../Common/Drawer'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  addNewModelToProvider,
  clearProvidersError,
  closeAddRemoteModelDrawer,
  loadAllModelProviders,
  Stores,
} from '../../../../store'
import { ModelParametersSection } from './common/ModelParametersSection'
import { BASIC_MODEL_FIELDS } from '../../../../constants/modelParameters.ts'

export function AddRemoteModelDrawer() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  // Get modal state from store
  const { open, providerId } = Stores.UI.AddRemoteModelDrawer

  const handleSubmit = async () => {
    try {
      setLoading(true)
      clearProvidersError() // Clear any previous errors
      const values = await form.validateFields()

      // For remote providers, use the existing workflow
      const modelData = {
        id: `model-${Date.now()}`,
        ...values,
        enabled: true,
        capabilities: {
          vision: values.vision || false,
          audio: values.audio || false,
          tools: values.tools || false,
          codeInterpreter: values.codeInterpreter || false,
        },
      }

      // Remove capability checkboxes from main data
      delete modelData.vision
      delete modelData.audio
      delete modelData.tools
      delete modelData.codeInterpreter

      await addNewModelToProvider(providerId || '', modelData)
      await loadAllModelProviders()

      form.resetFields()
      closeAddRemoteModelDrawer()

      message.success(t('providers.modelAddedSuccessfully'))
    } catch (error) {
      console.error('Failed to add model:', error)
      message.error(t('providers.failedToCreateModel'))
    } finally {
      setLoading(false)
    }
  }

  const handleCancel = () => {
    form.resetFields()
    closeAddRemoteModelDrawer()
  }

  return (
    <Drawer
      title={t('providers.addRemoteModel')}
      open={open}
      onClose={handleCancel}
      footer={[
        <Button key="cancel" onClick={handleCancel}>
          {t('buttons.cancel')}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          {t('buttons.add')}
        </Button>,
      ]}
      width={600}
      maskClosable={false}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          enabled: true,
          vision: false,
          audio: false,
          tools: false,
          codeInterpreter: false,
        }}
      >
        <ModelParametersSection parameters={BASIC_MODEL_FIELDS} />
      </Form>
    </Drawer>
  )
}
