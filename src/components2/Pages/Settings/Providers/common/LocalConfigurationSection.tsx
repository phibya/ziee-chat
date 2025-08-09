import {
  Card,
  Divider,
  Flex,
  Form,
  Input,
  InputNumber,
  Select,
  Switch,
  Typography,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { EyeInvisibleOutlined, EyeTwoTone } from '@ant-design/icons'
import { useEffect, useState } from 'react'

const { Text } = Typography

const KV_CACHE_TYPE_OPTIONS = [
  { value: 'q8_0', label: 'q8_0' },
  { value: 'q4_0', label: 'q4_0' },
  { value: 'q4_1', label: 'q4_1' },
  { value: 'q5_0', label: 'q5_0' },
  { value: 'q5_1', label: 'q5_1' },
]

interface LocalConfigurationSectionProps {
  disabled?: boolean
  useNestedSettings?: boolean
  wrapInCard?: boolean
}

export function LocalConfigurationSection({
  disabled = false,
  useNestedSettings = true,
  wrapInCard = true,
}: LocalConfigurationSectionProps) {
  const { t } = useTranslation()
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
    <Flex vertical className="gap-2 w-full">
      <ResponsiveConfigItem
        title={t('providers.autoUnloadOldModels')}
        description={t('providers.autoUnloadDescription')}
      >
        <Form.Item
          name={getFieldName('autoUnloadOldModels')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.contextShift')}
        description={t('providers.contextShiftDescription')}
      >
        <Form.Item
          name={getFieldName('contextShift')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.continuousBatching')}
        description={t('providers.continuousBatchingDescription')}
      >
        <Form.Item
          name={getFieldName('continuousBatching')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.parallelOperations')}
        description={t('providers.parallelOperationsDescription')}
      >
        <Form.Item
          name={getFieldName('parallelOperations')}
          style={{ margin: 0, width: isMobile ? '100%' : 100 }}
        >
          <InputNumber
            min={1}
            max={16}
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.cpuThreads')}
        description={t('providers.cpuThreadsDescription')}
      >
        <Form.Item
          name={getFieldName('cpuThreads')}
          style={{ margin: 0, width: isMobile ? '100%' : 100 }}
        >
          <InputNumber
            placeholder={t('providers.autoPlaceholder')}
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.threadsBatch')}
        description={t('providers.threadsBatchDescription')}
      >
        <Form.Item
          name={getFieldName('threadsBatch')}
          style={{ margin: 0, width: isMobile ? '100%' : 100 }}
        >
          <InputNumber
            placeholder={t('providers.sameAsThreadsPlaceholder')}
            style={{ width: '100%' }}
            disabled={disabled}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.flashAttention')}
        description={t('providers.flashAttentionDescription')}
      >
        <Form.Item
          name={getFieldName('flashAttention')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.caching')}
        description={t('providers.cachingDescription')}
      >
        <Form.Item
          name={getFieldName('caching')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.kvCacheType')}
        description={t('providers.kvCacheTypeDescription')}
      >
        <Form.Item
          name={getFieldName('kvCacheType')}
          style={{ margin: 0, width: isMobile ? '100%' : 100 }}
        >
          <Select
            style={{ width: '100%' }}
            disabled={disabled}
            options={KV_CACHE_TYPE_OPTIONS}
          />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <ResponsiveConfigItem
        title={t('providers.mmap')}
        description={t('providers.mmapDescription')}
      >
        <Form.Item
          name={getFieldName('mmap')}
          valuePropName="checked"
          style={{ margin: 0 }}
        >
          <Switch disabled={disabled} />
        </Form.Item>
      </ResponsiveConfigItem>

      <Divider style={{ margin: 0 }} />

      <div>
        <Text strong>Hugging Face Access Token</Text>
        <div>
          <Text type="secondary">
            Access tokens programmatically authenticate your identity to the
            Hugging Face Hub, allowing applications to perform specific actions
            specified by the scope of permissions granted.
          </Text>
        </div>
        <Form.Item
          name={getFieldName('huggingFaceAccessToken')}
          style={{ marginTop: 8, marginBottom: 0 }}
        >
          <Input.Password
            placeholder={t('providers.huggingfaceTokenPlaceholder')}
            style={{ width: '100%' }}
            disabled={disabled}
            iconRender={visible =>
              visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
            }
          />
        </Form.Item>
      </div>
    </Flex>
  )

  return wrapInCard ? (
    <Card title={t('providers.localConfiguration')}>{content}</Card>
  ) : (
    content
  )
}
