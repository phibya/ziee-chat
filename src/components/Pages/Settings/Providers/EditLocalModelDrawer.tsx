import { Button, Card, Flex, Form } from 'antd'
import { Drawer } from '../../../Common/Drawer'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  closeEditLocalModelDrawer,
  Stores,
  updateExistingModel,
} from '../../../../store'
import { DeviceSelectionSection } from './common/DeviceSelectionSection'
import { ModelCapabilitiesSection } from './common/ModelCapabilitiesSection'
import { ModelParametersSection } from './common/ModelParametersSection'
import { ModelSettingsSection } from './common/ModelSettingsSection'
import {
  BASIC_MODEL_FIELDS,
  MODEL_PARAMETERS,
} from '../../../../constants/modelParameters.ts'

export function EditLocalModelDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const { open, modelId } = Stores.UI.EditLocalModelDrawer
  const { providers } = Stores.AdminProviders

  // Find the current model from all providers
  const currentModel = modelId
    ? providers.flatMap(p => p.models).find(m => m.id === modelId)
    : null

  useEffect(() => {
    if (currentModel && open) {
      form.setFieldsValue({
        name: currentModel.name,
        alias: currentModel.alias,
        description: currentModel.description,
        capabilities: currentModel.capabilities || {},
        parameters: currentModel.parameters || {},
        settings: currentModel.settings || {},
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
      closeEditLocalModelDrawer()
    } catch (error) {
      console.error('Failed to update local model:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Drawer
      title={t('providers.editLocalModel')}
      open={open}
      onClose={closeEditLocalModelDrawer}
      footer={[
        <Button key="cancel" onClick={closeEditLocalModelDrawer}>
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
          <DeviceSelectionSection />
          <ModelSettingsSection />

          <Card title={t('providers.parameters')} size={'small'}>
            <ModelParametersSection parameters={MODEL_PARAMETERS} />
          </Card>
        </Flex>
      </Form>
    </Drawer>
  )
}
