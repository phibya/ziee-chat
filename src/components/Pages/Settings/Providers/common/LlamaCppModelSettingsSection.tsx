import {
  Card,
  Divider,
  Flex,
  Form,
  InputNumber,
  Select,
  Switch,
  Typography,
} from 'antd'

const { Text } = Typography

export function LlamaCppModelSettingsSection() {
  const getFieldName = (field: string) => ['engine_settings_llamacpp', field]

  const ResponsiveConfigItem = ({
    title,
    description,
    children,
  }: {
    title: string
    description: string
    children: React.ReactNode
  }) => (
    <Flex justify="space-between">
      <div>
        <Text strong>{title}</Text>
        <div>
          <Text type="secondary">{description}</Text>
        </div>
      </div>
      {children}
    </Flex>
  )

  return (
    <Flex vertical className="gap-4 w-full">
      {/* Context & Memory Management */}
      <Card title="Context & Memory Management">
        <Flex vertical className="gap-2 w-full">
          <ResponsiveConfigItem
            title="Context Size"
            description="Size of the prompt context (default: 2048)"
          >
            <Form.Item name={getFieldName('n_ctx')}>
              <InputNumber
                min={512}
                max={131072}
                placeholder="2048"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Batch Size"
            description="Batch size for prompt processing (default: 2048)"
          >
            <Form.Item name={getFieldName('n_batch')}>
              <InputNumber
                min={1}
                max={8192}
                placeholder="2048"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Ubatch Size"
            description="Physical maximum batch size (default: 512)"
          >
            <Form.Item name={getFieldName('n_ubatch')}>
              <InputNumber
                min={1}
                max={2048}
                placeholder="512"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Memory Lock"
            description="Lock the model in memory, preventing it from being swapped out"
          >
            <Form.Item
              name={getFieldName('mlock')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Memory Map"
            description="Use memory mapping for faster model loading"
          >
            <Form.Item
              name={getFieldName('mmap')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>
        </Flex>
      </Card>

      {/* Threading & Performance */}
      <Card title="Threading & Performance">
        <Flex vertical className="gap-2 w-full">
          <ResponsiveConfigItem
            title="Threads"
            description="Number of threads to use for generation (default: auto)"
          >
            <Form.Item name={getFieldName('n_threads')}>
              <InputNumber
                min={1}
                max={64}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Batch Threads"
            description="Number of threads to use for batch processing (default: auto)"
          >
            <Form.Item name={getFieldName('n_threads_batch')}>
              <InputNumber
                min={1}
                max={64}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Flash Attention"
            description="Enable Flash Attention for faster inference"
          >
            <Form.Item
              name={getFieldName('flash_attn')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="No KV Offload"
            description="Disable KV cache offloading"
          >
            <Form.Item
              name={getFieldName('no_kv_offload')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>
        </Flex>
      </Card>

      {/* GPU Configuration */}
      <Card title="GPU Configuration">
        <Flex vertical className="gap-2 w-full">
          <ResponsiveConfigItem
            title="GPU Layers"
            description="Number of layers to offload to GPU (default: auto)"
          >
            <Form.Item name={getFieldName('n_gpu_layers')}>
              <InputNumber
                min={0}
                max={128}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Main GPU"
            description="Main GPU to use for tensor splits (default: 0)"
          >
            <Form.Item name={getFieldName('main_gpu')}>
              <InputNumber
                min={0}
                max={16}
                placeholder="0"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Split Mode"
            description="How to split the model across multiple GPUs"
          >
            <Form.Item name={getFieldName('split_mode')}>
              <Select
                placeholder="None"
                style={{ width: '100%' }}
                allowClear
                options={[
                  { value: 'none', label: 'None' },
                  { value: 'layer', label: 'Layer' },
                  { value: 'row', label: 'Row' },
                ]}
              />
            </Form.Item>
          </ResponsiveConfigItem>
        </Flex>
      </Card>

      {/* Model Configuration */}
      <Card title="Model Configuration">
        <Flex vertical className="gap-2 w-full">
          <ResponsiveConfigItem
            title="Rope Frequency Base"
            description="RoPE base frequency (default: auto)"
          >
            <Form.Item name={getFieldName('rope_freq_base')}>
              <InputNumber
                min={1000}
                max={1000000}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Rope Frequency Scale"
            description="RoPE frequency scaling factor (default: auto)"
          >
            <Form.Item name={getFieldName('rope_freq_scale')}>
              <InputNumber
                min={0.1}
                max={10.0}
                step={0.1}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Random Seed"
            description="Seed for random number generation (-1 for random)"
          >
            <Form.Item name={getFieldName('seed')}>
              <InputNumber
                min={-1}
                max={4294967295}
                placeholder="-1"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>
        </Flex>
      </Card>

      {/* Advanced Options */}
      <Card title="Advanced Options">
        <Flex vertical className="gap-2 w-full">
          <ResponsiveConfigItem
            title="Use Tensor Cores"
            description="Use Tensor Cores for matrix multiplication (if available)"
          >
            <Form.Item
              name={getFieldName('mul_mat_q')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Continuous Batching"
            description="Enable continuous batching for better throughput"
          >
            <Form.Item
              name={getFieldName('cont_batching')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Log Disable"
            description="Disable logging output"
          >
            <Form.Item
              name={getFieldName('log_disable')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>
        </Flex>
      </Card>
    </Flex>
  )
}