import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { App, Button, Card, Divider, Empty, Switch, Typography } from 'antd'
import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons'
import { Stores } from '../../../../store'
import {
  addTrustedHostToApiProxyServer,
  removeTrustedHostFromApiProxyServer,
  updateApiProxyServerTrustedHost,
} from '../../../../store/admin/apiProxyServer.ts'
import type {
  ApiProxyServerTrustedHost,
  CreateTrustedHostRequest,
  UpdateTrustedHostRequest,
} from '../../../../types'
import { AddHostDrawer, EditHostDrawer } from './drawers'

const { Text } = Typography

export function TrustedHostsCard() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [addHostDrawerOpen, setAddHostDrawerOpen] = useState(false)
  const [editHostDrawerOpen, setEditHostDrawerOpen] = useState(false)
  const [editingHostId, setEditingHostId] = useState<string | null>(null)

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
          <Button type="text" icon={<PlusOutlined />} onClick={handleAddHost}>
            {t('apiProxyServer.addHost')}
          </Button>
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
  onUpdate: (hostId: string, updates: UpdateTrustedHostRequest) => Promise<void>
  onRemove: (hostId: string) => Promise<void>
  onEdit: (hostId: string) => void
}

function TrustedHostItem({
  host,
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
            />

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
