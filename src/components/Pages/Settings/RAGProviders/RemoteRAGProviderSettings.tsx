import { useParams } from 'react-router-dom'
import { findRAGProviderById } from '../../../../store'
import { RAGProviderHeader } from './shared/RAGProviderHeader'
import { RAGDatabasesSection } from './shared/RAGDatabasesSection'
import { RAGConfigurationSection } from './shared/RAGConfigurationSection'

export function RemoteRAGProviderSettings() {
  const { provider_id } = useParams<{ provider_id: string }>()
  const provider = provider_id ? findRAGProviderById(provider_id) : null

  if (!provider) {
    return <div>RAG Provider not found</div>
  }

  return (
    <div>
      <RAGProviderHeader provider={provider} />
      <RAGConfigurationSection provider={provider} />
      <RAGDatabasesSection provider={provider} />
    </div>
  )
}