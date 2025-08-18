import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Card,
  Button,
  Typography,
  App,
  Tag,
  Empty,
  Divider,
  Flex,
  Switch,
} from 'antd'
import { PlusOutlined, DeleteOutlined, EditOutlined } from '@ant-design/icons'
import { Permission, usePermissions } from '../../../../permissions'
import { Stores } from '../../../../store'
import {
  updateApiProxyServerModel,
  removeModelFromApiProxyServer,
} from '../../../../store/admin/apiProxyServer.ts'
import type {
  UpdateApiProxyServerModelRequest,
  ApiProxyServerModel,
} from '../../../../types'
import { AddModelDrawer, EditModelDrawer } from './drawers'

const { Text } = Typography

export function ModelSelectionCard() {
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

  const handleAddModel = () => {
    setAddModelDrawerOpen(true)
  }

  const handleEditModel = (modelId: string) => {
    setEditingModelId(modelId)
    setEditModelDrawerOpen(true)
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
