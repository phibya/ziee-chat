import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Form, Input, Button, App, Checkbox } from 'antd'
import { Drawer } from '../../../../Common/Drawer.tsx'
import type {
  ApiProxyServerTrustedHost,
  UpdateTrustedHostRequest,
} from '../../../../../types/api'

interface EditHostDrawerProps {
  open: boolean
  onClose: () => void
  hostId: string | null
  hosts: ApiProxyServerTrustedHost[]
  onUpdate: (hostId: string, updates: UpdateTrustedHostRequest) => Promise<any>
}

export function EditHostDrawer({
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
