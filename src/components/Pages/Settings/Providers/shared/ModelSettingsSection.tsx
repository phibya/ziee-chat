import {
  Card,
  Divider,
  Flex,
  Form,
  InputNumber,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'

const { Text } = Typography

interface ModelSettingsSectionProps {
  disabled?: boolean
  useNestedSettings?: boolean
  wrapInCard?: boolean
}

export function ModelSettingsSection({
  disabled = false,
  useNestedSettings = true,
  wrapInCard = true,
}: ModelSettingsSectionProps) {
  const [isMobile, setIsMobile] = useState(false)

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  const getFieldName = (field: string) =>
    useNestedSettings ? ['settings', field] : field

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

  const content = (
    <Space direction="vertical" size="middle" style={{ width: '100%' }}>
      <ResponsiveConfigItem
        title="Verbose Mode"
        description="Print all requests and detailed logging information"
      >
        <Form.Item
          name={getFieldName('verbose')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="Max Sequences"
        description="Maximum number of sequences to allow (default: 256)"
      >
        <Form.Item
          name={getFieldName('max_num_seqs')}
          style={{ margin: 0, width: isMobile ? '100%' : 100 }}
        >
          <InputNumber
            min={1}
            max={1024}
            placeholder="256"
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="Block Size"
        description="Size of a block in tokens (default: 32)"
      >
        <Form.Item
          name={getFieldName('block_size')}
          style={{ margin: 0, width: isMobile ? '100%' : 100 }}
        >
          <InputNumber
            min={1}
            max={128}
            placeholder="32"
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="GPU KV Cache Memory (MB)"
        description="Available GPU memory for KV cache in megabytes (default: 4096)"
      >
        <Form.Item
          name={getFieldName('kvcache_mem_gpu')}
          style={{ margin: 0, width: isMobile ? '100%' : 120 }}
        >
          <InputNumber
            min={128}
            max={32768}
            placeholder="4096"
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="CPU KV Cache Memory (MB)"
        description="Available CPU memory for KV cache in megabytes (default: 128)"
      >
        <Form.Item
          name={getFieldName('kvcache_mem_cpu')}
          style={{ margin: 0, width: isMobile ? '100%' : 120 }}
        >
          <InputNumber
            min={64}
            max={8192}
            placeholder="128"
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="Record Conversation"
        description="Whether to record conversation history on the server (default: false)"
      >
        <Form.Item
          name={getFieldName('record_conversation')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="Holding Time (ms)"
        description="Maximum waiting time for processing parallel requests in milliseconds (default: 500)"
      >
        <Form.Item
          name={getFieldName('holding_time')}
          style={{ margin: 0, width: isMobile ? '100%' : 120 }}
        >
          <InputNumber
            min={100}
            max={5000}
            placeholder="500"
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="Multi-Process Mode"
        description="Whether to run in multiprocess or multithread mode for parallel inference (default: false)"
      >
        <Form.Item
          name={getFieldName('multi_process')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title="Enable Logging"
        description="Enable detailed logging for debugging (default: false)"
      >
        <Form.Item
          name={getFieldName('log')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>
    </Space>
  )

  return wrapInCard ? (
    <Card size="small" title="Model Performance Settings">
      <div style={{ marginBottom: 8 }}>
        <Text type="secondary">
          Configure performance settings for this specific model. These settings
          will be used when starting the model server.
        </Text>
      </div>
      {content}
    </Card>
  ) : (
    content
  )
}
