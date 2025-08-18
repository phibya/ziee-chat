import { useTranslation } from 'react-i18next'
import { App, Button, Checkbox, Form, Input } from 'antd'
import { Drawer } from '../../../../Common/Drawer.tsx'
import type { CreateTrustedHostRequest } from '../../../../../types'

interface AddHostDrawerProps {
  open: boolean
  onClose: () => void
  onAdd: (data: CreateTrustedHostRequest) => Promise<any>
}

export function AddHostDrawer({ open, onClose, onAdd }: AddHostDrawerProps) {
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
