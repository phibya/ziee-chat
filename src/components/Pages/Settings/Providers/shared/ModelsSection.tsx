import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import { Button, Card, Flex, List, Switch, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { Model } from '../../../../../types/api/model'
import { Provider } from '../../../../../types/api/provider'

const { Text } = Typography

interface ModelsSectionProps {
  currentProvider: Provider
  currentModels: Model[]
  modelsLoading: boolean
  canEditProviders: boolean
  isMobile: boolean
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
  isMobile,
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

    if (canEditProviders) {
      // Local provider specific actions
      if (currentProvider.type === 'local' && onStartStopModel) {
        actions.push(
          <Button
            key="start-stop"
            type={model.is_active ? 'default' : 'primary'}
            size={isMobile ? 'small' : 'middle'}
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
          size={isMobile ? 'small' : 'middle'}
          onClick={() => onEditModel(model.id)}
        >
          {!isMobile && 'Edit'}
        </Button>,
      )

      actions.push(
        <Button
          key="delete"
          type="text"
          icon={<DeleteOutlined />}
          size={isMobile ? 'small' : 'middle'}
          onClick={() => onDeleteModel(model.id)}
        >
          {!isMobile && 'Delete'}
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
      <List
        loading={modelsLoading}
        dataSource={currentModels}
        locale={{ emptyText: 'No models added yet' }}
        renderItem={model => (
          <List.Item actions={getModelActions(model)}>
            <List.Item.Meta
              avatar={
                <Switch
                  checked={model.enabled !== false}
                  onChange={checked => onToggleModel(model.id, checked)}
                  disabled={!canEditProviders}
                />
              }
              title={
                <Flex align="center" gap="small">
                  <Text>{model.alias}</Text>
                  {model.is_deprecated && (
                    <span style={{ fontSize: '12px' }}>⚠️</span>
                  )}
                </Flex>
              }
              description={
                <Flex vertical className="gap-1">
                  <Text type="secondary" className="text-xs">
                    Model ID: {model.name}
                  </Text>
                  {model.description && (
                    <Text type="secondary">{model.description}</Text>
                  )}
                  {model.capabilities && (
                    <Flex wrap className="gap-1">
                      {model.capabilities.vision && (
                        <Text type="secondary">👁️ Vision</Text>
                      )}
                      {model.capabilities.audio && (
                        <Text type="secondary">🎵 Audio</Text>
                      )}
                      {model.capabilities.tools && (
                        <Text type="secondary">🔧 Tools</Text>
                      )}
                      {model.capabilities.code_interpreter && (
                        <Text type="secondary">💻 Code</Text>
                      )}
                    </Flex>
                  )}
                </Flex>
              }
            />
          </List.Item>
        )}
      />
    </Card>
  )
}
