import { App, Button, Form, Modal } from 'antd'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  addNewModelToProvider,
  clearProvidersError,
  closeAddRemoteModelModal,
  loadAllModelProviders,
  Stores,
} from '../../../../store'
import { BASIC_MODEL_FIELDS } from './shared/constants'
import { ModelParametersSection } from './shared/ModelParametersSection'

export function AddRemoteModelModal() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  // Get modal state from store
  const { open, providerId } = Stores.UI.AddRemoteModelModal

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
      closeAddRemoteModelModal()

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
    closeAddRemoteModelModal()
  }

  return (
    <Modal
      title={t('providers.addRemoteModel')}
      open={open}
      onCancel={handleCancel}
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
      destroyOnHidden={true}
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
    </Modal>
  )
}
