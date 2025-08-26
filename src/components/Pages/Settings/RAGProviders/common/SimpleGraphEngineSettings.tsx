import { Card, Form, InputNumber, Typography } from 'antd'
import { RAGEngineType } from '../../../../../types/api'

const { Text } = Typography

export function SimpleGraphEngineSettings() {
  // Get engine type from the parent form context
  const engineType = Form.useWatch('engine_type') as RAGEngineType

  // Helper function to generate proper field paths for nested form fields
  const getFieldName = (field: string) => [
    'engine_settings',
    'simple_graph',
    field,
  ]

  // Only render settings for Simple Graph RAG engine
  if (engineType !== 'simple_graph') {
    return null
  }

  return (
    <Card title="Simple Graph Engine Configuration" size="small">
      <div className="space-y-4">
        <Text type="secondary">
          Configuration settings for the Simple Graph RAG engine. This engine
          performs graph-based RAG with entity and relationship extraction.
        </Text>

        <Form.Item
          label="Similarity Threshold"
          name={getFieldName('similarity_threshold')}
          help="Minimum similarity score for retrieved nodes (0.0 to 1.0)"
        >
          <InputNumber min={0} max={1} step={0.1} placeholder="0.7" />
        </Form.Item>

        <Form.Item
          label="Max Results"
          name={getFieldName('max_results')}
          help="Maximum number of nodes to retrieve"
        >
          <InputNumber min={1} max={100} placeholder="10" />
        </Form.Item>

        <Form.Item
          label="Community Level"
          name={getFieldName('community_level')}
          help="Level in the community hierarchy for graph traversal"
        >
          <InputNumber min={0} max={5} placeholder="1" />
        </Form.Item>

        <Form.Item
          label="Graph Depth"
          name={getFieldName('graph_depth')}
          help="Maximum depth for graph traversal"
        >
          <InputNumber min={1} max={10} placeholder="2" />
        </Form.Item>
      </div>
    </Card>
  )
}
