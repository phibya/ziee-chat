import React from 'react'
import {
  Form,
  InputNumber,
  Select,
  Input,
  Divider,
  Typography,
  Collapse,
  Switch,
} from 'antd'
import { useAvailableModels } from './hooks/useAvailableModels'

const { Text } = Typography
const { Panel } = Collapse

// Helper function to generate form field names
const getFieldName = (section: 'indexing' | 'querying', field: string) => [
  'engine_settings',
  'simple_graph',
  section,
  field,
]

export const RagSimpleGraphEngineSettings: React.FC = () => {
  const { getAvailableModels } = useAvailableModels()
  return (
    <div>
      <Collapse defaultActiveKey={['indexing', 'querying']}>
        <Panel header="Indexing Settings" key="indexing">
          <div>
            <Text type="secondary" className="text-xs block mb-4">
              Changes to indexing settings require reprocessing of documents
            </Text>

            <Form.Item label="Embedding Model" name="embedding_model_id">
              <Select
                placeholder="Select embedding model"
                allowClear
                showSearch
                filterOption={(input, option) => {
                  if (!option) return false
                  if ('options' in option && Array.isArray(option.options)) {
                    // This is a group option - search in children
                    return option.options.some((child: any) =>
                      child?.label?.toLowerCase().includes(input.toLowerCase()),
                    )
                  }
                  // This is a regular option
                  return (option.label ?? '')
                    .toLowerCase()
                    .includes(input.toLowerCase())
                }}
                options={getAvailableModels('text_embedding')}
              />
            </Form.Item>

            <Form.Item label="LLM Model" name="llm_model_id">
              <Select
                placeholder="Select LLM model"
                allowClear
                showSearch
                filterOption={(input, option) => {
                  if (!option) return false
                  if ('options' in option && Array.isArray(option.options)) {
                    // This is a group option - search in children
                    return option.options.some((child: any) =>
                      child?.label?.toLowerCase().includes(input.toLowerCase()),
                    )
                  }
                  // This is a regular option
                  return (option.label ?? '')
                    .toLowerCase()
                    .includes(input.toLowerCase())
                }}
                options={getAvailableModels('chat')}
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Chunking Configuration</Text>
            </Divider>

            <Form.Item
              label="Chunk Token Size"
              name={getFieldName('indexing', 'chunk_token_size')}
              tooltip="Maximum number of tokens per chunk (default: 1200)"
            >
              <InputNumber
                placeholder="1200"
                min={100}
                max={8192}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Chunk Overlap Size"
              name={getFieldName('indexing', 'chunk_overlap_token_size')}
              tooltip="Number of tokens to overlap between chunks (default: 100)"
            >
              <InputNumber
                placeholder="100"
                min={0}
                max={500}
                className="w-full"
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Entity Extraction</Text>
            </Divider>

            <Form.Item
              label="Max Gleaning Iterations"
              name={getFieldName('indexing', 'entity_extract_max_gleaning')}
              tooltip="Maximum gleaning iterations for entity extraction (default: 1)"
            >
              <InputNumber
                placeholder="1"
                min={1}
                max={10}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Force LLM Summary Threshold"
              name={getFieldName('indexing', 'force_llm_summary_on_merge')}
              tooltip="Force LLM summary when merging this many entities (default: 4)"
            >
              <InputNumber
                placeholder="4"
                min={1}
                max={20}
                className="w-full"
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Graph Configuration</Text>
            </Divider>

            <Form.Item
              label="Max Graph Nodes"
              name={getFieldName('indexing', 'max_graph_nodes')}
              tooltip="Maximum number of nodes in the graph (default: 1000)"
            >
              <InputNumber
                placeholder="1000"
                min={100}
                max={10000}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Summary Max Tokens"
              name={getFieldName('indexing', 'summary_max_tokens')}
              tooltip="Maximum tokens for entity summaries (default: 30000)"
            >
              <InputNumber
                placeholder="30000"
                min={1000}
                max={100000}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Entity Types"
              name={getFieldName('indexing', 'entity_types')}
              tooltip="Types of entities to extract (default: organization, person, geo, event, category)"
            >
              <Select
                mode="tags"
                placeholder="Enter entity types..."
                className="w-full"
                tokenSeparators={[',']}
                options={[
                  { value: 'organization', label: 'Organization' },
                  { value: 'person', label: 'Person' },
                  { value: 'geo', label: 'Geographic' },
                  { value: 'event', label: 'Event' },
                  { value: 'category', label: 'Category' },
                ]}
              />
            </Form.Item>

            <Form.Item
              label="Extraction Language"
              name={getFieldName('indexing', 'extraction_language')}
              tooltip="Language for entity extraction (default: English)"
            >
              <Select
                placeholder="Select language"
                allowClear
                options={[
                  { value: 'English', label: 'English' },
                  { value: 'Spanish', label: 'Spanish' },
                  { value: 'French', label: 'French' },
                  { value: 'German', label: 'German' },
                  { value: 'Chinese', label: 'Chinese' },
                  { value: 'Japanese', label: 'Japanese' },
                ]}
              />
            </Form.Item>
          </div>
        </Panel>

        <Panel header="Querying Settings" key="querying">
          <div>
            <Text type="secondary" className="text-xs block mb-4">
              Changes to querying settings apply immediately without
              reprocessing
            </Text>

            <Form.Item
              label="Max Entity Tokens"
              name={getFieldName('querying', 'max_entity_tokens')}
              tooltip="Maximum tokens for entity context (default: 6000)"
            >
              <InputNumber
                placeholder="6000"
                min={1000}
                max={50000}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Max Relation Tokens"
              name={getFieldName('querying', 'max_relation_tokens')}
              tooltip="Maximum tokens for relationship context (default: 8000)"
            >
              <InputNumber
                placeholder="8000"
                min={1000}
                max={50000}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Max Total Tokens"
              name={getFieldName('querying', 'max_total_tokens')}
              tooltip="Maximum total tokens for context (default: 30000)"
            >
              <InputNumber
                placeholder="30000"
                min={5000}
                max={200000}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Max Graph Nodes Per Query"
              name={getFieldName('querying', 'max_graph_nodes_per_query')}
              tooltip="Maximum graph nodes to consider per query (default: 1000)"
            >
              <InputNumber
                placeholder="1000"
                min={10}
                max={5000}
                className="w-full"
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Retrieval</Text>
            </Divider>

            <Form.Item
              label="Top K Results"
              name={getFieldName('querying', 'top_k')}
              tooltip="Maximum number of results to retrieve (default: 40)"
            >
              <InputNumber
                placeholder="40"
                min={1}
                max={200}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Chunk Top K"
              name={getFieldName('querying', 'chunk_top_k')}
              tooltip="Maximum number of chunks to consider (default: 20)"
            >
              <InputNumber
                placeholder="20"
                min={1}
                max={100}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Related Chunk Number"
              name={getFieldName('querying', 'related_chunk_number')}
              tooltip="Number of related chunks to include (default: 5)"
            >
              <InputNumber
                placeholder="5"
                min={0}
                max={20}
                className="w-full"
              />
            </Form.Item>

            <Form.Item
              label="Query Mode"
              name={getFieldName('querying', 'query_mode')}
              tooltip="Graph query mode (default: mix)"
            >
              <Select
                placeholder="Select query mode"
                allowClear
                options={[
                  {
                    value: 'local',
                    label: 'Local - Context-dependent queries',
                  },
                  {
                    value: 'global',
                    label: 'Global - Global knowledge queries',
                  },
                  { value: 'hybrid', label: 'Hybrid - Combined local/global' },
                  { value: 'naive', label: 'Naive - Basic search' },
                  { value: 'mix', label: 'Mix - Knowledge graph + vector' },
                  { value: 'bypass', label: 'Bypass - Direct query' },
                ]}
              />
            </Form.Item>

            <Form.Item
              label="Chunk Selection Method"
              name={getFieldName('querying', 'chunk_selection_method')}
              tooltip="Method for selecting relevant chunks (default: vector)"
            >
              <Select
                placeholder="Select method"
                allowClear
                options={[
                  { value: 'vector', label: 'Vector Similarity' },
                  { value: 'weight', label: 'Weight-based' },
                ]}
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Prompt Templates</Text>
            </Divider>

            <Form.Item
              label="Pre-Query Template"
              name={getFieldName('querying', 'prompt_template_pre_query')}
              tooltip="Prompt template for query refinement before graph search (use {content} for query text, {history} for conversation history)"
            >
              <Input.TextArea
                placeholder="Enter pre-query prompt template..."
                rows={3}
              />
            </Form.Item>

            <Form.Item
              label="Post-Query Template"
              name={getFieldName('querying', 'prompt_template_post_query')}
              tooltip="Prompt template for final response generation after retrieval (use {content} for retrieved context, {history} for conversation history)"
            >
              <Input.TextArea
                placeholder="Enter post-query prompt template..."
                rows={3}
              />
            </Form.Item>

            <Divider orientation="left" orientationMargin="0">
              <Text type="secondary">Reranking</Text>
            </Divider>

            <Form.Item
              label="Enable Reranking"
              name={getFieldName('querying', 'enable_rerank')}
              tooltip="Enable reranking of search results (default: false)"
              valuePropName="checked"
            >
              <Switch />
            </Form.Item>

            <Form.Item
              label="Min Rerank Score"
              name={getFieldName('querying', 'min_rerank_score')}
              tooltip="Minimum score threshold for reranking (default: 0.0)"
            >
              <InputNumber
                placeholder="0.0"
                min={0}
                max={1}
                step={0.1}
                className="w-full"
              />
            </Form.Item>
          </div>
        </Panel>
      </Collapse>
    </div>
  )
}
