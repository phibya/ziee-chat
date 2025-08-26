import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { App, Button, Checkbox, Form, Input } from 'antd'
import { Drawer } from '../../../../common/Drawer.tsx'
import { Stores } from '../../../../../store'
import { updateApiProxyServerModel } from '../../../../../store/admin/apiProxyServer'

interface EditModelDrawerProps {
  open: boolean
  onClose: () => void
  modelId: string | null
}

export function EditModelDrawer({
  open,
  onClose,
  modelId,
}: EditModelDrawerProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  // Get model data from store
  const { models } = Stores.AdminApiProxyServer
  const model = models.find(m => m.model_id === modelId)

  useEffect(() => {
    if (model) {
      form.setFieldsValue({
        alias_id: model.alias_id,
        enabled: model.enabled,
        is_default: model.is_default,
      })
    }
  }, [model, form])

  const handleSubmit = async () => {
    if (!modelId) return

    try {
      const values = await form.validateFields()
      await updateApiProxyServerModel(modelId, values)
      message.success(t('apiProxyServer.modelUpdated'))
      onClose()
    } catch (error) {
      console.error('Form validation failed:', error)
      message.error(t('apiProxyServer.modelUpdateError'))
    }
  }

  return (
    <Drawer
      title={t('apiProxyServer.editModel')}
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
          name="alias_id"
          label={t('apiProxyServer.alias')}
          tooltip={t('apiProxyServer.aliasTooltip')}
        >
          <Input placeholder={t('apiProxyServer.aliasPlaceholder')} />
        </Form.Item>

        <Form.Item name="enabled" valuePropName="checked">
          <Checkbox>{t('apiProxyServer.enabled')}</Checkbox>
        </Form.Item>

        <Form.Item name="is_default" valuePropName="checked">
          <Checkbox>{t('apiProxyServer.setAsDefault')}</Checkbox>
        </Form.Item>
      </Form>
    </Drawer>
  )
}
