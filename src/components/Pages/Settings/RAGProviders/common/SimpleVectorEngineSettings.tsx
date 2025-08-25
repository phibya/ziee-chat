import { Card, Form, InputNumber, Typography } from 'antd'
import { RAGEngineType } from '../../../../../types/api'

const { Text } = Typography

interface SimpleVectorEngineSettingsProps {
  engineType: RAGEngineType
  settings: Record<string, any>
  onChange?: (settings: Record<string, any>) => void
}

export function SimpleVectorEngineSettings({
  engineType,
  settings,
  onChange,
}: SimpleVectorEngineSettingsProps) {
  // Placeholder implementation for Simple Vector RAG engine settings
  if (engineType !== 'rag_simple_vector') {
    return null
  }

  return (
    <Card title="Simple Vector Engine Configuration" size="small">
      <div className="space-y-4">
        <Text type="secondary">
          Configuration settings for the Simple Vector RAG engine. This engine performs basic vector similarity search using embeddings.
        </Text>
        
        <Form.Item
          label="Similarity Threshold"
          name={['engine_settings', 'similarity_threshold']}
          help="Minimum similarity score for retrieved documents (0.0 to 1.0)"
        >
          <InputNumber
            min={0}
            max={1}
            step={0.1}
            placeholder="0.7"
            value={settings?.similarity_threshold || 0.7}
            onChange={(value) => onChange?.({ ...settings, similarity_threshold: value })}
          />
        </Form.Item>

        <Form.Item
          label="Max Results"
          name={['engine_settings', 'max_results']}
          help="Maximum number of documents to retrieve"
        >
          <InputNumber
            min={1}
            max={100}
            placeholder="10"
            value={settings?.max_results || 10}
            onChange={(value) => onChange?.({ ...settings, max_results: value })}
          />
        </Form.Item>

        <Form.Item
          label="Chunk Size"
          name={['engine_settings', 'chunk_size']}
          help="Size of text chunks for processing"
        >
          <InputNumber
            min={100}
            max={5000}
            placeholder="1000"
            value={settings?.chunk_size || 1000}
            onChange={(value) => onChange?.({ ...settings, chunk_size: value })}
          />
        </Form.Item>

        <Form.Item
          label="Chunk Overlap"
          name={['engine_settings', 'chunk_overlap']}
          help="Overlap between consecutive chunks"
        >
          <InputNumber
            min={0}
            max={1000}
            placeholder="200"
            value={settings?.chunk_overlap || 200}
            onChange={(value) => onChange?.({ ...settings, chunk_overlap: value })}
          />
        </Form.Item>
      </div>
    </Card>
  )
}