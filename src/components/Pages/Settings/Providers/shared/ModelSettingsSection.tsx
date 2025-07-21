import {
  Card,
  Divider,
  Flex,
  Form,
  InputNumber,
  Select,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'

const { Text } = Typography

export function ModelSettingsSection() {
  const [isMobile, setIsMobile] = useState(false)

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  const getFieldName = (field: string) => ['settings', field]

  const ResponsiveConfigItem = ({
    title,
    description,
    children,
  }: {
    title: string
    description: string
    children: React.ReactNode
  }) => (
    <Flex
      justify="space-between"
      align={isMobile ? 'flex-start' : 'center'}
      vertical={isMobile}
      gap={isMobile ? 'small' : 0}
    >
      <div style={{ flex: isMobile ? undefined : 1 }}>
        <Text strong>{title}</Text>
        <div>
          <Text type="secondary">{description}</Text>
        </div>
      </div>
      {children}
    </Flex>
  )

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      {/* Sequence & Memory Management */}
      <Card size="small" title="Sequence & Memory Management">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <ResponsiveConfigItem
            title="Max Sequences"
            description="Maximum running sequences at any time (default: 16)"
          >
            <Form.Item
              name={getFieldName('max_seqs')}
              style={{ margin: 0, width: isMobile ? '100%' : 100 }}
            >
              <InputNumber
                min={1}
                max={1024}
                placeholder="16"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Max Sequence Length"
            description="Maximum prompt sequence length to expect for this model (default: 4096)"
          >
            <Form.Item
              name={getFieldName('max_seq_len')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={512}
                max={131072}
                placeholder="4096"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="No KV Cache"
            description="Use no KV cache"
          >
            <Form.Item
              name={getFieldName('no_kv_cache')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Truncate Sequence"
            description="If a sequence is larger than the maximum model length, truncate the number of tokens such that the sequence will fit at most the maximum length"
          >
            <Form.Item
              name={getFieldName('truncate_sequence')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>
        </Space>
      </Card>

      {/* PagedAttention Configuration */}
      <Card size="small" title="PagedAttention Configuration">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <ResponsiveConfigItem
            title="PagedAttention GPU Memory (MB)"
            description="GPU memory to allocate for KV cache with PagedAttention in MBs"
          >
            <Form.Item
              name={getFieldName('paged_attn_gpu_mem')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={128}
                max={65536}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="PagedAttention GPU Memory Usage"
            description="Percentage of GPU memory to utilize after allocation of KV cache with PagedAttention, from 0 to 1 (default: 0.9 on CUDA)"
          >
            <Form.Item
              name={getFieldName('paged_attn_gpu_mem_usage')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={0.1}
                max={1.0}
                step={0.1}
                placeholder="0.9"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="PagedAttention Context Length"
            description="Total context length to allocate the KV cache for (total number of tokens which the KV cache can hold)"
          >
            <Form.Item
              name={getFieldName('paged_ctxt_len')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={512}
                max={131072}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="PagedAttention Block Size"
            description="Block size (number of tokens per block) for PagedAttention (default: 32 on CUDA)"
          >
            <Form.Item
              name={getFieldName('paged_attn_block_size')}
              style={{ margin: 0, width: isMobile ? '100%' : 100 }}
            >
              <InputNumber
                min={1}
                max={512}
                placeholder="32"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Disable PagedAttention"
            description="Disable PagedAttention on CUDA (PagedAttention is automatically activated on CUDA but not on Metal)"
          >
            <Form.Item
              name={getFieldName('no_paged_attn')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Enable PagedAttention on Metal"
            description="Enable PagedAttention on Metal (PagedAttention is automatically activated on CUDA but not on Metal)"
          >
            <Form.Item
              name={getFieldName('paged_attn')}
              valuePropName="checked"
              style={{ margin: 0 }}
            >
              <Switch />
            </Form.Item>
          </ResponsiveConfigItem>
        </Space>
      </Card>

      {/* Performance Optimization */}
      <Card size="small" title="Performance Optimization">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <ResponsiveConfigItem
            title="Prefix Cache Count"
            description="Number of prefix caches to hold on the device. Other caches are evicted to the CPU based on a LRU strategy (default: 16)"
          >
            <Form.Item
              name={getFieldName('prefix_cache_n')}
              style={{ margin: 0, width: isMobile ? '100%' : 100 }}
            >
              <InputNumber
                min={1}
                max={128}
                placeholder="16"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Prompt Chunk Size"
            description="Number of tokens to batch the prompt step into. This can help with OOM errors when in the prompt step, but reduces performance"
          >
            <Form.Item
              name={getFieldName('prompt_chunksize')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={1}
                max={8192}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>
        </Space>
      </Card>

      {/* Model Configuration */}
      <Card size="small" title="Model Configuration">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <ResponsiveConfigItem
            title="Data Type"
            description="Model data type (default: auto)"
          >
            <Form.Item
              name={getFieldName('dtype')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <Select
                placeholder="auto"
                style={{ width: '100%' }}
                allowClear
                options={[
                  { value: 'auto', label: 'Auto' },
                  { value: 'f16', label: 'Float16' },
                  { value: 'f32', label: 'Float32' },
                  { value: 'bf16', label: 'BFloat16' },
                ]}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="In-Situ Quantization"
            description="In-situ quantization to apply"
          >
            <Form.Item
              name={getFieldName('in_situ_quant')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <Select
                placeholder="None"
                style={{ width: '100%' }}
                allowClear
                options={[
                  { value: 'Q4_0', label: 'Q4_0' },
                  { value: 'Q4_1', label: 'Q4_1' },
                  { value: 'Q5_0', label: 'Q5_0' },
                  { value: 'Q5_1', label: 'Q5_1' },
                  { value: 'Q8_0', label: 'Q8_0' },
                ]}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Random Seed"
            description="Integer seed to ensure reproducible random number generation"
          >
            <Form.Item
              name={getFieldName('seed')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={0}
                max={4294967295}
                placeholder="Random"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>
        </Space>
      </Card>

      {/* Vision Model Settings */}
      <Card size="small" title="Vision Model Settings">
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <ResponsiveConfigItem
            title="Max Edge Length (Vision)"
            description="Automatically resize and pad images to this maximum edge length. Aspect ratio is preserved (vision models only)"
          >
            <Form.Item
              name={getFieldName('max_edge')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={224}
                max={2048}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Max Number of Images (Vision)"
            description="Maximum prompt number of images to expect for this model (vision models only)"
          >
            <Form.Item
              name={getFieldName('max_num_images')}
              style={{ margin: 0, width: isMobile ? '100%' : 100 }}
            >
              <InputNumber
                min={1}
                max={32}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>

          <Divider style={{ margin: 0 }} />

          <ResponsiveConfigItem
            title="Max Image Length (Vision)"
            description="Maximum expected image size will have this edge length on both edges (vision models only)"
          >
            <Form.Item
              name={getFieldName('max_image_length')}
              style={{ margin: 0, width: isMobile ? '100%' : 120 }}
            >
              <InputNumber
                min={224}
                max={2048}
                placeholder="Auto"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </ResponsiveConfigItem>
        </Space>
      </Card>
    </Space>
  )
}
