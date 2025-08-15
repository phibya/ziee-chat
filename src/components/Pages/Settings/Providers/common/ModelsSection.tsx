import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import {
  Button,
  Card,
  Divider,
  Empty,
  Flex,
  Spin,
  Switch,
  Typography,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { Model } from '../../../../../types/api/model'
import { Provider } from '../../../../../types/api/provider'

const { Text } = Typography

interface ModelsSectionProps {
  currentProvider: Provider
  currentModels: Model[]
  modelsLoading: boolean
  canEditProviders: boolean
  modelOperations: Record<string, boolean>
  onAddModel: () => void
  onToggleModel: (modelId: string, enabled: boolean) => void
  onEditModel: (modelId: string) => void
  onDeleteModel: (modelId: string) => void
  onStartStopModel?: (modelId: string, isActive: boolean) => void
  customAddButton?: React.ReactNode
}

export function ModelsSection({
  currentProvider,
  currentModels,
  modelsLoading,
  canEditProviders,
  modelOperations,
  onAddModel,
  onToggleModel,
  onEditModel,
  onDeleteModel,
  onStartStopModel,
  customAddButton,
}: ModelsSectionProps) {
  const { t } = useTranslation()

  const getModelActions = (model: Model) => {
    const actions: React.ReactNode[] = []

    // Always include the enable/disable switch first
    actions.push(
      <Switch
        className={'!mr-2'}
        key="enable"
        checked={model.enabled !== false}
        onChange={checked => onToggleModel(model.id, checked)}
        disabled={!canEditProviders}
      />,
    )

    if (canEditProviders) {
      // Local provider specific actions
      if (currentProvider.type === 'local' && onStartStopModel) {
        actions.push(
          <Button
            key="start-stop"
            type={model.is_active ? 'default' : 'primary'}
            loading={modelOperations[model.id] || false}
            disabled={modelOperations[model.id] || false}
            onClick={() => onStartStopModel(model.id, !model.is_active)}
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
          onClick={() => onEditModel(model.id)}
        >
          {'Edit'}
        </Button>,
      )

      actions.push(
        <Button
          key="delete"
          type="text"
          icon={<DeleteOutlined />}
          onClick={() => onDeleteModel(model.id)}
        >
          {'Delete'}
        </Button>,
      )
    }

    return actions.filter(Boolean)
  }

  return (
    <Card
      title={t('providers.models')}
      extra={
        canEditProviders &&
        (customAddButton || (
          <Button type="text" icon={<PlusOutlined />} onClick={onAddModel} />
        ))
      }
    >
      {modelsLoading ? (
        <div className="flex justify-center py-8">
          <Spin size="large" />
        </div>
      ) : currentModels.length === 0 ? (
        <div>
          <Empty description="No models added yet" />
        </div>
      ) : (
        <div>
          {currentModels.map((model, index) => (
            <div key={model.id}>
              <div className="flex items-start gap-3 flex-wrap">
                {/* Model Info */}
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2 flex-wrap-reverse">
                    <div className={'flex-1 min-w-48'}>
                      <Text className="font-medium">{model.alias}</Text>
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
                      </Flex>
                    )}
                  </div>
                </div>
              </div>
              {index < currentModels.length - 1 && <Divider className="my-0" />}
            </div>
          ))}
        </div>
      )}
    </Card>
  )
}
