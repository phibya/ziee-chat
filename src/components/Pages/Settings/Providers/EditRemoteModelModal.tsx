import { Button, Flex, Form } from 'antd'
import { Drawer } from '../../../UI/Drawer'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  closeEditRemoteModelModal,
  Stores,
  updateExistingModel,
} from '../../../../store'
import { BASIC_MODEL_FIELDS } from './shared/constants'
import { ModelCapabilitiesSection } from './shared/ModelCapabilitiesSection'
import { ModelParametersSection } from './shared/ModelParametersSection'

export function EditRemoteModelModal() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const { open, modelId } = Stores.UI.EditRemoteModelModal
  const { modelsByProvider } = Stores.Providers

  // Find the current model from the store
  const currentModel = modelId
    ? Object.values(modelsByProvider)
        .flat()
        .find(m => m.id === modelId)
    : null

  useEffect(() => {
    if (currentModel && open) {
      form.setFieldsValue({
        name: currentModel.name,
        alias: currentModel.alias,
        description: currentModel.description,
        capabilities: currentModel.capabilities || {},
        parameters: currentModel.parameters || {},
      })
    }
  }, [currentModel, open, form])

  const handleSubmit = async () => {
    if (!currentModel) return

    try {
      setLoading(true)
      const values = await form.validateFields()

      const modelData = {
        ...currentModel,
        ...values,
      }
      await updateExistingModel(modelData.id, modelData)
      closeEditRemoteModelModal()
    } catch (error) {
      console.error('Failed to update remote model:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Drawer
      title={t('providers.editRemoteModel')}
      open={open}
      onClose={closeEditRemoteModelModal}
      footer={[
        <Button key="cancel" onClick={closeEditRemoteModelModal}>
          {t('buttons.cancel')}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          {t('buttons.saveChanges')}
        </Button>,
      ]}
      width={600}
      maskClosable={false}
    >
      <Form form={form} layout="vertical">
        <ModelParametersSection parameters={BASIC_MODEL_FIELDS} />

        <Flex className={`flex-col gap-3`}>
          <ModelCapabilitiesSection />
        </Flex>
      </Form>
    </Drawer>
  )
}
