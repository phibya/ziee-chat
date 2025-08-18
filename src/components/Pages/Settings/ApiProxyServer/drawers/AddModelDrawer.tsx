import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { App, Button, Checkbox, Form, Input, Select } from 'antd'
import { Drawer } from '../../../../Common/Drawer.tsx'
import { loadAllModelProviders, Stores } from '../../../../../store'
import { addModelToApiProxyServer } from '../../../../../store/admin/apiProxyServer'

interface AddModelDrawerProps {
  open: boolean
  onClose: () => void
}

export function AddModelDrawer({ open, onClose }: AddModelDrawerProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  // Get store data
  const { models } = Stores.AdminApiProxyServer
  const allProviders = Stores.AdminProviders.providers || []

  // Load model providers when drawer opens
  useEffect(() => {
    if (open) {
      loadAllModelProviders()
    }
  }, [open])

  // Construct select options directly from providers
  const selectOptions = allProviders
    .map(provider => ({
      label: provider.name,
      options: (provider.models || [])
        .filter(model => !models.find(pm => pm.model_id === model.id))
        .map(model => ({
          label: model.alias,
          value: model.id,
          model: model, // Include model data for onChange handler
        })),
    }))
    .filter(group => group.options.length > 0) // Only include providers with available models

  // Create a flat lookup for model data by ID
  const modelLookup = allProviders
    .flatMap(provider => provider.models || [])
    .reduce(
      (lookup, model) => {
        lookup[model.id] = model
        return lookup
      },
      {} as Record<string, any>,
    )

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields()
      await addModelToApiProxyServer(values)
      message.success(t('apiProxyServer.modelAdded'))
      form.resetFields()
      onClose()
    } catch (error) {
      console.error('Add model failed:', error)
      message.error(t('apiProxyServer.modelAddError'))
    }
  }

  return (
    <Drawer
      title={t('apiProxyServer.addModelToProxy')}
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
          name="model_id"
          label={t('apiProxyServer.selectModel')}
          rules={[
            { required: true, message: t('apiProxyServer.modelRequired') },
          ]}
        >
          <Select
            placeholder={t('apiProxyServer.selectModelPlaceholder')}
            showSearch
            filterOption={(input, option) => {
              // Filter option labels
              const optionLabel = String(option?.label ?? '')
              const searchTerm = input.toLowerCase()
              return optionLabel.toLowerCase().includes(searchTerm)
            }}
            options={selectOptions}
            onChange={modelId => {
              const selectedModel = modelLookup[modelId]
              if (selectedModel) {
                form.setFieldValue('alias_id', selectedModel.name)
              }
            }}
          />
        </Form.Item>

        <Form.Item
          name="alias_id"
          label={t('apiProxyServer.alias')}
          tooltip={t('apiProxyServer.aliasTooltip')}
        >
          <Input placeholder={t('apiProxyServer.aliasPlaceholder')} />
        </Form.Item>

        <Form.Item name="enabled" valuePropName="checked" initialValue={true}>
          <Checkbox>{t('apiProxyServer.enabledByDefault')}</Checkbox>
        </Form.Item>

        <Form.Item name="is_default" valuePropName="checked">
          <Checkbox>{t('apiProxyServer.setAsDefault')}</Checkbox>
        </Form.Item>
      </Form>
    </Drawer>
  )
}
