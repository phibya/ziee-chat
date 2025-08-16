import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Card,
  Form,
  Input,
  InputNumber,
  Switch,
  Button,
  Typography,
  Alert,
  Flex,
  App,
  Select,
  Tag,
  Empty,
  Divider,
  Checkbox,
} from 'antd'
import {
  LockOutlined,
  PlayCircleOutlined,
  StopOutlined,
  PlusOutlined,
  DeleteOutlined,
  EditOutlined,
  FileTextOutlined,
} from '@ant-design/icons'
import { SettingsPageContainer } from './common/SettingsPageContainer'
import { Permission, usePermissions } from '../../../permissions'
import { Drawer } from '../../Common/Drawer'
import { Stores } from '../../../store'
import { isTauriView } from '../../../api/core'
import {
  initializeApiProxyServerData,
  updateApiProxyServerConfig,
  startApiProxyServer,
  stopApiProxyServer,
  addModelToApiProxyServer,
  updateApiProxyServerModel,
  removeModelFromApiProxyServer,
  addTrustedHostToApiProxyServer,
  updateApiProxyServerTrustedHost,
  removeTrustedHostFromApiProxyServer,
} from '../../../store/admin/apiProxyServer'
import type {
  CreateApiProxyServerModelRequest,
  UpdateApiProxyServerModelRequest,
  CreateTrustedHostRequest,
  UpdateTrustedHostRequest,
  ApiProxyServerModel,
  ApiProxyServerTrustedHost,
} from '../../../types/api'

const { Text } = Typography

export function ApiProxyServerSettings() {
  const { t } = useTranslation()
  const { hasPermission } = usePermissions()

  // Permission check
  const canRead = hasPermission(Permission.config.apiProxyServer?.read)

  if (!canRead) {
    return (
      <SettingsPageContainer title={t('apiProxyServer.title')}>
        <Card>
          <Text type="secondary">{t('permissions.insufficient')}</Text>
        </Card>
      </SettingsPageContainer>
    )
  }

  // Store data
  const { error, initialized } = Stores.AdminApiProxyServer

  // Load data on mount
  useEffect(() => {
    if (!initialized) {
      initializeApiProxyServerData().catch(console.error)
    }
  }, [initialized])

  return (
    <SettingsPageContainer
      title={t('apiProxyServer.title')}
      subtitle={t('apiProxyServer.subtitle')}
    >
      <div className="flex flex-col gap-3 flex-wrap w-full">
        {error && (
          <Alert
            message={t('apiProxyServer.error')}
            description={error}
            type="error"
            showIcon
            closable
          />
        )}

        {/* Configuration Card */}
        <ServerConfigurationCard />

        {/* Model Selection Card */}
        <ModelSelectionCard />

        {/* Trusted Hosts Card */}
        <TrustedHostsCard />

        {/* Server Control Card */}
        <ServerControlCard />
      </div>
    </SettingsPageContainer>
  )
}

// Server Configuration Card Component
function ServerConfigurationCard() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const { hasPermission } = usePermissions()

  // Permission check
  const canEdit = hasPermission(Permission.config.apiProxyServer?.edit)

  // Store data
  const { config, loadingConfig } = Stores.AdminApiProxyServer

  const handleConfigSave = async (values: any) => {
    try {
      console.log('Saving proxy config with values:', values)
      await updateApiProxyServerConfig(values)
      message.success(t('apiProxyServer.configurationSaved'))
    } catch (error) {
      console.error('Failed to save proxy config:', error)
      message.error(t('apiProxyServer.configurationError'))
    }
  }

  // Update form when config changes
  useEffect(() => {
    if (config) {
      form.setFieldsValue({
        address: config.address,
        port: config.port,
        prefix: config.prefix,
        api_key: config.api_key,
        allow_cors: config.allow_cors,
        log_level: config.log_level,
      })
    }
  }, [config, form])

  return (
    <Card title={t('apiProxyServer.configuration')}>
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          address: '127.0.0.1',
          port: 8080,
          prefix: '/v1',
          api_key: '',
          allow_cors: false,
          log_level: 'info',
          ...config,
        }}
        onFinish={handleConfigSave}
        disabled={!canEdit}
      >
        {/* Server Address */}
        <Form.Item
          name="address"
          label={t('apiProxyServer.address')}
          tooltip={t('apiProxyServer.addressTooltip')}
          rules={[
            { required: true, message: t('apiProxyServer.addressRequired') },
          ]}
        >
          <Select
            placeholder={t('apiProxyServer.addressPlaceholder')}
            options={[
              { label: '127.0.0.1 (localhost only)', value: '127.0.0.1' },
              { label: '0.0.0.0 (all interfaces)', value: '0.0.0.0' },
            ]}
          />
        </Form.Item>

        {/* Server Port */}
        <Form.Item
          name="port"
          label={t('apiProxyServer.port')}
          tooltip={t('apiProxyServer.portTooltip')}
          rules={[
            { required: true, message: t('apiProxyServer.portRequired') },
            {
              type: 'number',
              min: 1,
              max: 65535,
              message: t('apiProxyServer.portRange'),
            },
          ]}
        >
          <InputNumber
            placeholder="8080"
            min={1}
            max={65535}
            style={{ width: '100%' }}
          />
        </Form.Item>

        {/* URL Prefix */}
        <Form.Item
          name="prefix"
          label={t('apiProxyServer.prefix')}
          tooltip={t('apiProxyServer.prefixTooltip')}
          rules={[
            { required: true, message: t('apiProxyServer.prefixRequired') },
          ]}
        >
          <Input placeholder="/v1" />
        </Form.Item>

        {/* API Key */}
        <Form.Item
          name="api_key"
          label={t('apiProxyServer.apiKey')}
          tooltip={t('apiProxyServer.apiKeyTooltip')}
        >
          <Input.Password
            placeholder={t('apiProxyServer.apiKeyPlaceholder')}
            prefix={<LockOutlined />}
          />
        </Form.Item>

        {/* CORS Toggle */}
        <div className="flex justify-between items-center mb-4">
          <div style={{ flex: 1, marginRight: 16 }}>
            <Text strong>{t('apiProxyServer.allowCors')}</Text>
            <br />
            <Text type="secondary">{t('apiProxyServer.allowCorsDesc')}</Text>
          </div>
          <Form.Item
            name="allow_cors"
            valuePropName="checked"
            style={{ margin: 0 }}
          >
            <Switch disabled={!canEdit} />
          </Form.Item>
        </div>

        {/* Log Level */}
        <Form.Item
          name="log_level"
          label={t('apiProxyServer.logLevel')}
          tooltip={t('apiProxyServer.logLevelTooltip')}
        >
          <Select
            placeholder={t('apiProxyServer.logLevelPlaceholder')}
            options={[
              { label: 'Error', value: 'error' },
              { label: 'Warn', value: 'warn' },
              { label: 'Info', value: 'info' },
              { label: 'Debug', value: 'debug' },
              { label: 'Trace', value: 'trace' },
            ]}
          />
        </Form.Item>

        {canEdit && (
          <Form.Item>
            <Button type="primary" htmlType="submit" loading={loadingConfig}>
              {t('common.save')}
            </Button>
          </Form.Item>
        )}
      </Form>
    </Card>
  )
}

// Model Selection Card Component
function ModelSelectionCard() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()
  const [addModelDrawerOpen, setAddModelDrawerOpen] = useState(false)
  const [editModelDrawerOpen, setEditModelDrawerOpen] = useState(false)
  const [editingModelId, setEditingModelId] = useState<string | null>(null)

  // Permission check
  const canEdit = hasPermission(Permission.config.apiProxyServer?.edit)

  // Store data
  const { models } = Stores.AdminApiProxyServer
  const allProviders = Stores.AdminProviders.providers || []
  const availableModels = allProviders
    .flatMap(provider => (provider.models || []).map(m => ({ ...m, provider })))
    .filter(model => !models.find(pm => pm.model_id === model.id))

  const handleAddModel = () => {
    setAddModelDrawerOpen(true)
  }

  const handleEditModel = (modelId: string) => {
    setEditingModelId(modelId)
    setEditModelDrawerOpen(true)
  }

  const handleAddModelSubmit = async (
    data: CreateApiProxyServerModelRequest,
  ) => {
    try {
      await addModelToApiProxyServer(data)
      message.success(t('apiProxyServer.modelAdded'))
      setAddModelDrawerOpen(false)
    } catch (_error) {
      message.error(t('apiProxyServer.modelAddError'))
    }
  }

  const handleUpdateModelSubmit = async (
    modelId: string,
    updates: UpdateApiProxyServerModelRequest,
  ) => {
    try {
      await updateApiProxyServerModel(modelId, updates)
      message.success(t('apiProxyServer.modelUpdated'))
      setEditModelDrawerOpen(false)
      setEditingModelId(null)
    } catch (_error) {
      message.error(t('apiProxyServer.modelUpdateError'))
    }
  }

  const handleRemoveModel = async (modelId: string) => {
    try {
      await removeModelFromApiProxyServer(modelId)
      message.success(t('apiProxyServer.modelRemoved'))
    } catch (_error) {
      message.error(t('apiProxyServer.modelRemoveError'))
    }
  }

  const handleUpdateModel = async (
    modelId: string,
    updates: UpdateApiProxyServerModelRequest,
  ) => {
    try {
      await updateApiProxyServerModel(modelId, updates)
      message.success(t('apiProxyServer.modelUpdated'))
    } catch (_error) {
      message.error(t('apiProxyServer.modelUpdateError'))
    }
  }

  return (
    <>
      <Card
        title={t('apiProxyServer.modelSelection')}
        extra={
          canEdit && (
            <Button
              type="text"
              icon={<PlusOutlined />}
              onClick={handleAddModel}
              disabled={availableModels.length === 0}
            >
              {t('apiProxyServer.addModel')}
            </Button>
          )
        }
      >
        {models.length === 0 ? (
          <Empty description={t('apiProxyServer.noModelsConfigured')} />
        ) : (
          <div className="space-y-3">
            {models.map((proxyModel, index) => (
              <div key={proxyModel.id}>
                <ModelItem
                  proxyModel={proxyModel}
                  canEdit={canEdit}
                  onUpdate={handleUpdateModel}
                  onRemove={handleRemoveModel}
                  onEdit={handleEditModel}
                />
                {index < models.length - 1 && <Divider className="my-0" />}
              </div>
            ))}
          </div>
        )}
      </Card>

      {/* Add Model Drawer */}
      <AddModelDrawer
        open={addModelDrawerOpen}
        onClose={() => setAddModelDrawerOpen(false)}
        availableModels={availableModels}
        onAdd={handleAddModelSubmit}
      />

      {/* Edit Model Drawer */}
      <EditModelDrawer
        open={editModelDrawerOpen}
        onClose={() => {
          setEditModelDrawerOpen(false)
          setEditingModelId(null)
        }}
        modelId={editingModelId}
        models={models}
        onUpdate={handleUpdateModelSubmit}
      />
    </>
  )
}

// Model Item Component
interface ModelItemProps {
  proxyModel: ApiProxyServerModel
  canEdit: boolean
  onUpdate: (
    modelId: string,
    updates: UpdateApiProxyServerModelRequest,
  ) => Promise<void>
  onRemove: (modelId: string) => Promise<void>
  onEdit: (modelId: string) => void
}

function ModelItem({
  proxyModel,
  canEdit,
  onUpdate,
  onRemove,
  onEdit,
}: ModelItemProps) {
  const { t } = useTranslation()

  // Find the actual model details
  const allProviders = Stores.AdminProviders.providers || []
  const model = allProviders
    .flatMap(provider => (provider.models || []).map(m => ({ ...m, provider })))
    .find(m => m.id === proxyModel.model_id)

  return (
    <div className="flex items-start gap-3 flex-wrap">
      <div className="flex-1">
        <div className="flex items-center gap-2 mb-2 flex-wrap-reverse">
          <div className="flex-1 min-w-48">
            <Flex gap={2} align="center">
              <Text className="font-medium">
                {proxyModel.alias_id ||
                  model?.alias ||
                  model?.name ||
                  'Unknown Model'}
              </Text>
              {proxyModel.is_default && (
                <Tag color="blue">{t('apiProxyServer.defaultModel')}</Tag>
              )}
            </Flex>
          </div>

          <div className="flex gap-1 items-center justify-end">
            {/* Enable/Disable Switch */}
            <Switch
              className="!mr-2"
              checked={proxyModel.enabled}
              onChange={checked =>
                onUpdate(proxyModel.model_id, { enabled: checked })
              }
              disabled={!canEdit}
            />

            {canEdit && (
              <>
                <Button
                  type="text"
                  icon={<EditOutlined />}
                  onClick={() => onEdit(proxyModel.model_id)}
                />
                <Button
                  type="text"
                  icon={<DeleteOutlined />}
                  onClick={() => onRemove(proxyModel.model_id)}
                />
              </>
            )}
          </div>
        </div>

        <div className="space-y-1">
          <Text type="secondary" className="text-xs block">
            Model ID: {model?.name || proxyModel.model_id}
          </Text>
          {proxyModel.alias_id && (
            <Text type="secondary" className="text-xs block">
              Alias: {proxyModel.alias_id}
            </Text>
          )}
          {model?.description && (
            <Text type="secondary" className="block">
              {model.description}
            </Text>
          )}
          {model?.provider && (
            <Text type="secondary" className="text-xs block">
              Provider: {model.provider.name} ({model.provider.type})
            </Text>
          )}
        </div>
      </div>
    </div>
  )
}

// Trusted Hosts Card Component
function TrustedHostsCard() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()
  const [addHostDrawerOpen, setAddHostDrawerOpen] = useState(false)
  const [editHostDrawerOpen, setEditHostDrawerOpen] = useState(false)
  const [editingHostId, setEditingHostId] = useState<string | null>(null)

  // Permission check
  const canEdit = hasPermission(Permission.config.apiProxyServer?.edit)

  // Store data
  const { trustedHosts } = Stores.AdminApiProxyServer

  const handleAddHost = () => {
    setAddHostDrawerOpen(true)
  }

  const handleEditHost = (hostId: string) => {
    setEditingHostId(hostId)
    setEditHostDrawerOpen(true)
  }

  const handleAddHostSubmit = async (data: CreateTrustedHostRequest) => {
    try {
      await addTrustedHostToApiProxyServer(data)
      message.success(t('apiProxyServer.hostAdded'))
      setAddHostDrawerOpen(false)
    } catch (_error) {
      message.error(t('apiProxyServer.hostAddError'))
    }
  }

  const handleUpdateHostSubmit = async (
    hostId: string,
    updates: UpdateTrustedHostRequest,
  ) => {
    try {
      await updateApiProxyServerTrustedHost(hostId, updates)
      message.success(t('apiProxyServer.hostUpdated'))
      setEditHostDrawerOpen(false)
      setEditingHostId(null)
    } catch (_error) {
      message.error(t('apiProxyServer.hostUpdateError'))
    }
  }

  const handleRemoveHost = async (hostId: string) => {
    try {
      await removeTrustedHostFromApiProxyServer(hostId)
      message.success(t('apiProxyServer.hostRemoved'))
    } catch (_error) {
      message.error(t('apiProxyServer.hostRemoveError'))
    }
  }

  const handleUpdateHost = async (
    hostId: string,
    updates: UpdateTrustedHostRequest,
  ) => {
    try {
      await updateApiProxyServerTrustedHost(hostId, updates)
      message.success(t('apiProxyServer.hostUpdated'))
    } catch (_error) {
      message.error(t('apiProxyServer.hostUpdateError'))
    }
  }

  return (
    <>
      <Card
        title={t('apiProxyServer.trustedHosts')}
        extra={
          canEdit && (
            <Button type="text" icon={<PlusOutlined />} onClick={handleAddHost}>
              {t('apiProxyServer.addHost')}
            </Button>
          )
        }
      >
        {trustedHosts.length === 0 ? (
          <Empty description={t('apiProxyServer.noHostsConfigured')} />
        ) : (
          <div className="space-y-0">
            {trustedHosts.map((host, index) => (
              <div key={host.id}>
                <TrustedHostItem
                  host={host}
                  canEdit={canEdit}
                  onUpdate={handleUpdateHost}
                  onRemove={handleRemoveHost}
                  onEdit={handleEditHost}
                />
                {index < trustedHosts.length - 1 && (
                  <Divider className="!my-1" />
                )}
              </div>
            ))}
          </div>
        )}
      </Card>

      {/* Add Host Drawer */}
      <AddHostDrawer
        open={addHostDrawerOpen}
        onClose={() => setAddHostDrawerOpen(false)}
        onAdd={handleAddHostSubmit}
      />

      {/* Edit Host Drawer */}
      <EditHostDrawer
        open={editHostDrawerOpen}
        onClose={() => {
          setEditHostDrawerOpen(false)
          setEditingHostId(null)
        }}
        hostId={editingHostId}
        hosts={trustedHosts}
        onUpdate={handleUpdateHostSubmit}
      />
    </>
  )
}

// Trusted Host Item Component
interface TrustedHostItemProps {
  host: ApiProxyServerTrustedHost
  canEdit: boolean
  onUpdate: (hostId: string, updates: UpdateTrustedHostRequest) => Promise<void>
  onRemove: (hostId: string) => Promise<void>
  onEdit: (hostId: string) => void
}

function TrustedHostItem({
  host,
  canEdit,
  onUpdate,
  onRemove,
  onEdit,
}: TrustedHostItemProps) {
  return (
    <div className="flex items-start gap-3 flex-wrap">
      <div className="flex-1">
        <div className="flex items-center gap-2 flex-wrap-reverse">
          <div className="flex-1 min-w-48">
            <Text className="font-medium">{host.host}</Text>
          </div>

          <div className="flex gap-1 items-center justify-end">
            {/* Enable/Disable Switch */}
            <Switch
              className="!mr-2"
              checked={host.enabled}
              onChange={checked => onUpdate(host.id, { enabled: checked })}
              disabled={!canEdit}
            />

            {canEdit && (
              <>
                <Button
                  type="text"
                  icon={<EditOutlined />}
                  onClick={() => onEdit(host.id)}
                />
                <Button
                  type="text"
                  icon={<DeleteOutlined />}
                  onClick={() => onRemove(host.id)}
                />
              </>
            )}
          </div>
        </div>

        {host.description && (
          <Text type="secondary" className="block">
            {host.description}
          </Text>
        )}
      </div>
    </div>
  )
}

// Server Control Card Component
function ServerControlCard() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  // Permission check
  const canEdit = hasPermission(Permission.config.apiProxyServer?.edit)

  // Store data
  const { config, status, loadingStatus, models } = Stores.AdminApiProxyServer
  console.log({ config })

  // Check if server is properly configured
  const isConfigured = Boolean(
    config?.address && config?.port && config?.prefix,
  )

  const handleStart = async () => {
    try {
      await startApiProxyServer()
      message.success(t('apiProxyServer.serverStarted'))
    } catch (_error) {
      message.error(t('apiProxyServer.serverStartError'))
    }
  }

  const handleStop = async () => {
    try {
      await stopApiProxyServer()
      message.success(t('apiProxyServer.serverStopped'))
    } catch (_error) {
      message.error(t('apiProxyServer.serverStopError'))
    }
  }


  const handleOpenLogMonitor = async () => {
    if (isTauriView) {
      try {
        const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')
        const existingWindow = await WebviewWindow.getByLabel('log-monitor')
        if (existingWindow) {
          await existingWindow.setFocus()
        }
      } catch {
        try {
          const { WebviewWindow } = await import(
            '@tauri-apps/api/webviewWindow'
          )
          new WebviewWindow('log-monitor', {
            url: '/api-proxy-log-monitor',
            title: t('apiProxyServer.logMonitor'),
            width: 800,
            height: 600,
          })
        } catch (error) {
          console.error('Failed to open log monitor window:', error)
          message.error(t('apiProxyServer.logMonitorError'))
        }
      }
    } else {
      // Use browser popup for web app
      const popup = window.open(
        window.location.origin + '/api-proxy-log-monitor',
        'api-proxy-log-monitor', // Using same name will focus existing popup
        'width=800,height=600,scrollbars=yes,resizable=yes,menubar=no,toolbar=no',
      )
      if (popup) {
        popup.focus()
      } else {
        message.error('Please allow popups for this website')
      }
    }
  }

  // Modified title with Log Monitor button (following HardwareSettings pattern)
  const titleWithButton = (
    <div className="flex items-center justify-between w-full">
      <span>{t('apiProxyServer.serverControl')}</span>
      <Button icon={<FileTextOutlined />} onClick={handleOpenLogMonitor}>
        {t('apiProxyServer.logMonitor')}
      </Button>
    </div>
  )

  return (
    <Card title={titleWithButton}>
      <div className="flex flex-col gap-3 flex-wrap w-full">
        {/* Status Display */}
        <Flex justify="space-between" align="center">
          <Text strong>{t('apiProxyServer.status')}:</Text>
          <Text type={status?.running ? 'success' : 'secondary'}>
            {status?.running
              ? t('apiProxyServer.running')
              : t('apiProxyServer.stopped')}
          </Text>
        </Flex>

        {/* Server URL */}
        {status?.running && config && (
          <Flex justify="space-between" align="center">
            <Text strong>{t('apiProxyServer.serverUrl')}:</Text>
            <div className="flex gap-3 flex-wrap">
              <Text code copyable>
                http://{config.address}:{config.port}
                {config.prefix}
              </Text>
            </div>
          </Flex>
        )}

        {/* Active Models Count */}
        {status?.running && (
          <Flex justify="space-between" align="center">
            <Text strong>{t('apiProxyServer.activeModels')}:</Text>
            <Text>{status.active_models || 0}</Text>
          </Flex>
        )}

        {/* Control Buttons */}
        {canEdit && (
          <div className="flex gap-3 flex-wrap">
            {!status?.running ? (
              <Button
                type="primary"
                icon={<PlayCircleOutlined />}
                onClick={handleStart}
                loading={loadingStatus}
                disabled={!isConfigured}
              >
                {t('apiProxyServer.startServer')}
              </Button>
            ) : (
              <Button
                danger
                icon={<StopOutlined />}
                onClick={handleStop}
                loading={loadingStatus}
              >
                {t('apiProxyServer.stopServer')}
              </Button>
            )}
          </div>
        )}

        {/* Configuration Warnings */}
        {!isConfigured && (
          <Alert
            message={t('apiProxyServer.configurationIncomplete')}
            description={t('apiProxyServer.configurationIncompleteDesc')}
            type="warning"
            showIcon
          />
        )}

        {/* No Models Warning when server is running */}
        {status?.running && models.length === 0 && (
          <Alert
            message={t('apiProxyServer.noModelsRunning')}
            description={t('apiProxyServer.noModelsRunningDesc')}
            type="info"
            showIcon
          />
        )}
      </div>
    </Card>
  )
}

// Drawer Components
interface AddModelDrawerProps {
  open: boolean
  onClose: () => void
  availableModels: any[]
  onAdd: (data: CreateApiProxyServerModelRequest) => Promise<any>
}

function AddModelDrawer({
  open,
  onClose,
  availableModels,
  onAdd,
}: AddModelDrawerProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      await onAdd(values)
      message.success(t('apiProxyServer.modelAdded'))
      form.resetFields()
      onClose()
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  return (
    <Drawer
      title={t('apiProxyServer.addModelToProxy')}
      open={open}
      onClose={onClose}
      width={400}
      footer={[
        <Button key="cancel" onClick={onClose}>
          {t('common.cancel')}
        </Button>,
        <Button key="submit" type="primary" onClick={handleSubmit}>
          {t('common.add')}
        </Button>,
      ]}
    >
      <Form form={form} layout="vertical">
        <Form.Item
          name="model_id"
          label={t('apiProxyServer.selectModel')}
          rules={[
            { required: true, message: t('apiProxyServer.modelRequired') },
          ]}
        >
          <Select
            placeholder={t('apiProxyServer.selectModelPlaceholder')}
            showSearch
            filterOption={(input, option) =>
              (option?.label ?? '').toLowerCase().includes(input.toLowerCase())
            }
            options={availableModels.map(model => ({
              label: `${model.alias} (${model.provider?.name})`,
              value: model.id,
            }))}
          />
        </Form.Item>

        <Form.Item
          name="alias_id"
          label={t('apiProxyServer.alias')}
          tooltip={t('apiProxyServer.aliasTooltip')}
        >
          <Input placeholder={t('apiProxyServer.aliasPlaceholder')} />
        </Form.Item>

        <Form.Item name="enabled" valuePropName="checked" initialValue={true}>
          <Checkbox>{t('apiProxyServer.enabledByDefault')}</Checkbox>
        </Form.Item>

        <Form.Item name="is_default" valuePropName="checked">
          <Checkbox>{t('apiProxyServer.setAsDefault')}</Checkbox>
        </Form.Item>
      </Form>
    </Drawer>
  )
}

interface EditModelDrawerProps {
  open: boolean
  onClose: () => void
  modelId: string | null
  models: ApiProxyServerModel[]
  onUpdate: (
    modelId: string,
    updates: UpdateApiProxyServerModelRequest,
  ) => Promise<any>
}

function EditModelDrawer({
  open,
  onClose,
  modelId,
  models,
  onUpdate,
}: EditModelDrawerProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  const model = models.find(m => m.model_id === modelId)

  useEffect(() => {
    if (model) {
      form.setFieldsValue({
        alias_id: model.alias_id,
        enabled: model.enabled,
        is_default: model.is_default,
      })
    }
  }, [model, form])

  const handleSubmit = async () => {
    if (!modelId) return

    try {
      const values = await form.validateFields()
      await onUpdate(modelId, values)
      message.success(t('apiProxyServer.modelUpdated'))
      onClose()
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  return (
    <Drawer
      title={t('apiProxyServer.editModel')}
      open={open}
      onClose={onClose}
      width={400}
      footer={[
        <Button key="cancel" onClick={onClose}>
          {t('common.cancel')}
        </Button>,
        <Button key="submit" type="primary" onClick={handleSubmit}>
          {t('common.save')}
        </Button>,
      ]}
    >
      <Form form={form} layout="vertical">
        <Form.Item
          name="alias_id"
          label={t('apiProxyServer.alias')}
          tooltip={t('apiProxyServer.aliasTooltip')}
        >
          <Input placeholder={t('apiProxyServer.aliasPlaceholder')} />
        </Form.Item>

        <Form.Item name="enabled" valuePropName="checked">
          <Checkbox>{t('apiProxyServer.enabled')}</Checkbox>
        </Form.Item>

        <Form.Item name="is_default" valuePropName="checked">
          <Checkbox>{t('apiProxyServer.setAsDefault')}</Checkbox>
        </Form.Item>
      </Form>
    </Drawer>
  )
}

interface AddHostDrawerProps {
  open: boolean
  onClose: () => void
  onAdd: (data: CreateTrustedHostRequest) => Promise<any>
}

function AddHostDrawer({ open, onClose, onAdd }: AddHostDrawerProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      await onAdd(values)
      message.success(t('apiProxyServer.hostAdded'))
      form.resetFields()
      onClose()
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  return (
    <Drawer
      title={t('apiProxyServer.addTrustedHost')}
      open={open}
      onClose={onClose}
      width={400}
      footer={[
        <Button key="cancel" onClick={onClose}>
          {t('common.cancel')}
        </Button>,
        <Button key="submit" type="primary" onClick={handleSubmit}>
          {t('common.add')}
        </Button>,
      ]}
    >
      <Form form={form} layout="vertical">
        <Form.Item
          name="host"
          label={t('apiProxyServer.hostAddress')}
          rules={[
            { required: true, message: t('apiProxyServer.hostRequired') },
            {
              validator: (_, value) => {
                if (!value) return Promise.resolve()

                // Basic validation for IP addresses, domains, and CIDR
                const ipv4Regex =
                  /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(?:\/(?:3[0-2]|[12]?[0-9]))?$/
                const ipv6Regex = /^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$/
                const domainRegex =
                  /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/

                if (
                  ipv4Regex.test(value) ||
                  ipv6Regex.test(value) ||
                  domainRegex.test(value)
                ) {
                  return Promise.resolve()
                }

                return Promise.reject(
                  new Error(t('apiProxyServer.invalidHost')),
                )
              },
            },
          ]}
          tooltip={t('apiProxyServer.hostTooltip')}
        >
          <Input placeholder={t('apiProxyServer.hostPlaceholder')} />
        </Form.Item>

        <Form.Item name="description" label={t('apiProxyServer.description')}>
          <Input.TextArea
            placeholder={t('apiProxyServer.descriptionPlaceholder')}
            rows={3}
          />
        </Form.Item>

        <Form.Item name="enabled" valuePropName="checked" initialValue={true}>
          <Checkbox>{t('apiProxyServer.enabledByDefault')}</Checkbox>
        </Form.Item>
      </Form>
    </Drawer>
  )
}

interface EditHostDrawerProps {
  open: boolean
  onClose: () => void
  hostId: string | null
  hosts: ApiProxyServerTrustedHost[]
  onUpdate: (hostId: string, updates: UpdateTrustedHostRequest) => Promise<any>
}

function EditHostDrawer({
  open,
  onClose,
  hostId,
  hosts,
  onUpdate,
}: EditHostDrawerProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  const host = hosts.find(h => h.id === hostId)

  useEffect(() => {
    if (host) {
      form.setFieldsValue({
        host: host.host,
        description: host.description,
        enabled: host.enabled,
      })
    }
  }, [host, form])

  const handleSubmit = async () => {
    if (!hostId) return

    try {
      const values = await form.validateFields()
      await onUpdate(hostId, values)
      message.success(t('apiProxyServer.hostUpdated'))
      onClose()
    } catch (error) {
      console.error('Form validation failed:', error)
    }
  }

  return (
    <Drawer
      title={t('apiProxyServer.editHost')}
      open={open}
      onClose={onClose}
      width={400}
      footer={[
        <Button key="cancel" onClick={onClose}>
          {t('common.cancel')}
        </Button>,
        <Button key="submit" type="primary" onClick={handleSubmit}>
          {t('common.save')}
        </Button>,
      ]}
    >
      <Form form={form} layout="vertical">
        <Form.Item
          name="host"
          label={t('apiProxyServer.hostAddress')}
          rules={[
            { required: true, message: t('apiProxyServer.hostRequired') },
          ]}
          tooltip={t('apiProxyServer.hostTooltip')}
        >
          <Input placeholder={t('apiProxyServer.hostPlaceholder')} />
        </Form.Item>

        <Form.Item name="description" label={t('apiProxyServer.description')}>
          <Input.TextArea
            placeholder={t('apiProxyServer.descriptionPlaceholder')}
            rows={3}
          />
        </Form.Item>

        <Form.Item name="enabled" valuePropName="checked">
          <Checkbox>{t('apiProxyServer.enabled')}</Checkbox>
        </Form.Item>
      </Form>
    </Drawer>
  )
}
