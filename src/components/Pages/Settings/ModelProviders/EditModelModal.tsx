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
import { useEffect, useState } from 'react'
import { UploadOutlined } from '@ant-design/icons'
import {
  ModelProviderModel,
  ModelProviderType,
} from '../../../../types/api/modelProvider'

const { Title } = Typography
const { TextArea } = Input

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

  const handleFileSelect = (info: any) => {
    if (info.file.status === 'done') {
      form.setFieldsValue({ path: info.file.response?.path || info.file.name })
      message.success(`${info.file.name} file selected successfully`)
    }
  }

  const renderLlamaCppParameters = () => (
    <>
      <Title level={5}>Parameters</Title>

      <Form.Item
        label="Context Size"
        name={['parameters', 'contextSize']}
        help="Size of the prompt context (0 = loaded from model)"
      >
        <InputNumber placeholder="8192" style={{ width: '100%' }} min={0} />
      </Form.Item>

      <Form.Item
        label="GPU Layers"
        name={['parameters', 'gpuLayers']}
        help="Number of model layers to offload to the GPU (-1 for all layers, 0 for CPU only)"
      >
        <InputNumber placeholder="100" style={{ width: '100%' }} min={-1} />
      </Form.Item>

      <Form.Item
        label="Temperature"
        name={['parameters', 'temperature']}
        help="Temperature for sampling (higher = more random)"
      >
        <InputNumber
          placeholder="0.6"
          style={{ width: '100%' }}
          min={0}
          max={2}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label="Top K"
        name={['parameters', 'topK']}
        help="Top-K sampling (0 = disabled)"
      >
        <InputNumber placeholder="40" style={{ width: '100%' }} min={0} />
      </Form.Item>

      <Form.Item
        label="Top P"
        name={['parameters', 'topP']}
        help="Top-P sampling (1.0 = disabled)"
      >
        <InputNumber
          placeholder="0.9"
          style={{ width: '100%' }}
          min={0}
          max={1}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label="Min P"
        name={['parameters', 'minP']}
        help="Min-P sampling (0.0 = disabled)"
      >
        <InputNumber
          placeholder="0.1"
          style={{ width: '100%' }}
          min={0}
          max={1}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label="Repeat Last N"
        name={['parameters', 'repeatLastN']}
        help="Number of tokens to consider for repeat penalty (0 = disabled, -1 = ctx_size)"
      >
        <InputNumber placeholder="64" style={{ width: '100%' }} min={-1} />
      </Form.Item>

      <Form.Item
        label="Repeat Penalty"
        name={['parameters', 'repeatPenalty']}
        help="Penalize repeating token sequences (1.0 = disabled)"
      >
        <InputNumber
          placeholder="1.0"
          style={{ width: '100%' }}
          min={0}
          max={2}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label="Presence Penalty"
        name={['parameters', 'presencePenalty']}
        help="Repeat alpha presence penalty (0.0 = disabled)"
      >
        <InputNumber
          placeholder="0.0"
          style={{ width: '100%' }}
          min={0}
          max={2}
          step={0.1}
        />
      </Form.Item>

      <Form.Item
        label="Frequency Penalty"
        name={['parameters', 'frequencyPenalty']}
        help="Repeat alpha frequency penalty (0.0 = disabled)"
      >
        <InputNumber
          placeholder="0.0"
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
      title="Edit Model"
      open={open}
      onCancel={onClose}
      footer={[
        <Button key="cancel" onClick={onClose}>
          Cancel
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          Save Changes
        </Button>,
      ]}
      width={600}
      maskClosable={false}
    >
      <Form form={form} layout="vertical">
        <Form.Item
          name="name"
          label="Model ID"
          rules={[{ required: true, message: 'Please enter a model ID' }]}
          help="The technical model identifier used by the provider (e.g., 'claude-3-sonnet-20240229', 'gpt-4')"
        >
          <Input placeholder="Enter model ID (e.g., claude-3-sonnet-20240229)" />
        </Form.Item>

        <Form.Item
          name="alias"
          label="Display Name"
          rules={[{ required: true, message: 'Please enter a display name' }]}
          help="User-friendly name shown in the interface"
        >
          <Input placeholder="Enter display name (e.g., Claude 3 Sonnet)" />
        </Form.Item>

        <Form.Item name="description" label="Description">
          <TextArea placeholder="Enter model description" rows={3} />
        </Form.Item>

        {providerType === 'llama.cpp' && (
          <Form.Item
            name="path"
            label="Model Path"
            rules={[{ required: true, message: 'Please select a model file' }]}
          >
            <Input
              placeholder="Select model file"
              addonAfter={
                <Upload
                  showUploadList={false}
                  beforeUpload={() => false}
                  onChange={handleFileSelect}
                >
                  <Button icon={<UploadOutlined />} size="small">
                    Browse
                  </Button>
                </Upload>
              }
            />
          </Form.Item>
        )}

        <Title level={5}>Capabilities</Title>
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>Vision</span>
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
            <span>Audio</span>
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
            <span>Tools</span>
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
            <span>Code Interpreter</span>
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
