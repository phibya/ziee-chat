import { useParams } from 'react-router-dom'
import { findRAGProviderById } from '../../../../store'
import { RAGProviderHeader } from './shared/RAGProviderHeader'
import { RAGDatabasesSection } from './shared/RAGDatabasesSection'

export function LocalRAGProviderSettings() {
  const { provider_id } = useParams<{ provider_id: string }>()
  const provider = provider_id ? findRAGProviderById(provider_id) : null

  if (!provider) {
    return <div>RAG Provider not found</div>
  }

  return (
    <div>
      <RAGProviderHeader provider={provider} />
      <RAGDatabasesSection provider={provider} />
    </div>
  )
}