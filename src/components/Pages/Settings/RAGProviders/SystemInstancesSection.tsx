import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import {
  Button,
  Card,
  Divider,
  Empty,
  Spin,
  Switch,
  Typography,
} from 'antd'
import { RAGInstance, RAGProvider } from '../../../../types/api'

const { Text } = Typography

interface SystemInstancesSectionProps {
  currentProvider: RAGProvider
  currentInstances: RAGInstance[]
  instancesLoading: boolean
  instanceOperations: Record<string, boolean>
  onAddInstance: () => void
  onToggleInstance: (instanceId: string, enabled: boolean) => void
  onEditInstance: (instanceId: string) => void
  onDeleteInstance: (instanceId: string) => void
}

export function SystemInstancesSection({
  currentInstances,
  instancesLoading,
  instanceOperations,
  onAddInstance,
  onToggleInstance,
  onEditInstance,
  onDeleteInstance,
}: SystemInstancesSectionProps) {

  const getInstanceActions = (instance: RAGInstance) => {
    const actions: React.ReactNode[] = []

    // Always include the enable/disable switch first
    actions.push(
      <Switch
        className={'!mr-2'}
        key="enable"
        checked={instance.enabled !== false}
        onChange={checked => onToggleInstance(instance.id, checked)}
        loading={instanceOperations[instance.id] || false}
      />,
    )

    actions.push(
      <Button
        key="edit"
        type="text"
        icon={<EditOutlined />}
        onClick={() => onEditInstance(instance.id)}
        disabled={instanceOperations[instance.id] || false}
      >
        {'Edit'}
      </Button>,
    )

    actions.push(
      <Button
        key="delete"
        type="text"
        icon={<DeleteOutlined />}
        onClick={() => onDeleteInstance(instance.id)}
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
        <Button type="text" icon={<PlusOutlined />} onClick={onAddInstance} />
      }
    >
      {instancesLoading ? (
        <div className="flex justify-center py-8">
          <Spin size="large" />
        </div>
      ) : currentInstances.length === 0 ? (
        <div>
          <Empty description="No instances added yet" />
        </div>
      ) : (
        <div>
          {currentInstances.map((instance, index) => (
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
                    {(instance.engine_settings_rag_simple_vector || instance.engine_settings_rag_simple_graph) && (
                      <Text type="secondary" className="text-xs block">
                        Engine Settings: {JSON.stringify(
                          instance.engine_settings_rag_simple_vector || instance.engine_settings_rag_simple_graph, 
                          null, 
                          2
                        )}
                      </Text>
                    )}
                    <Text type="secondary" className="text-xs block">
                      Created: {new Date(instance.created_at).toLocaleDateString()}
                    </Text>
                  </div>
                </div>
              </div>
              {index < currentInstances.length - 1 && <Divider className="my-0" />}
            </div>
          ))}
        </div>
      )}
    </Card>
  )
}