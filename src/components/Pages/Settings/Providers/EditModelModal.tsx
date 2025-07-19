import { Button, Card, Flex, Form, Modal } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Model, ProviderType } from '../../../../types/api/provider'
import { ModelCapabilitiesSection } from './shared/ModelCapabilitiesSection'
import { DeviceSelectionSection } from './shared/DeviceSelectionSection'
import { ModelParametersSection } from './shared/ModelParametersSection'
import { ModelSettingsSection } from './shared/ModelSettingsSection'
import { BASIC_MODEL_FIELDS, LOCAL_PARAMETERS } from './shared/constants'

interface EditModelModalProps {
  open: boolean
  model: Model | null
  providerType: ProviderType
  onClose: () => void
  onSubmit: (modelData: any) => void
}

export function EditModelModal({
  open,
  model,
  providerType,
  onClose,
  onSubmit,
}: EditModelModalProps) {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (model && open) {
      form.setFieldsValue({
        name: model.name,
        alias: model.alias,
        description: model.description,
        capabilities: model.capabilities || {},
        parameters: model.parameters || {},
        settings: model.settings || {},
      })

      console.log({ model, f: form.getFieldsValue() })
    }
  }, [model, open, form])

  const handleSubmit = async () => {
    try {
      setLoading(true)
      const values = await form.validateFields()

      const modelData = {
        ...model,
        ...values,
      }
      await onSubmit(modelData)
    } catch (error) {
      console.error('Failed to update model:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Modal
      title={t('providers.editModel')}
      open={open}
      onCancel={onClose}
      footer={[
        <Button key="cancel" onClick={onClose}>
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

          {providerType === 'local' && <DeviceSelectionSection />}

          {providerType === 'local' && <ModelSettingsSection />}

          {providerType === 'local' && (
            <Card title={t('providers.parameters')} size={'small'}>
              <ModelParametersSection parameters={LOCAL_PARAMETERS} />
            </Card>
          )}
        </Flex>
      </Form>
    </Modal>
  )
}
