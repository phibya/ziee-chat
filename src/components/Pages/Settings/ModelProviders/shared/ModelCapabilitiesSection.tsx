import { Form, Space, Switch, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Title } = Typography

interface ModelCapabilitiesSectionProps {
  capabilities?: ('vision' | 'audio' | 'tools' | 'codeInterpreter')[]
}

export function ModelCapabilitiesSection({
  capabilities = ['vision', 'audio', 'tools', 'codeInterpreter'],
}: ModelCapabilitiesSectionProps) {
  const { t } = useTranslation()

  return (
    <>
      <Title level={5}>{t('modelProviders.capabilities')}</Title>
      <Space direction="vertical" size="middle" style={{ width: '100%' }}>
        {capabilities.includes('vision') && (
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.vision')}</span>
            <Form.Item
              name="vision"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
        )}

        {capabilities.includes('audio') && (
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.audio')}</span>
            <Form.Item
              name="audio"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
        )}

        {capabilities.includes('tools') && (
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.tools')}</span>
            <Form.Item
              name="tools"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
        )}

        {capabilities.includes('codeInterpreter') && (
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span>{t('modelProviders.codeInterpreter')}</span>
            <Form.Item
              name="codeInterpreter"
              valuePropName="checked"
              style={{ marginBottom: 0 }}
            >
              <Switch />
            </Form.Item>
          </div>
        )}
      </Space>
    </>
  )
}
