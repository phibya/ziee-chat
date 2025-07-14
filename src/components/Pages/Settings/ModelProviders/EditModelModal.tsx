import { Button, Form, Input, Modal, Upload } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ModelProviderModel,
  ModelProviderType,
} from '../../../../types/api/modelProvider'
import { UploadOutlined } from '@ant-design/icons'
import { ModelCapabilitiesSection } from './shared/ModelCapabilitiesSection'
import { ModelParametersSection } from './shared/ModelParametersSection'
import { BASIC_MODEL_FIELDS, CANDLE_PARAMETERS } from './shared/constants'

interface EditModelModalProps {
  open: boolean
  model: ModelProviderModel | null
  providerType: ModelProviderType
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
        path: model.path,
        vision: model.capabilities?.vision,
        audio: model.capabilities?.audio,
        tools: model.capabilities?.tools,
        codeInterpreter: model.capabilities?.codeInterpreter,
        parameters: model.parameters || {},
      })
    }
  }, [model, open, form])

  const handleFileSelect = (info: any) => {
    const file = info.file.originFileObj || info.file

    if (file) {
      form.setFieldsValue({
        path: file.name,
      })
    }
  }

  const handleSubmit = async () => {
    try {
      setLoading(true)
      const values = await form.validateFields()

      const modelData = {
        ...model,
        ...values,
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

      await onSubmit(modelData)
    } catch (error) {
      console.error('Failed to update model:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Modal
      title={t('modelProviders.editModel')}
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

        {providerType === 'candle' && (
          <Form.Item
            name="path"
            label={t('modelProviders.modelPath')}
            rules={[
              {
                required: true,
                message: t('modelProviders.selectModelFileRequired'),
              },
            ]}
          >
            <Input
              placeholder={t('modelProviders.selectModelFile')}
              addonAfter={
                <Upload
                  showUploadList={false}
                  beforeUpload={() => false}
                  onChange={handleFileSelect}
                >
                  <Button icon={<UploadOutlined />} size="small">
                    {t('modelProviders.browse')}
                  </Button>
                </Upload>
              }
            />
          </Form.Item>
        )}

        <ModelCapabilitiesSection />

        {providerType === 'candle' && (
          <ModelParametersSection
            title={t('modelProviders.parameters')}
            parameters={CANDLE_PARAMETERS}
          />
        )}
      </Form>
    </Modal>
  )
}
