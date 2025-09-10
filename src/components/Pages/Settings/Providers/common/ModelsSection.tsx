import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  UploadOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Divider,
  Dropdown,
  Empty,
  Flex,
  Spin,
  Switch,
  Typography,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import {
  deleteExistingModel,
  disableModelFromUse,
  enableModelForUse,
  openAddLocalModelDownloadDrawer,
  openAddLocalModelUploadDrawer,
  openAddRemoteModelDrawer,
  openEditLocalModelDrawer,
  openEditRemoteModelDrawer,
  startModelExecution,
  stopModelExecution,
  Stores,
  updateModelProvider,
} from '../../../../../store'
import { Model } from '../../../../../types'

const { Text } = Typography

export function ModelsSection() {
  const { t } = useTranslation()
  const { message, modal } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()

  // Store data
  const { modelsLoading, modelOperations } = Stores.AdminProviders

  // Get current provider and its models
  const currentProvider = Stores.AdminProviders.providers.find(
    p => p.id === providerId,
  )
  const models = currentProvider?.models || []
  const loading = modelsLoading[providerId!] || false

  const handleToggleModel = async (modelId: string, enabled: boolean) => {
    if (!currentProvider) return

    try {
      if (enabled) {
        await enableModelForUse(modelId)
      } else {
        await disableModelFromUse(modelId)
      }

      // Check if this was the last enabled model being disabled
      if (!enabled) {
        const remainingEnabledModels = models.filter(
          m => m.id !== modelId && m.enabled !== false,
        )

        // If no models remain enabled and provider is currently enabled, disable the provider
        if (remainingEnabledModels.length === 0 && currentProvider.enabled) {
          try {
            await updateModelProvider(currentProvider.id, { enabled: false })
            const modelName =
              models.find(m => m.id === modelId)?.name || 'Model'
            message.success(
              `${modelName} disabled. ${currentProvider.name} provider disabled as no models remain active.`,
            )
          } catch (providerError) {
            console.error('Failed to disable provider:', providerError)
            const modelName =
              models.find(m => m.id === modelId)?.name || 'Model'
            message.warning(
              `${modelName} disabled, but failed to disable provider automatically`,
            )
          }
        } else {
          const modelName = models.find(m => m.id === modelId)?.name || 'Model'
          message.success(`${modelName} ${enabled ? 'enabled' : 'disabled'}`)
        }
      } else {
        const modelName = models.find(m => m.id === modelId)?.name || 'Model'
        message.success(`${modelName} ${enabled ? 'enabled' : 'disabled'}`)
      }
    } catch (error) {
      console.error('Failed to toggle model:', error)
      // Error is handled by the store
    }
  }

  const handleDeleteModel = async (modelId: string) => {
    if (!currentProvider) return

    try {
      await deleteExistingModel(modelId)
      message.success(t('providers.modelDeleted'))
    } catch (error) {
      console.error('Failed to delete model:', error)
      // Error is handled by the store
    }
  }

  const handleStartStopModel = async (modelId: string, is_active: boolean) => {
    if (!currentProvider || currentProvider.type !== 'local') return

    try {
      if (is_active) {
        await startModelExecution(modelId)
      } else {
        await stopModelExecution(modelId)
      }

      const modelName = models.find(m => m.id === modelId)?.name || 'Model'
      message.success(`${modelName} ${is_active ? 'started' : 'stopped'}`)
    } catch (error) {
      console.error('Failed to start/stop model:', error)
      if (error instanceof Error) {
        const modelName = models.find(m => m.id === modelId)?.name || 'Model'
        const action = is_active ? 'start' : 'stop'

        const errorMessage = error.message
        modal.error({
          title: `Failed to ${action} ${modelName}`,
          width: '100%',
          closable: true,
          maskClosable: false,
          content: (
            <div className={'w-full h-full overflow-y-auto overflow-x-auto'}>
              <pre>{errorMessage}</pre>
            </div>
          ),
        })
      }
    }
  }

  const handleAddModel = () => {
    if (!currentProvider) return
    if (currentProvider.type === 'local') {
      // For local providers, open the upload drawer by default
      openAddLocalModelUploadDrawer(currentProvider.id)
    } else {
      openAddRemoteModelDrawer(currentProvider.id, currentProvider.type)
    }
  }

  const handleEditModel = (modelId: string) => {
    if (!currentProvider) return
    if (currentProvider.type === 'local') {
      openEditLocalModelDrawer(modelId)
    } else {
      openEditRemoteModelDrawer(modelId)
    }
  }

  const getModelActions = (model: Model) => {
    const actions: React.ReactNode[] = []

    // Always include the enable/disable switch first
    actions.push(
      <Switch
        className={'!mr-2'}
        key="enable"
        checked={model.enabled !== false}
        onChange={checked => handleToggleModel(model.id, checked)}
      />,
    )

    if (currentProvider?.type === 'local') {
      actions.push(
        <Button
          key="start-stop"
          type={model.is_active ? 'default' : 'primary'}
          loading={modelOperations[model.id] || false}
          disabled={modelOperations[model.id] || false}
          onClick={() => handleStartStopModel(model.id, !model.is_active)}
        >
          {modelOperations[model.id]
            ? model.is_active
              ? 'Stopping...'
              : 'Starting...'
            : model.is_active
              ? 'Stop'
              : 'Start'}
        </Button>,
      )
    }

    actions.push(
      <Button
        key="edit"
        type="text"
        icon={<EditOutlined />}
        onClick={() => handleEditModel(model.id)}
      >
        {'Edit'}
      </Button>,
    )

    actions.push(
      <Button
        key="delete"
        type="text"
        icon={<DeleteOutlined />}
        onClick={() => handleDeleteModel(model.id)}
      >
        {'Delete'}
      </Button>,
    )

    return actions.filter(Boolean)
  }

  const getAddButton = () => {
    if (!currentProvider) return null

    if (currentProvider.type === 'local') {
      return (
        <Dropdown
          menu={{
            items: [
              {
                key: 'upload',
                label: 'Upload from Files',
                icon: <UploadOutlined />,
                onClick: () =>
                  openAddLocalModelUploadDrawer(currentProvider.id),
              },
              {
                key: 'download',
                label: 'Download from Repository',
                icon: <PlusOutlined />,
                onClick: () =>
                  openAddLocalModelDownloadDrawer(currentProvider.id),
              },
            ],
          }}
          trigger={['click']}
        >
          <Button type="text" icon={<PlusOutlined />} />
        </Dropdown>
      )
    }

    return (
      <Button type="text" icon={<PlusOutlined />} onClick={handleAddModel} />
    )
  }

  // Return early if no provider
  if (!currentProvider) {
    return null
  }

  return (
    <Card title={t('providers.models')} extra={getAddButton()}>
      {loading ? (
        <div className="flex justify-center py-8">
          <Spin size="large" />
        </div>
      ) : models.length === 0 ? (
        <div>
          <Empty description="No models added yet" />
        </div>
      ) : (
        <div>
          {models.map((model, index) => (
            <div key={model.id}>
              <div className="flex items-start gap-3 flex-wrap">
                {/* Model Info */}
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2 flex-wrap-reverse">
                    <div className={'flex-1 min-w-48'}>
                      <Text className="font-medium">{model.display_name}</Text>
                      {model.is_deprecated && (
                        <span className="text-xs">⚠️</span>
                      )}
                    </div>
                    <div className={'flex gap-1 items-center justify-end'}>
                      {getModelActions(model)}
                    </div>
                  </div>

                  <div className="space-y-1">
                    <Text type="secondary" className="text-xs block">
                      Model ID: {model.name}
                    </Text>
                    {model.is_active && model.port && (
                      <Text type="secondary" className="text-xs block">
                        Running on:{' '}
                        <a
                          href={`http://127.0.0.1:${model.port}`}
                          target="_blank"
                          rel="noopener noreferrer"
                        >
                          http://127.0.0.1:{model.port}
                        </a>
                      </Text>
                    )}
                    {model.description && (
                      <Text type="secondary" className="block">
                        {model.description}
                      </Text>
                    )}
                    {model.capabilities && (
                      <Flex wrap className="gap-3 pt-1 flex-wrap">
                        {model.capabilities.vision && (
                          <Text type="secondary" className="text-xs">
                            👁️ Vision
                          </Text>
                        )}
                        {model.capabilities.audio && (
                          <Text type="secondary" className="text-xs">
                            🎵 Audio
                          </Text>
                        )}
                        {model.capabilities.tools && (
                          <Text type="secondary" className="text-xs">
                            🔧 Tools
                          </Text>
                        )}
                        {model.capabilities.code_interpreter && (
                          <Text type="secondary" className="text-xs">
                            💻 Code
                          </Text>
                        )}
                        {model.capabilities.chat && (
                          <Text type="secondary" className="text-xs">
                            💬 Chat
                          </Text>
                        )}
                        {model.capabilities.text_embedding && (
                          <Text type="secondary" className="text-xs">
                            🔍 Embedding
                          </Text>
                        )}
                        {model.capabilities.image_generator && (
                          <Text type="secondary" className="text-xs">
                            🎨 Image Gen
                          </Text>
                        )}
                      </Flex>
                    )}
                  </div>
                </div>
              </div>
              {index < models.length - 1 && <Divider className="my-0" />}
            </div>
          ))}
        </div>
      )}
    </Card>
  )
}
