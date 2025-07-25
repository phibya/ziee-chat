import { Button, Card, Flex, Form } from 'antd'
import { Drawer } from '../../../common/Drawer.tsx'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  closeEditLocalModelDrawer,
  Stores,
  updateExistingModel,
} from '../../../../store'
import { BASIC_MODEL_FIELDS, LOCAL_PARAMETERS } from './shared/constants'
import { DeviceSelectionSection } from './shared/DeviceSelectionSection'
import { ModelCapabilitiesSection } from './shared/ModelCapabilitiesSection'
import { ModelParametersSection } from './shared/ModelParametersSection'
import { ModelSettingsSection } from './shared/ModelSettingsSection'

export function EditLocalModelDrawer() {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const { open, modelId } = Stores.UI.EditLocalModelModal
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
            <ModelParametersSection parameters={LOCAL_PARAMETERS} />
          </Card>
        </Flex>
      </Form>
    </Drawer>
  )
}
