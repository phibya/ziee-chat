import { Card, Form, InputNumber, Typography } from 'antd'
import { RAGEngineType } from '../../../../../types/api'

const { Text } = Typography

export function SimpleVectorEngineSettings() {
  // Get engine type from the parent form context
  const engineType = Form.useWatch('engine_type') as RAGEngineType

  // Helper function to generate proper field paths for nested form fields
  const getFieldName = (field: string) => [
    'engine_settings',
    'simple_vector',
    field,
  ]

  // Only render settings for Simple Vector RAG engine
  if (engineType !== 'simple_vector') {
    return null
  }

  return (
    <Card title="Simple Vector Engine Configuration" size="small">
      <div className="space-y-4">
        <Text type="secondary">
          Configuration settings for the Simple Vector RAG engine. This engine
          performs basic vector similarity search using embeddings.
        </Text>

        <Form.Item
          label="Similarity Threshold"
          name={getFieldName('similarity_threshold')}
          help="Minimum similarity score for retrieved documents (0.0 to 1.0)"
        >
          <InputNumber min={0} max={1} step={0.1} placeholder="0.7" />
        </Form.Item>

        <Form.Item
          label="Max Results"
          name={getFieldName('max_results')}
          help="Maximum number of documents to retrieve"
        >
          <InputNumber min={1} max={100} placeholder="10" />
        </Form.Item>

        <Form.Item
          label="Chunk Size"
          name={getFieldName('chunk_size')}
          help="Size of text chunks for processing"
        >
          <InputNumber min={100} max={5000} placeholder="1000" />
        </Form.Item>

        <Form.Item
          label="Chunk Overlap"
          name={getFieldName('chunk_overlap')}
          help="Overlap between consecutive chunks"
        >
          <InputNumber min={0} max={1000} placeholder="200" />
        </Form.Item>
      </div>
    </Card>
  )
}
