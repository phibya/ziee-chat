import {
  App,
  Button,
  Card,
  Divider,
  Flex,
  Form,
  Input,
  InputNumber,
  Layout,
  List,
  Menu,
  Select,
  Space,
  Switch,
  Typography,
} from 'antd'
import { useState, useEffect } from 'react'
import {
  CopyOutlined,
  DeleteOutlined,
  EditOutlined,
  EyeInvisibleOutlined,
  EyeTwoTone,
  PlusOutlined,
  SettingOutlined,
} from '@ant-design/icons'

const { Title, Text } = Typography
const { Sider, Content } = Layout

interface Model {
  id: string
  name: string
  isDeprecated?: boolean
}

interface Provider {
  id: string
  name: string
  icon: string
  enabled: boolean
  apiKey?: string
  baseUrl?: string
  models: Model[]
  settings?: Record<string, any>
}

const PROVIDER_ICONS: Record<string, string> = {
  'llama.cpp': '🦙',
  openai: '🤖',
  anthropic: '🤖',
  cohere: '💜',
  openrouter: '🔀',
  mistral: '🌊',
  groq: '⚡',
  gemini: '💎',
  test: '🧪',
}

const INITIAL_PROVIDERS: Provider[] = [
  {
    id: 'llama.cpp',
    name: 'Llama.cpp',
    icon: '🦙',
    enabled: true,
    models: [
      { id: 'gemma3-1b', name: 'gemma3:1b' },
      {
        id: 'menlo-jan-nano',
        name: 'Menlo:Jan-nano-128k-gguf:jan-nano-128k-iQ4_XS.gguf',
        isDeprecated: true,
      },
    ],
    settings: {
      autoUnloadOldModels: true,
      contextShift: false,
      continuousBatching: false,
      parallelOperations: 1,
      cpuThreads: -1,
      threadsBatch: -1,
      flashAttention: true,
      caching: true,
      kvCacheType: 'q8_0',
      mmap: true,
      huggingFaceAccessToken: '',
    },
  },
  {
    id: 'openai',
    name: 'OpenAI',
    icon: '🤖',
    enabled: true,
    apiKey: '',
    baseUrl: 'https://api.openai.com/v1',
    models: [
      { id: 'gpt-4.5-preview', name: 'gpt-4.5-preview', isDeprecated: true },
      {
        id: 'gpt-4.5-preview-2025-02-27',
        name: 'gpt-4.5-preview-2025-02-27',
        isDeprecated: true,
      },
      { id: 'gpt-4.1', name: 'gpt-4.1', isDeprecated: true },
      {
        id: 'gpt-4.1-2025-04-14',
        name: 'gpt-4.1-2025-04-14',
        isDeprecated: true,
      },
      { id: 'gpt-4o', name: 'gpt-4o', isDeprecated: true },
      { id: 'gpt-4o-mini', name: 'gpt-4o-mini', isDeprecated: true },
      {
        id: 'gpt-4o-2024-05-13',
        name: 'gpt-4o-2024-05-13',
        isDeprecated: true,
      },
      {
        id: 'gpt-4o-2024-08-06',
        name: 'gpt-4o-2024-08-06',
        isDeprecated: true,
      },
      { id: 'gpt-4-turbo', name: 'gpt-4-turbo', isDeprecated: true },
      {
        id: 'gpt-4-turbo-2024-04-09',
        name: 'gpt-4-turbo-2024-04-09',
        isDeprecated: true,
      },
      {
        id: 'gpt-4-0125-preview',
        name: 'gpt-4-0125-preview',
        isDeprecated: true,
      },
      {
        id: 'gpt-4-turbo-preview',
        name: 'gpt-4-turbo-preview',
        isDeprecated: true,
      },
      {
        id: 'gpt-4-1106-preview',
        name: 'gpt-4-1106-preview',
        isDeprecated: true,
      },
      { id: 'gpt-4-vision-preview', name: 'gpt-4-vision-preview' },
      { id: 'gpt-4', name: 'gpt-4', isDeprecated: true },
      { id: 'gpt-4-0314', name: 'gpt-4-0314' },
      { id: 'gpt-4-0613', name: 'gpt-4-0613', isDeprecated: true },
    ],
  },
  {
    id: 'anthropic',
    name: 'Anthropic',
    icon: '🤖',
    enabled: true,
    apiKey: '',
    baseUrl: 'https://api.anthropic.com/v1',
    models: [
      {
        id: 'claude-3-7-sonnet-latest',
        name: 'claude-3-7-sonnet-latest',
        isDeprecated: true,
      },
      {
        id: 'claude-3-7-sonnet-20250219',
        name: 'claude-3-7-sonnet-20250219',
        isDeprecated: true,
      },
      {
        id: 'claude-3-5-sonnet-latest',
        name: 'claude-3-5-sonnet-latest',
        isDeprecated: true,
      },
      {
        id: 'claude-3-5-sonnet-20240620',
        name: 'claude-3-5-sonnet-20240620',
        isDeprecated: true,
      },
      {
        id: 'claude-3-5-haiku-20241022',
        name: 'claude-3-5-haiku-20241022',
        isDeprecated: true,
      },
      {
        id: 'claude-3-opus-20240229',
        name: 'claude-3-opus-20240229',
        isDeprecated: true,
      },
      {
        id: 'claude-3-sonnet-20240229',
        name: 'claude-3-sonnet-20240229',
        isDeprecated: true,
      },
      {
        id: 'claude-3-haiku-20240307',
        name: 'claude-3-haiku-20240307',
        isDeprecated: true,
      },
      { id: 'claude-2.1', name: 'claude-2.1' },
      { id: 'claude-2.0', name: 'claude-2.0' },
      { id: 'claude-instant-1.2', name: 'claude-instant-1.2' },
    ],
  },
  {
    id: 'mistral',
    name: 'Mistral',
    icon: '🌊',
    enabled: true,
    apiKey: '',
    baseUrl: 'https://api.mistral.ai',
    models: [
      { id: 'open-mistral-7b', name: 'open-mistral-7b' },
      { id: 'mistral-tiny-2312', name: 'mistral-tiny-2312' },
      { id: 'open-mixtral-8x7b', name: 'open-mixtral-8x7b' },
      { id: 'mistral-small-2312', name: 'mistral-small-2312' },
      {
        id: 'open-mixtral-8x22b',
        name: 'open-mixtral-8x22b',
        isDeprecated: true,
      },
      {
        id: 'open-mixtral-8x22b-2404',
        name: 'open-mixtral-8x22b-2404',
        isDeprecated: true,
      },
      {
        id: 'mistral-small-latest',
        name: 'mistral-small-latest',
        isDeprecated: true,
      },
      {
        id: 'mistral-small-2402',
        name: 'mistral-small-2402',
        isDeprecated: true,
      },
      { id: 'mistral-medium-latest', name: 'mistral-medium-latest' },
      { id: 'mistral-medium-2312', name: 'mistral-medium-2312' },
      {
        id: 'mistral-large-latest',
        name: 'mistral-large-latest',
        isDeprecated: true,
      },
      {
        id: 'mistral-large-2402',
        name: 'mistral-large-2402',
        isDeprecated: true,
      },
      { id: 'codestral-latest', name: 'codestral-latest' },
      { id: 'codestral-2405', name: 'codestral-2405' },
      {
        id: 'codestral-mamba-2407',
        name: 'codestral-mamba-2407',
        isDeprecated: true,
      },
    ],
  },
  {
    id: 'groq',
    name: 'Groq',
    icon: '⚡',
    enabled: true,
    apiKey: '',
    baseUrl: 'https://api.groq.com/openai/v1',
    models: [
      { id: 'llama-3.3-70b-versatile', name: 'llama-3.3-70b-versatile' },
      { id: 'llama-3.1-8b-instant', name: 'llama-3.1-8b-instant' },
      { id: 'llama3-8b-8192', name: 'llama3-8b-8192' },
      { id: 'llama3-70b-8192', name: 'llama3-70b-8192' },
      { id: 'mixtral-8x7b-32768', name: 'mixtral-8x7b-32768' },
      { id: 'gemma-7b-it', name: 'gemma-7b-it' },
      { id: 'gemma2-9b-it', name: 'gemma2-9b-it' },
    ],
  },
  {
    id: 'gemini',
    name: 'Gemini',
    icon: '💎',
    enabled: true,
    apiKey: '',
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai',
    models: [
      {
        id: 'gemini-2.0-flash-001',
        name: 'gemini-2.0-flash-001',
        isDeprecated: true,
      },
      {
        id: 'gemini-2.0-flash-lite-preview-02-05',
        name: 'gemini-2.0-flash-lite-preview-02-05',
      },
      { id: 'gemini-1.5-pro', name: 'gemini-1.5-pro', isDeprecated: true },
      { id: 'gemini-1.5-flash', name: 'gemini-1.5-flash', isDeprecated: true },
      {
        id: 'gemini-1.5-flash-8b',
        name: 'gemini-1.5-flash-8b',
        isDeprecated: true,
      },
      { id: 'gemini-1.0-pro', name: 'gemini-1.0-pro', isDeprecated: true },
    ],
  },
  {
    id: 'cohere',
    name: 'Cohere',
    icon: '💜',
    enabled: false,
    apiKey: '',
    baseUrl: '',
    models: [],
  },
  {
    id: 'openrouter',
    name: 'OpenRouter',
    icon: '🔀',
    enabled: false,
    apiKey: '',
    baseUrl: '',
    models: [],
  },
  {
    id: 'test',
    name: 'Test',
    icon: '🧪',
    enabled: false,
    apiKey: '',
    baseUrl: '',
    models: [],
  },
]

export function ModelProvidersSettings() {
  const { message } = App.useApp()
  const [providers, setProviders] = useState<Provider[]>(INITIAL_PROVIDERS)
  const [selectedProvider, setSelectedProvider] = useState<string>('llama.cpp')
  const [form] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  const currentProvider = providers.find(p => p.id === selectedProvider)

  const handleProviderToggle = (providerId: string, enabled: boolean) => {
    setProviders(prev =>
      prev.map(p => (p.id === providerId ? { ...p, enabled } : p)),
    )
    message.success(
      `${currentProvider?.name} ${enabled ? 'enabled' : 'disabled'}`,
    )
  }

  const handleFormChange = (changedValues: any) => {
    if (!currentProvider) return

    setProviders(prev =>
      prev.map(p =>
        p.id === selectedProvider ? { ...p, ...changedValues } : p,
      ),
    )
  }

  const handleSettingsChange = (changedValues: any) => {
    if (!currentProvider) return

    setProviders(prev =>
      prev.map(p =>
        p.id === selectedProvider
          ? { ...p, settings: { ...p.settings, ...changedValues } }
          : p,
      ),
    )
  }

  const handleDeleteModel = (modelId: string) => {
    if (!currentProvider) return

    setProviders(prev =>
      prev.map(p =>
        p.id === selectedProvider
          ? { ...p, models: p.models.filter(m => m.id !== modelId) }
          : p,
      ),
    )
    message.success('Model removed')
  }

  const handleAddModel = () => {
    message.info('Add model functionality would be implemented here')
  }

  const copyToClipboard = (text: string) => {
    // eslint-disable-next-line no-undef
    if (typeof navigator !== 'undefined' && navigator.clipboard) {
      // eslint-disable-next-line no-undef
      navigator.clipboard.writeText(text)
      message.success('Copied to clipboard')
    } else {
      message.error('Clipboard not available')
    }
  }

  const menuItems = providers.map(provider => ({
    key: provider.id,
    icon: (
      <span style={{ fontSize: '16px' }}>{PROVIDER_ICONS[provider.id]}</span>
    ),
    label: provider.name,
    style: {
      backgroundColor: provider.id === selectedProvider ? '#f0f0f0' : undefined,
    },
  }))

  menuItems.push({
    key: 'add-provider',
    icon: <PlusOutlined />,
    label: 'Add Provider',
    style: { backgroundColor: undefined },
  })

  const ProviderMenu = () => (
    <Menu
      mode="inline"
      selectedKeys={[selectedProvider]}
      items={menuItems}
      onClick={({ key }) => {
        if (key === 'add-provider') {
          message.info('Add provider functionality would be implemented here')
        } else {
          setSelectedProvider(key)
        }
      }}
      style={{ border: 'none' }}
    />
  )

  const renderProviderSettings = () => {
    if (!currentProvider) return null

    return (
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* Provider Header */}
        <Flex justify="space-between" align="center">
          <Flex align="center" gap="middle">
            <span style={{ fontSize: '24px' }}>
              {PROVIDER_ICONS[currentProvider.id]}
            </span>
            <Title level={3} style={{ margin: 0 }}>
              {currentProvider.name}
            </Title>
          </Flex>
          <Switch
            checked={currentProvider.enabled}
            onChange={enabled =>
              handleProviderToggle(currentProvider.id, enabled)
            }
          />
        </Flex>

        {currentProvider.enabled && (
          <>
            {/* API Configuration */}
            {currentProvider.id !== 'llama.cpp' && (
              <Form
                form={form}
                layout="vertical"
                initialValues={currentProvider}
                onValuesChange={handleFormChange}
              >
                <Card title="API Key">
                  <Text type="secondary">
                    The {currentProvider.name} API uses API keys for
                    authentication. Visit your{' '}
                    <Text type="danger">API Keys</Text> page to retrieve the API
                    key you'll use in your requests.
                  </Text>
                  <Form.Item
                    name="apiKey"
                    style={{ marginBottom: 0, marginTop: 16 }}
                  >
                    <Input.Password
                      placeholder="Insert API Key"
                      iconRender={visible =>
                        visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />
                      }
                      suffix={
                        <Button
                          type="text"
                          icon={<CopyOutlined />}
                          onClick={() =>
                            copyToClipboard(currentProvider.apiKey || '')
                          }
                        />
                      }
                    />
                  </Form.Item>
                </Card>

                <Card title="Base URL">
                  <Text type="secondary">
                    The base{' '}
                    {currentProvider.id === 'gemini' ? 'OpenAI-compatible' : ''}{' '}
                    endpoint to use. See the{' '}
                    <Text type="danger">
                      {currentProvider.name} documentation
                    </Text>{' '}
                    for more information.
                  </Text>
                  <Form.Item
                    name="baseUrl"
                    style={{ marginBottom: 0, marginTop: 16 }}
                  >
                    <Input placeholder="Base URL" />
                  </Form.Item>
                </Card>
              </Form>
            )}

            {/* Models Section */}
            <Card
              title="Models"
              extra={
                <Button
                  type="text"
                  icon={<PlusOutlined />}
                  onClick={handleAddModel}
                />
              }
            >
              {currentProvider.id === 'llama.cpp' && (
                <Flex
                  justify="space-between"
                  align="center"
                  style={{ marginBottom: 16 }}
                >
                  <Text>Import models from your local machine</Text>
                  <Button icon={<PlusOutlined />}>Import</Button>
                </Flex>
              )}

              <List
                dataSource={currentProvider.models}
                renderItem={model => (
                  <List.Item
                    actions={[
                      <Button
                        key="edit"
                        type="text"
                        icon={<EditOutlined />}
                        onClick={() =>
                          message.info(
                            'Edit model functionality would be implemented',
                          )
                        }
                      />,
                      currentProvider.id === 'llama.cpp' &&
                      model.name.includes('gemma3') ? (
                        <Button key="action" type="primary" danger>
                          Stop
                        </Button>
                      ) : currentProvider.id === 'llama.cpp' &&
                        model.isDeprecated ? (
                        <Button key="action" type="primary">
                          Start
                        </Button>
                      ) : null,
                      <Button
                        key="delete"
                        type="text"
                        icon={<DeleteOutlined />}
                        onClick={() => handleDeleteModel(model.id)}
                      />,
                    ].filter(Boolean)}
                  >
                    <List.Item.Meta
                      title={
                        <Flex align="center" gap="small">
                          <Text>{model.name}</Text>
                          {model.isDeprecated && (
                            <span style={{ fontSize: '12px' }}>⚠️</span>
                          )}
                        </Flex>
                      }
                    />
                  </List.Item>
                )}
              />
            </Card>

            {/* Llama.cpp Specific Settings */}
            {currentProvider.id === 'llama.cpp' && currentProvider.settings && (
              <Form
                layout="vertical"
                initialValues={currentProvider.settings}
                onValuesChange={handleSettingsChange}
              >
                <Card title="Configuration">
                  <Space
                    direction="vertical"
                    size="middle"
                    style={{ width: '100%' }}
                  >
                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>Auto-Unload Old Models</Text>
                        <div>
                          <Text type="secondary">
                            Automatically unloads models that are not in use to
                            free up memory. Ensure only one model is loaded at a
                            time.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="autoUnloadOldModels"
                        valuePropName="checked"
                        style={{ margin: 0 }}
                      >
                        <Switch />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>Context Shift</Text>
                        <div>
                          <Text type="secondary">
                            Automatically shifts the context window when the
                            model is unable to process the entire prompt,
                            ensuring that the most relevant information is
                            always included.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="contextShift"
                        valuePropName="checked"
                        style={{ margin: 0 }}
                      >
                        <Switch />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>Continuous Batching</Text>
                        <div>
                          <Text type="secondary">
                            Allows processing prompts in parallel with text
                            generation, which usually improves performance.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="continuousBatching"
                        valuePropName="checked"
                        style={{ margin: 0 }}
                      >
                        <Switch />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>Parallel Operations</Text>
                        <div>
                          <Text type="secondary">
                            Number of prompts that can be processed
                            simultaneously by the model.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="parallelOperations"
                        style={{ margin: 0, width: 100 }}
                      >
                        <InputNumber min={1} max={16} />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>CPU Threads</Text>
                        <div>
                          <Text type="secondary">
                            Number of CPU cores used for model processing when
                            running without GPU.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="cpuThreads"
                        style={{ margin: 0, width: 100 }}
                      >
                        <InputNumber placeholder="-1 (auto)" />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>Threads (Batch)</Text>
                        <div>
                          <Text type="secondary">
                            Number of threads for batch and prompt processing
                            (default: same as Threads).
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="threadsBatch"
                        style={{ margin: 0, width: 100 }}
                      >
                        <InputNumber placeholder="-1 (same as Threads)" />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>Flash Attention</Text>
                        <div>
                          <Text type="secondary">
                            Optimizes memory usage and speeds up model inference
                            using an efficient attention implementation.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="flashAttention"
                        valuePropName="checked"
                        style={{ margin: 0 }}
                      >
                        <Switch />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>Caching</Text>
                        <div>
                          <Text type="secondary">
                            Stores recent prompts and responses to improve speed
                            when similar questions are asked.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="caching"
                        valuePropName="checked"
                        style={{ margin: 0 }}
                      >
                        <Switch />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>KV Cache Type</Text>
                        <div>
                          <Text type="secondary">
                            Controls memory usage and precision trade-off.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="kvCacheType"
                        style={{ margin: 0, width: 100 }}
                      >
                        <Select
                          options={[
                            { value: 'q8_0', label: 'q8_0' },
                            { value: 'q4_0', label: 'q4_0' },
                            { value: 'q4_1', label: 'q4_1' },
                            { value: 'q5_0', label: 'q5_0' },
                            { value: 'q5_1', label: 'q5_1' },
                          ]}
                        />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <Flex justify="space-between" align="center">
                      <div>
                        <Text strong>mmap</Text>
                        <div>
                          <Text type="secondary">
                            Loads model files more efficiently by mapping them
                            to memory, reducing RAM usage.
                          </Text>
                        </div>
                      </div>
                      <Form.Item
                        name="mmap"
                        valuePropName="checked"
                        style={{ margin: 0 }}
                      >
                        <Switch />
                      </Form.Item>
                    </Flex>

                    <Divider style={{ margin: 0 }} />

                    <div>
                      <Text strong>Hugging Face Access Token</Text>
                      <div>
                        <Text type="secondary">
                          Access tokens programmatically authenticate your
                          identity to the Hugging Face Hub, allowing
                          applications to perform specific actions specified by
                          the scope of permissions granted.
                        </Text>
                      </div>
                      <Form.Item
                        name="huggingFaceAccessToken"
                        style={{ marginTop: 8, marginBottom: 0 }}
                      >
                        <Input.Password placeholder="hf_*****************************" />
                      </Form.Item>
                    </div>
                  </Space>
                </Card>
              </Form>
            )}
          </>
        )}
      </Space>
    )
  }

  return (
    <Layout style={{ height: '100%', backgroundColor: 'transparent' }}>
      {/* Desktop Sidebar */}
      {!isMobile && (
        <Sider
          width={200}
          theme="light"
          style={{ backgroundColor: 'transparent' }}
        >
          <div style={{ padding: '16px 0' }}>
            <Title level={4} style={{ margin: '0 16px 16px' }}>
              <SettingOutlined style={{ marginRight: 8 }} />
              Model Providers
            </Title>
            <ProviderMenu />
          </div>
        </Sider>
      )}

      {/* Main Content */}
      <Layout style={{ backgroundColor: 'transparent' }}>
        <Content
          style={{
            padding: isMobile ? '16px' : '24px',
            overflow: 'auto',
          }}
        >
          {renderProviderSettings()}
        </Content>
      </Layout>
    </Layout>
  )
}
