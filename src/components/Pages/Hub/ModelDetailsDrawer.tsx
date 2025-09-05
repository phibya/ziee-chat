import {
  AppstoreOutlined,
  EyeOutlined,
  FileTextOutlined,
  LockOutlined,
  MessageOutlined,
  PictureOutlined,
  SearchOutlined,
  ToolOutlined,
  UnlockOutlined,
} from '@ant-design/icons'
import { Card, Flex, Tag, Typography } from 'antd'
import { Drawer } from '../../common/Drawer'
import type { HubModel } from '../../../types'

const { Title, Text } = Typography

interface ModelDetailsDrawerProps {
  model: HubModel | null
  open: boolean
  onClose: () => void
}

export function ModelDetailsDrawer({
  model,
  open,
  onClose,
}: ModelDetailsDrawerProps) {
  if (!model) return null

  return (
    <Drawer title={model.alias} open={open} onClose={onClose} width={600}>
      <Flex vertical className="gap-4">
        {/* Basic Info */}
        <div>
          <Flex justify="space-between" align="center" className="mb-2">
            <Title level={3} className="m-0">
              {model.alias}
            </Title>
            {model.public ? (
              <Tag color="green" icon={<UnlockOutlined />}>
                Public
              </Tag>
            ) : (
              <Tag color="red" icon={<LockOutlined />}>
                Private
              </Tag>
            )}
          </Flex>
          {model.description && (
            <Text type="secondary">{model.description}</Text>
          )}
        </div>

        {/* Repository Information */}
        <div>
          <Title level={5}>Repository Information</Title>
          <Flex vertical className="gap-2">
            <Flex justify="space-between">
              <Text type="secondary">Repository URL:</Text>
              <Text className="text-right">{model.repository_url}</Text>
            </Flex>
            <Flex justify="space-between">
              <Text type="secondary">Repository Path:</Text>
              <Text className="text-right">{model.repository_path}</Text>
            </Flex>
            <Flex justify="space-between">
              <Text type="secondary">Main Filename:</Text>
              <Text className="text-right">{model.main_filename}</Text>
            </Flex>
          </Flex>
        </div>

        {/* Model Details */}
        <div>
          <Title level={5}>Model Details</Title>
          <Flex vertical className="gap-2">
            <Flex justify="space-between">
              <Text type="secondary">File Format:</Text>
              <Tag color="blue">{model.file_format.toUpperCase()}</Tag>
            </Flex>
            <Flex justify="space-between">
              <Text type="secondary">Size:</Text>
              <Text>{model.size_gb}GB</Text>
            </Flex>
            {model.license && (
              <Flex justify="space-between">
                <Text type="secondary">License:</Text>
                <Text>{model.license}</Text>
              </Flex>
            )}
            {model.context_length && (
              <Flex justify="space-between">
                <Text type="secondary">Context Length:</Text>
                <Text>{model.context_length.toLocaleString()} tokens</Text>
              </Flex>
            )}
            {model.popularity_score && (
              <Flex justify="space-between">
                <Text type="secondary">Popularity Score:</Text>
                <Text>{model.popularity_score}</Text>
              </Flex>
            )}
          </Flex>
        </div>

        {/* Capabilities */}
        {model.capabilities && (
          <div>
            <Title level={5}>Capabilities</Title>
            <Flex wrap className="gap-2">
              {model.capabilities.vision && (
                <Tag color="purple" icon={<EyeOutlined />}>
                  Vision
                </Tag>
              )}
              {model.capabilities.tools && (
                <Tag color="blue" icon={<ToolOutlined />}>
                  Tools
                </Tag>
              )}
              {model.capabilities.code_interpreter && (
                <Tag color="orange" icon={<AppstoreOutlined />}>
                  Code Interpreter
                </Tag>
              )}
              {model.capabilities.audio && (
                <Tag color="green" icon={<FileTextOutlined />}>
                  Audio
                </Tag>
              )}
              {model.capabilities.chat && (
                <Tag color="cyan" icon={<MessageOutlined />}>
                  Chat
                </Tag>
              )}
              {model.capabilities.text_embedding && (
                <Tag color="gold" icon={<SearchOutlined />}>
                  Text Embedding
                </Tag>
              )}
              {model.capabilities.image_generator && (
                <Tag color="magenta" icon={<PictureOutlined />}>
                  Image Generator
                </Tag>
              )}
            </Flex>
          </div>
        )}

        {/* Tags */}
        {model.tags.length > 0 && (
          <div>
            <Title level={5}>Tags</Title>
            <Flex wrap className="gap-1">
              {model.tags.map(tag => (
                <Tag key={tag} color="default">
                  {tag}
                </Tag>
              ))}
            </Flex>
          </div>
        )}

        {/* Language Support */}
        {model.language_support && model.language_support.length > 0 && (
          <div>
            <Title level={5}>Language Support</Title>
            <Flex wrap className="gap-1">
              {model.language_support.map(lang => (
                <Tag key={lang} color="cyan">
                  {lang}
                </Tag>
              ))}
            </Flex>
          </div>
        )}

        {/* Quantization Options */}
        {model.quantization_options &&
          model.quantization_options.length > 0 && (
            <div>
              <Title level={5}>Quantization Options</Title>
              <Flex wrap className="gap-1">
                {model.quantization_options.map(option => (
                  <Tag key={option.name} color="gold">
                    {option.name}
                  </Tag>
                ))}
              </Flex>
            </div>
          )}

        {/* Recommended Parameters */}
        {model.recommended_parameters &&
          Object.keys(model.recommended_parameters).length > 0 && (
            <div>
              <Title level={5}>Recommended Parameters</Title>
              <Card size="small">
                <pre className="text-xs overflow-auto m-0">
                  {JSON.stringify(model.recommended_parameters, null, 2)}
                </pre>
              </Card>
            </div>
          )}
      </Flex>
    </Drawer>
  )
}
