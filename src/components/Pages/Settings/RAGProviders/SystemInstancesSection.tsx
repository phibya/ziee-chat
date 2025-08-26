import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Divider,
  Empty,
  Spin,
  Switch,
  Typography,
} from 'antd'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import {
  deleteSystemRAGInstance,
  disableSystemRAGInstance,
  enableSystemRAGInstance,
  getInstancesForProvider,
  openAddSystemInstanceDrawer,
  Stores,
  updateRAGProvider,
} from '../../../../store'
import { RAGInstance } from '../../../../types/api'

const { Text } = Typography

export function SystemInstancesSection() {
  const { message } = App.useApp()
  const { t } = useTranslation()
  const { providerId } = useParams<{ providerId?: string }>()
  const navigate = useNavigate()

  // Store data
  const { instanceOperations, instancesLoading } = Stores.AdminRAGProviders

  // Get current provider and instances
  const currentProvider = Stores.AdminRAGProviders.providers.find(
    p => p.id === providerId,
  )
  const instances = getInstancesForProvider(providerId || '')
  const loading = instancesLoading[providerId!] || false

  const handleAddInstance = () => {
    if (currentProvider) {
      openAddSystemInstanceDrawer(currentProvider.id)
    }
  }

  const handleToggleInstance = async (instanceId: string, enabled: boolean) => {
    if (!currentProvider) return

    try {
      if (enabled) {
        await enableSystemRAGInstance(instanceId)
      } else {
        await disableSystemRAGInstance(instanceId)
      }

      // Check if this was the last enabled instance being disabled
      if (!enabled) {
        const remainingEnabledInstances = instances.filter(
          i => i.id !== instanceId && i.enabled !== false,
        )

        // If no instances remain enabled and provider is currently enabled, disable the provider
        if (remainingEnabledInstances.length === 0 && currentProvider.enabled) {
          try {
            await updateRAGProvider(currentProvider.id, { enabled: false })
            const instanceName =
              instances.find(i => i.id === instanceId)?.name || 'Instance'
            message.success(
              `${instanceName} disabled. ${currentProvider.name} provider disabled as no instances remain active.`,
            )
          } catch (providerError) {
            console.error('Failed to disable RAG provider:', providerError)
            const instanceName =
              instances.find(i => i.id === instanceId)?.name || 'Instance'
            message.warning(
              `${instanceName} disabled, but failed to disable provider automatically`,
            )
          }
        } else {
          const instanceName =
            instances.find(i => i.id === instanceId)?.name || 'Instance'
          message.success(`${instanceName} ${enabled ? 'enabled' : 'disabled'}`)
        }
      } else {
        const instanceName =
          instances.find(i => i.id === instanceId)?.name || 'Instance'
        message.success(`${instanceName} ${enabled ? 'enabled' : 'disabled'}`)
      }
    } catch (error) {
      console.error('Failed to toggle instance:', error)
      // Error is handled by the store
    }
  }

  const handleDeleteInstance = async (instanceId: string) => {
    if (!currentProvider) return

    try {
      await deleteSystemRAGInstance(instanceId)
      message.success(t('providers.instanceDeleted'))
    } catch (error) {
      console.error('Failed to delete instance:', error)
      // Error is handled by the store
    }
  }

  // Return early if no provider
  if (!currentProvider) {
    return null
  }

  const getInstanceActions = (instance: RAGInstance) => {
    const actions: React.ReactNode[] = []

    // Always include the enable/disable switch first
    actions.push(
      <Switch
        className={'!mr-2'}
        key="enable"
        checked={instance.enabled !== false}
        onChange={checked => handleToggleInstance(instance.id, checked)}
        loading={instanceOperations[instance.id] || false}
      />,
    )

    actions.push(
      <Button
        key="details"
        type="text"
        icon={<EditOutlined />}
        onClick={() => navigate(`/rags/${instance.id}`)}
        disabled={instanceOperations[instance.id] || false}
      >
        Details
      </Button>,
    )

    actions.push(
      <Button
        key="delete"
        type="text"
        icon={<DeleteOutlined />}
        onClick={() => handleDeleteInstance(instance.id)}
        disabled={instanceOperations[instance.id] || false}
      >
        {'Delete'}
      </Button>,
    )

    return actions.filter(Boolean)
  }

  return (
    <Card
      title="Instances"
      extra={
        <Button
          type="text"
          icon={<PlusOutlined />}
          onClick={handleAddInstance}
        />
      }
    >
      {loading ? (
        <div className="flex justify-center py-8">
          <Spin size="large" />
        </div>
      ) : instances.length === 0 ? (
        <div>
          <Empty description="No instances added yet" />
        </div>
      ) : (
        <div>
          {instances.map((instance, index) => (
            <div key={instance.id}>
              <div className="flex items-start gap-3 flex-wrap">
                {/* Instance Info */}
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2 flex-wrap-reverse">
                    <div className={'flex-1 min-w-48'}>
                      <Text className="font-medium">{instance.name}</Text>
                    </div>
                    <div className={'flex gap-1 items-center justify-end'}>
                      {getInstanceActions(instance)}
                    </div>
                  </div>

                  <div className="space-y-1">
                    <Text type="secondary" className="text-xs block">
                      Instance ID: {instance.id}
                    </Text>
                    <Text type="secondary" className="text-xs block">
                      Engine Type: {instance.engine_type}
                    </Text>
                    {instance.description && (
                      <Text type="secondary" className="block">
                        {instance.description}
                      </Text>
                    )}
                    <Text type="secondary" className="text-xs block">
                      Status: {instance.enabled ? 'Enabled' : 'Disabled'}
                    </Text>
                    <Text type="secondary" className="text-xs block">
                      Created:{' '}
                      {new Date(instance.created_at).toLocaleDateString()}
                    </Text>
                  </div>
                </div>
              </div>
              {index < instances.length - 1 && <Divider className="my-0" />}
            </div>
          ))}
        </div>
      )}
    </Card>
  )
}
