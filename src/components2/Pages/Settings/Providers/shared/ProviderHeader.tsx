import { Flex, Form, Input, Switch, Tooltip, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { Provider, ProviderType } from '../../../../../types/api/provider'

const { Text } = Typography

const PROVIDER_ICONS: Record<ProviderType, string> = {
  local: '🕯',
  openai: '🤖',
  anthropic: '🤖',
  groq: '⚡',
  gemini: '💎',
  mistral: '🌊',
  custom: '🔧',
}

interface ProviderHeaderProps {
  currentProvider: Provider
  isMobile: boolean
  canEditProviders: boolean
  nameForm: any
  onNameChange: (values: any) => void
  onProviderToggle: (providerId: string, enabled: boolean) => void
  canEnableProvider: (provider: Provider) => boolean
  getEnableDisabledReason: (provider: Provider) => string | null
}

export function ProviderHeader({
  currentProvider,
  isMobile,
  canEditProviders,
  nameForm,
  onNameChange,
  onProviderToggle,
  canEnableProvider,
  getEnableDisabledReason,
}: ProviderHeaderProps) {
  const { t } = useTranslation()

  if (isMobile) {
    return (
      <Flex className={'flex-col gap-2'}>
        <Form
          form={nameForm}
          layout="vertical"
          initialValues={{ name: currentProvider.name }}
          onValuesChange={onNameChange}
        >
          <Form.Item
            name="name"
            label={t('providers.providerName')}
            style={{ margin: 0 }}
          >
            <Input
              style={{
                fontSize: '16px',
                fontWeight: 600,
              }}
              disabled={!canEditProviders}
            />
          </Form.Item>
        </Form>
        <Flex justify="space-between" align="center">
          <Text strong style={{ fontSize: '16px' }}>
            Enable Provider
          </Text>
          {(() => {
            const disabledReason = getEnableDisabledReason(currentProvider)
            const switchElement = (
              <Switch
                checked={currentProvider.enabled}
                disabled={
                  !canEditProviders ||
                  (!currentProvider.enabled &&
                    !canEnableProvider(currentProvider))
                }
                onChange={enabled =>
                  onProviderToggle(currentProvider.id, enabled)
                }
              />
            )

            if (!canEditProviders) return switchElement
            if (disabledReason && !currentProvider.enabled) {
              return <Tooltip title={disabledReason}>{switchElement}</Tooltip>
            }
            return switchElement
          })()}
        </Flex>
      </Flex>
    )
  }

  return (
    <Flex justify="space-between" align="center">
      <Flex align="center" gap="middle">
        <span style={{ fontSize: '24px' }}>
          {PROVIDER_ICONS[currentProvider.type]}
        </span>
        <Form
          form={nameForm}
          layout="inline"
          initialValues={{ name: currentProvider.name }}
          onValuesChange={onNameChange}
        >
          <Form.Item name="name" style={{ margin: 0 }}>
            <Input
              variant="borderless"
              style={{
                fontSize: '24px',
                fontWeight: 600,
                padding: 0,
                border: 'none',
                boxShadow: 'none',
              }}
              disabled={!canEditProviders}
            />
          </Form.Item>
        </Form>
      </Flex>
      {(() => {
        const disabledReason = getEnableDisabledReason(currentProvider)
        const switchElement = (
          <Switch
            checked={currentProvider.enabled}
            disabled={
              !canEditProviders ||
              (!currentProvider.enabled && !canEnableProvider(currentProvider))
            }
            onChange={enabled => onProviderToggle(currentProvider.id, enabled)}
          />
        )

        if (!canEditProviders) return switchElement
        if (disabledReason && !currentProvider.enabled) {
          return <Tooltip title={disabledReason}>{switchElement}</Tooltip>
        }
        return switchElement
      })()}
    </Flex>
  )
}
