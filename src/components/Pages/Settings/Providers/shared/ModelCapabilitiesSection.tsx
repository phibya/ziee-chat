import { Card, Form, Flex, Switch } from 'antd'
import { useTranslation } from 'react-i18next'

interface ModelCapabilitiesSectionProps {
  capabilities?: ('vision' | 'audio' | 'tools' | 'codeInterpreter')[]
}

export function ModelCapabilitiesSection({
  capabilities = ['vision', 'audio', 'tools', 'codeInterpreter'],
}: ModelCapabilitiesSectionProps) {
  const { t } = useTranslation()

  return (
    <Card title={t('providers.capabilities')} size={'small'}>
      <Flex vertical className="gap-2 w-full">
        {capabilities.includes('vision') && (
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
        )}

        {capabilities.includes('audio') && (
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
        )}

        {capabilities.includes('tools') && (
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
        )}

        {capabilities.includes('codeInterpreter') && (
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
        )}
      </Flex>
    </Card>
  )
}
