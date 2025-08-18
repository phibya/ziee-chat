import { Badge, Card, Flex, Switch, Typography } from 'antd'
import { RAGProvider, RAGProviderType } from '../../../../../types'
import { updateRAGProvider } from '../../../../../store'

const { Title, Text } = Typography

const RAG_PROVIDER_ICONS: Record<RAGProviderType, string> = {
  local: '🏠',
  lightrag: '🔍',
  ragstack: '📚',
  chroma: '🌈',
  weaviate: '🕷️',
  pinecone: '🌲',
  custom: '🔧',
}

interface RAGProviderHeaderProps {
  provider: RAGProvider
}

export function RAGProviderHeader({ provider }: RAGProviderHeaderProps) {
  const handleEnabledChange = async (enabled: boolean) => {
    try {
      await updateRAGProvider(provider.id, { enabled })
    } catch (error) {
      console.error('Failed to update RAG provider:', error)
    }
  }

  return (
    <Card style={{ marginBottom: 24 }}>
      <Flex justify="space-between" align="center">
        <Flex align="center" gap="middle">
          <span style={{ fontSize: '32px' }}>
            {RAG_PROVIDER_ICONS[provider.type]}
          </span>
          <div>
            <Title level={3} style={{ margin: 0 }}>
              {provider.name}
            </Title>
            <Text type="secondary">
              {provider.type.charAt(0).toUpperCase() + provider.type.slice(1)}{' '}
              RAG Provider
            </Text>
            {provider.built_in && (
              <Badge text="Built-in" color="blue" style={{ marginLeft: 8 }} />
            )}
          </div>
        </Flex>
        <Flex align="center" gap="middle">
          <Text>Enabled</Text>
          <Switch
            checked={provider.enabled}
            onChange={handleEnabledChange}
            disabled={provider.built_in}
          />
        </Flex>
      </Flex>
    </Card>
  )
}
