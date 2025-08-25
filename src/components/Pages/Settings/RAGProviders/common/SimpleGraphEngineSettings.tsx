import { Card, Form, InputNumber, Typography } from 'antd'
import { RAGEngineType } from '../../../../../types/api'

const { Text } = Typography

interface SimpleGraphEngineSettingsProps {
  engineType: RAGEngineType
  settings: Record<string, any>
  onChange?: (settings: Record<string, any>) => void
}

export function SimpleGraphEngineSettings({
  engineType,
  settings,
  onChange,
}: SimpleGraphEngineSettingsProps) {
  // Placeholder implementation for Simple Graph RAG engine settings
  if (engineType !== 'rag_simple_graph') {
    return null
  }

  return (
    <Card title="Simple Graph Engine Configuration" size="small">
      <div className="space-y-4">
        <Text type="secondary">
          Configuration settings for the Simple Graph RAG engine. This engine performs graph-based RAG with entity and relationship extraction.
        </Text>
        
        <Form.Item
          label="Similarity Threshold"
          name={['engine_settings', 'similarity_threshold']}
          help="Minimum similarity score for retrieved nodes (0.0 to 1.0)"
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
          help="Maximum number of nodes to retrieve"
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
          label="Community Level"
          name={['engine_settings', 'community_level']}
          help="Level in the community hierarchy for graph traversal"
        >
          <InputNumber
            min={0}
            max={5}
            placeholder="1"
            value={settings?.community_level || 1}
            onChange={(value) => onChange?.({ ...settings, community_level: value })}
          />
        </Form.Item>

        <Form.Item
          label="Graph Depth"
          name={['engine_settings', 'graph_depth']}
          help="Maximum depth for graph traversal"
        >
          <InputNumber
            min={1}
            max={10}
            placeholder="2"
            value={settings?.graph_depth || 2}
            onChange={(value) => onChange?.({ ...settings, graph_depth: value })}
          />
        </Form.Item>
      </div>
    </Card>
  )
}