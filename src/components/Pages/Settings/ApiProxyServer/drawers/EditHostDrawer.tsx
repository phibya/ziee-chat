import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { App, Button, Checkbox, Form, Input } from 'antd'
import { Drawer } from '../../../../common/Drawer.tsx'
import { Stores } from '../../../../../store'
import { updateApiProxyServerTrustedHost } from '../../../../../store/admin/apiProxyServer'

interface EditHostDrawerProps {
  open: boolean
  onClose: () => void
  hostId: string | null
}

export function EditHostDrawer({
  open,
  onClose,
  hostId,
}: EditHostDrawerProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  // Get host data from store
  const { trustedHosts } = Stores.AdminApiProxyServer
  const host = trustedHosts.find(h => h.id === hostId)

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
      await updateApiProxyServerTrustedHost(hostId, values)
      message.success(t('apiProxyServer.hostUpdated'))
      onClose()
    } catch (error) {
      console.error('Form validation failed:', error)
      message.error(t('apiProxyServer.hostUpdateError'))
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
