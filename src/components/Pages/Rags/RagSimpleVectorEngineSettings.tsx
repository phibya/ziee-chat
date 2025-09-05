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
  'simple_vector',
  section,
  field,
]

export const RagSimpleVectorEngineSettings: React.FC = () => {
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
                      child?.label
                        ?.toLowerCase()
                        .includes(input.toLowerCase()),
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
                      child?.label
                        ?.toLowerCase()
                        .includes(input.toLowerCase()),
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

            <Form.Item
              label="Cosine Similarity Threshold"
              name={getFieldName('indexing', 'cosine_better_than_threshold')}
              tooltip="Minimum cosine similarity score for relevance (default: 0.2)"
            >
              <InputNumber
                placeholder="0.2"
                min={0}
                max={1}
                step={0.1}
                className="w-full"
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
              label="Similarity Threshold"
              name={getFieldName('querying', 'similarity_threshold')}
              tooltip="Minimum similarity score for query results (default: 0.2)"
            >
              <InputNumber
                placeholder="0.2"
                min={0}
                max={1}
                step={0.1}
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
              label="Max Total Tokens"
              name={getFieldName('querying', 'max_total_tokens')}
              tooltip="Maximum total tokens for context (default: 30000)"
            >
              <InputNumber
                placeholder="30000"
                min={1000}
                max={100000}
                className="w-full"
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

            <Form.Item
              label="User Prompt"
              name={getFieldName('querying', 'user_prompt')}
              tooltip="Custom prompt template for queries"
            >
              <Input.TextArea
                placeholder="Enter custom prompt template..."
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
