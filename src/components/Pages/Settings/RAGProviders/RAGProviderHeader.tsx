import { Button, Flex, Form, Input, Switch, Tooltip, Typography } from 'antd'
import { RAGProvider } from '../../../../types/api'
import { CheckOutlined, CloseOutlined, EditOutlined } from '@ant-design/icons'
import { useState } from 'react'
import { updateRAGProvider } from '../../../../store'
import { RAG_PROVIDER_ICONS } from '../../../../constants/ragProviders'

interface RAGProviderHeaderProps {
  currentProvider: RAGProvider
  onProviderToggle: (providerId: string, enabled: boolean) => void
  canEnableProvider: (provider: RAGProvider) => boolean
  getEnableDisabledReason: (provider: RAGProvider) => string | null
}

export function RAGProviderHeader({
  currentProvider,
  onProviderToggle,
  canEnableProvider,
  getEnableDisabledReason,
}: RAGProviderHeaderProps) {
  const [isEditingName, setIsEditingName] = useState(false)
  const [form] = Form.useForm()
  
  return (
    <Flex justify="space-between" align="center">
      <Flex align="center" gap="middle">
        {(() => {
          const IconComponent = RAG_PROVIDER_ICONS[currentProvider.type]
          return <IconComponent className="text-2xl" />
        })()}
        <Form
          style={{
            display: isEditingName ? 'block' : 'none',
          }}
          form={form}
          layout="inline"
          initialValues={{ name: currentProvider.name }}
        >
          <div className={'flex items-center gap-2 w-full flex-wrap'}>
            <Form.Item
              name="name"
              style={{ margin: 0 }}
              rules={[{ required: true, message: 'Name is required' }]}
            >
              <Input className={'!text-lg'} />
            </Form.Item>
            <div className={'flex items-center gap-2'}>
              <Button
                type={'primary'}
                onClick={() => {
                  form.validateFields().then(async values => {
                    await updateRAGProvider(currentProvider.id, {
                      name: values.name,
                    })
                    setIsEditingName(false)
                  })
                }}
              >
                <CheckOutlined />
              </Button>
              <Button onClick={() => setIsEditingName(false)}>
                <CloseOutlined />
              </Button>
            </div>
          </div>
        </Form>
        <div
          className={'flex items-center gap-2'}
          style={{
            display: isEditingName ? 'none' : 'flex',
          }}
        >
          <Typography.Title level={4} className={'!m-0'}>
            {currentProvider.name}
          </Typography.Title>
          <Button
            type={'text'}
            onClick={() => {
              setIsEditingName(!isEditingName)
            }}
          >
            <EditOutlined />
          </Button>
        </div>
      </Flex>
      {(() => {
        const disabledReason = getEnableDisabledReason(currentProvider)
        const switchElement = (
          <Switch
            checked={currentProvider.enabled}
            disabled={
              !currentProvider.enabled && !canEnableProvider(currentProvider)
            }
            onChange={enabled => onProviderToggle(currentProvider.id, enabled)}
          />
        )

        if (disabledReason && !currentProvider.enabled) {
          return <Tooltip title={disabledReason}>{switchElement}</Tooltip>
        }
        return switchElement
      })()}
    </Flex>
  )
}