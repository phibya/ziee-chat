import { Card, Flex, Form, Switch } from 'antd'
import { useTranslation } from 'react-i18next'

export function ModelCapabilitiesSection() {
  const { t } = useTranslation()

  return (
    <Card title={t('providers.capabilities')}>
      <Flex vertical className="gap-2 w-full">
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>{t('providers.chat')}</span>
          <Form.Item
            name={['capabilities', 'chat']}
            valuePropName="checked"
            style={{ marginBottom: 0 }}
          >
            <Switch />
          </Form.Item>
        </div>
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>{t('providers.vision')}</span>
          <Form.Item
            name={['capabilities', 'vision']}
            valuePropName="checked"
            style={{ marginBottom: 0 }}
          >
            <Switch />
          </Form.Item>
        </div>

        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>{t('providers.audio')}</span>
          <Form.Item
            name={['capabilities', 'audio']}
            valuePropName="checked"
            style={{ marginBottom: 0 }}
          >
            <Switch />
          </Form.Item>
        </div>

        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>{t('providers.tools')}</span>
          <Form.Item
            name={['capabilities', 'tools']}
            valuePropName="checked"
            style={{ marginBottom: 0 }}
          >
            <Switch />
          </Form.Item>
        </div>

        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>{t('providers.codeInterpreter')}</span>
          <Form.Item
            name={['capabilities', 'codeInterpreter']}
            valuePropName="checked"
            style={{ marginBottom: 0 }}
          >
            <Switch />
          </Form.Item>
        </div>

        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>{t('providers.textEmbedding')}</span>
          <Form.Item
            name={['capabilities', 'text_embedding']}
            valuePropName="checked"
            style={{ marginBottom: 0 }}
          >
            <Switch />
          </Form.Item>
        </div>

        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <span>{t('providers.imageGenerator')}</span>
          <Form.Item
            name={['capabilities', 'image_generator']}
            valuePropName="checked"
            style={{ marginBottom: 0 }}
          >
            <Switch />
          </Form.Item>
        </div>
      </Flex>
    </Card>
  )
}
