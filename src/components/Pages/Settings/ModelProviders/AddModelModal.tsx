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
      title="Add Model"
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
          Add Model
        </Button>,
      ]}
      width={600}
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
          label="Name"
          rules={[{ required: true, message: 'Please enter a model name' }]}
        >
          <Input placeholder="Enter model name" />
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
