import {
  Button,
  Form,
  Input,
  InputNumber,
  message,
  Modal,
  Space,
  Switch,
  Typography,
  Upload,
} from 'antd'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { UploadOutlined } from '@ant-design/icons'
import { ModelProviderType } from '../../../../types/api/modelProvider'

const { Title } = Typography
const { TextArea } = Input

interface AddModelModalProps {
  open: boolean
  providerType: ModelProviderType
  onClose: () => void
  onSubmit: (modelData: any) => void
}

export function AddModelModal({
  open,
  providerType,
  onClose,
  onSubmit,
}: AddModelModalProps) {
  const { t } = useTranslation()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)

  const handleSubmit = async () => {
    try {
      setLoading(true)
      const values = await form.validateFields()

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

      await onSubmit(modelData)
      form.resetFields()
    } catch (error) {
      console.error('Failed to add model:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleFileSelect = (info: any) => {
    if (info.file.status === 'done') {
      form.setFieldsValue({ path: info.file.response?.path || info.file.name })
      message.success(`${info.file.name} file selected successfully`)
    }
  }

  const renderLlamaCppParameters = () => (
    <>
      <Title level={5}>{t('modelProviders.parameters')}</Title>

      <Form.Item
        label={t('modelProviders.contextSize')}
        name={['parameters', 'contextSize']}
        help={t('modelProviders.contextSizeHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.contextSizePlaceholder')}
          style={{ width: '100%' }}
          min={0}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.gpuLayers')}
        name={['parameters', 'gpuLayers']}
        help={t('modelProviders.nglHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.gpuLayersPlaceholder')}
          style={{ width: '100%' }}
          min={-1}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.temperature')}
        name={['parameters', 'temperature']}
        help={t('modelProviders.temperatureHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.temperaturePlaceholder')}
          style={{ width: '100%' }}
          min={0}
          max={2}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.topK')}
        name={['parameters', 'topK']}
        help={t('modelProviders.topKHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.topKPlaceholder')}
          style={{ width: '100%' }}
          min={0}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.topP')}
        name={['parameters', 'topP']}
        help={t('modelProviders.topPHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.topPPlaceholder')}
          style={{ width: '100%' }}
          min={0}
          max={1}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.minP')}
        name={['parameters', 'minP']}
        help={t('modelProviders.minPHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.minPPlaceholder')}
          style={{ width: '100%' }}
          min={0}
          max={1}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.repeatLastN')}
        name={['parameters', 'repeatLastN']}
        help={t('modelProviders.repeatLastNHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.repeatLastNPlaceholder')}
          style={{ width: '100%' }}
          min={-1}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.repeatPenalty')}
        name={['parameters', 'repeatPenalty']}
        help={t('modelProviders.repeatPenaltyHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.repeatPenaltyPlaceholder')}
          style={{ width: '100%' }}
          min={0}
          max={2}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.presencePenalty')}
        name={['parameters', 'presencePenalty']}
        help={t('modelProviders.presencePenaltyHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.presencePenaltyPlaceholder')}
          style={{ width: '100%' }}
          min={0}
          max={2}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label={t('modelProviders.frequencyPenalty')}
        name={['parameters', 'frequencyPenalty']}
        help={t('modelProviders.frequencyPenaltyHelp')}
      >
        <InputNumber
          placeholder={t('modelProviders.frequencyPenaltyPlaceholder')}
          style={{ width: '100%' }}
          min={0}
          max={2}
          step={0.1}
        />
      </Form.Item>
    </>
  )

  return (
    <Modal
      title={t('modelProviders.addModel')}
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
          {t('modelProviders.addModel')}
        </Button>,
      ]}
      width={600}
      maskClosable={false}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          parameters: {
            contextSize: 8192,
            gpuLayers: 100,
            temperature: 0.6,
            topK: 40,
            topP: 0.9,
            minP: 0.1,
            repeatLastN: 64,
            repeatPenalty: 1.0,
            presencePenalty: 0.0,
            frequencyPenalty: 0.0,
          },
        }}
      >
        <Form.Item
          name="name"
          label={t('modelProviders.modelId')}
          rules={[
            { required: true, message: t('modelProviders.enterModelId') },
          ]}
          help={t('modelProviders.modelIdHelp')}
        >
          <Input placeholder={t('modelProviders.modelIdPlaceholder')} />
        </Form.Item>

        <Form.Item
          name="alias"
          label={t('modelProviders.displayName')}
          rules={[
            { required: true, message: t('modelProviders.enterDisplayName') },
          ]}
          help={t('modelProviders.displayNameHelp')}
        >
          <Input placeholder={t('modelProviders.displayNamePlaceholder')} />
        </Form.Item>

        <Form.Item name="description" label={t('modelProviders.description')}>
          <TextArea
            placeholder={t('modelProviders.descriptionPlaceholder')}
            rows={3}
          />
        </Form.Item>

        {providerType === 'llama.cpp' && (
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

        <Title level={5}>{t('modelProviders.capabilities')}</Title>
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.vision')}</span>
            <Form.Item
              name="vision"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.audio')}</span>
            <Form.Item
              name="audio"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.tools')}</span>
            <Form.Item
              name="tools"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.codeInterpreter')}</span>
            <Form.Item
              name="codeInterpreter"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
        </Space>

        {providerType === 'llama.cpp' && renderLlamaCppParameters()}
      </Form>
    </Modal>
  )
}
