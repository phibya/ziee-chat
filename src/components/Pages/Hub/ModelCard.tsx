import {
  AppstoreOutlined,
  DownloadOutlined,
  EyeOutlined,
  FileTextOutlined,
  LockOutlined,
  MessageOutlined,
  PictureOutlined,
  SearchOutlined,
  ToolOutlined,
  UnlockOutlined,
} from '@ant-design/icons'
import { App, Button, Card, Flex, Select, Tag, Typography } from 'antd'
import { useState } from 'react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { isTauriView } from '../../../api/core.ts'
import type { HubModel, HubModelQuantizationOption } from '../../../types'
import { Provider } from '../../../types'
import { Stores } from '../../../store'
import { adminRepositoryHasCredentials } from '../../../store/admin/repositories.ts'
import { downloadModelFromRepository } from '../../../store/admin/modelDownload.ts'
import { openRepositoryDrawer } from '../../../store/ui'
import { DownloadItem } from '../../common/DownloadItem.tsx'
import { ModelDetailsDrawer } from './ModelDetailsDrawer'

const { Text } = Typography

interface ModelCardProps {
  model: HubModel
}

export function ModelCard({ model }: ModelCardProps) {
  const { message, modal } = App.useApp()
  const { repositories } = Stores.AdminRepositories
  const { providers } = Stores.AdminProviders
  const { downloads } = Stores.ModelDownload
  const [showDetails, setShowDetails] = useState(false)

  // Find active download for this model
  const activeDownload = Object.values(downloads).find(
    download =>
      download.request_data.repository_path === model.repository_path &&
      (download.status === 'downloading' || download.status === 'pending'),
  )

  const isModelBeingDownloaded = !!activeDownload

  // Check if this hub model has been downloaded locally
  const isModelDownloaded = providers
    .filter(provider => provider.type === 'local')
    .some(provider =>
      provider.models.some(
        localModel =>
          localModel.source?.type === 'hub' &&
          localModel.source?.id === model.id,
      ),
    )

  const handleDownload = async (model: HubModel) => {
    console.log('Downloading model:', model.id)
    const repo = repositories.find(repo => repo.url === model.repository_url)
    if (!repo) {
      message.error(
        `Repository not found for model ${model.alias}. Please check the repository configuration.`,
      )
      return
    }

    if (!model.public && !adminRepositoryHasCredentials(repo)) {
      message.info(
        `Model ${model.alias} is private and requires credentials. Please configure the repository with valid credentials.`,
      )

      openRepositoryDrawer(repo)
      return
    }

    const localProviders = providers.filter(p => p.type === 'local')

    if (localProviders.length === 0) {
      message.error(
        `No local provider found for model ${model.alias}. Please ensure a local provider is configured.`,
      )
      return
    }

    let provider: Provider | undefined = localProviders[0]
    let selectedFilename = model.main_filename
    let selectedQuantization: HubModelQuantizationOption | undefined = undefined

    // Handle quantization options selection
    if (model.quantization_options && model.quantization_options.length > 1) {
      selectedQuantization = model.quantization_options[0]

      await new Promise<void>(resolve => {
        let m = modal.info({
          icon: null,
          footer: null,
          title: 'Select Quantization',
          closable: false,
          onCancel: () => {
            selectedQuantization = undefined
            resolve()
          },
          content: (
            <div className="flex flex-col gap-2">
              <Text>
                Multiple quantization options available. Please select one:
              </Text>
              <Select
                options={model.quantization_options!.map(option => ({
                  label: (
                    <div className="flex flex-col">
                      <Text strong>{option.name.toUpperCase()}</Text>
                      <Text type="secondary" className="text-xs">
                        Main file: {option.main_filename}
                      </Text>
                    </div>
                  ),
                  value: option.name,
                }))}
                defaultValue={model.quantization_options![0].name}
                onChange={value => {
                  selectedQuantization = model.quantization_options!.find(
                    opt => opt.name === value,
                  )
                }}
                placeholder="Select quantization"
                optionRender={option => option.label}
                labelRender={props => (
                  <Text strong>{props.value?.toString().toUpperCase()}</Text>
                )}
              />
              <Flex className={'gap-2 w-full justify-end'}>
                <Button
                  onClick={() => {
                    selectedQuantization = undefined
                    m.destroy()
                    resolve()
                  }}
                >
                  Cancel
                </Button>
                <Button
                  type="primary"
                  onClick={() => {
                    resolve()
                    m.destroy()
                  }}
                >
                  Continue
                </Button>
              </Flex>
            </div>
          ),
        })
      })

      if (!selectedQuantization) {
        return
      }

      selectedFilename = selectedQuantization.main_filename
    } else if (
      model.quantization_options &&
      model.quantization_options.length === 1
    ) {
      // If only one quantization option, use it
      selectedQuantization = model.quantization_options[0]
      selectedFilename = model.quantization_options[0].main_filename
    }

    if (localProviders.length > 1) {
      await new Promise<void>(resolve => {
        let m = modal.info({
          icon: null,
          footer: null,
          title: 'Select Local Provider',
          closable: false,
          onCancel: () => {
            provider = undefined
            resolve()
          },
          content: (
            <div className="flex flex-col gap-2">
              <Text>
                Multiple local providers found. Please select one to download
                the model:
              </Text>
              <Select
                options={localProviders.map(p => ({
                  label: p.name,
                  value: p.id,
                }))}
                defaultValue={localProviders[0].id}
                onChange={value => {
                  provider = localProviders.find(p => p.id === value)
                }}
                placeholder="Select a provider"
              />
              <Flex className={'gap-2 w-full justify-end'}>
                <Button
                  onClick={() => {
                    provider = undefined
                    m.destroy()
                    resolve()
                  }}
                >
                  Cancel
                </Button>
                <Button
                  type="primary"
                  onClick={() => {
                    resolve()
                    m.destroy()
                  }}
                >
                  Continue
                </Button>
              </Flex>
            </div>
          ),
        })
      })
    }

    if (!provider) {
      return
    }

    try {
      // Generate a unique model name for local storage
      const modelName = `${model.alias
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, '')
        .replace(/\s+/g, '-')}-${Date.now().toString(36)}`

      // Prepare download request
      const alias = selectedQuantization 
        ? `${model.alias} (${selectedQuantization.name.toUpperCase()})`
        : model.alias

      const downloadRequest = {
        provider_id: provider.id,
        repository_id: repo.id,
        repository_path: model.repository_path,
        main_filename: selectedFilename,
        repository_branch: 'main', // Default branch
        name: modelName,
        alias: alias,
        description:
          model.description || `Downloaded from ${model.repository_url}`,
        file_format: model.file_format,
        capabilities: model.capabilities || {},
        parameters: model.recommended_parameters || {},
        settings: {}, // Empty settings for now
        source: {
          type: 'hub' as const,
          id: model.id,
        },
      }

      // Start the download
      await downloadModelFromRepository(downloadRequest)

      message.success(
        `Download started for ${model.alias}. You can monitor the progress in the download view.`,
      )
    } catch (error: any) {
      console.error('Failed to start model download:', error)
      message.error(
        `Failed to start download for ${model.alias}: ${error.message || 'Unknown error'}`,
      )
    }
  }

  const handleViewReadme = (model: HubModel) => {
    // Construct the README URL based on repository type
    const constructReadmeUrl = (model: HubModel): string => {
      const baseUrl = model.repository_url.replace(/\/$/, '')
      const repoPath = model.repository_path

      if (baseUrl.startsWith('https://github.com')) {
        return `${baseUrl}/${repoPath}/blob/main/README.md`
      } else if (baseUrl.startsWith('https://huggingface.co')) {
        return `${baseUrl}/${repoPath}/blob/main/README.md`
      } else {
        // Fallback to the repository URL itself
        return `${baseUrl}/${repoPath}`
      }
    }

    const readmeUrl = constructReadmeUrl(model)
    if (isTauriView) {
      openUrl(readmeUrl).catch(err => {
        console.error(`Failed to open ${readmeUrl}:`, err)
        message.error(`Failed to open ${readmeUrl}`)
      })
    } else {
      window.open(readmeUrl, '_blank', 'noopener,noreferrer')
    }
  }

  const handleCardClick = () => {
    setShowDetails(true)
  }

  const handleCloseDetails = () => {
    setShowDetails(false)
  }

  return (
    <>
      <Card
        hoverable
        className="cursor-pointer relative group hover:!shadow-md transition-shadow h-full"
        onClick={handleCardClick}
      >
        <div className="flex items-start gap-3 flex-wrap">
          {/* Model Info */}
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-2 flex-wrap">
              <div className="flex-1 min-w-48">
                <Flex className="gap-2 items-center">
                  <AppstoreOutlined />
                  <Text
                    className="font-medium cursor-pointer"
                    onClick={handleCardClick}
                  >
                    {model.alias}
                  </Text>
                  {model.public ? (
                    <Tag color="green" icon={<UnlockOutlined />}>
                      Public
                    </Tag>
                  ) : (
                    <Tag color="red" icon={<LockOutlined />}>
                      Private
                    </Tag>
                  )}
                  {isModelBeingDownloaded && (
                    <Tag color="blue">Downloading...</Tag>
                  )}
                  {isModelDownloaded && (
                    <Tag color="geekblue-inverse">Downloaded</Tag>
                  )}
                </Flex>
              </div>
              <div className="flex gap-1 items-center justify-end">
                <Button
                  icon={<FileTextOutlined />}
                  onClick={e => {
                    e.stopPropagation()
                    handleViewReadme(model)
                  }}
                >
                  README
                </Button>
                <Button
                  type="primary"
                  icon={<DownloadOutlined />}
                  onClick={e => {
                    e.stopPropagation()
                    handleDownload(model)
                  }}
                  disabled={isModelBeingDownloaded}
                  loading={isModelBeingDownloaded}
                >
                  Download
                </Button>
              </div>
            </div>

            <div>
              <Text type="secondary" className="text-sm mb-2 block">
                {model.description}
              </Text>

              {/* Capabilities */}
              {model.capabilities && (
                <div className="mb-2">
                  <Text type="secondary" className="text-xs mr-2">
                    Capabilities:
                  </Text>
                  <Flex
                    wrap
                    className="gap-1"
                    style={{ display: 'inline-flex' }}
                  >
                    {model.capabilities.vision && (
                      <Tag
                        color="purple"
                        icon={<EyeOutlined />}
                        className="text-xs"
                      >
                        Vision
                      </Tag>
                    )}
                    {model.capabilities.tools && (
                      <Tag
                        color="blue"
                        icon={<ToolOutlined />}
                        className="text-xs"
                      >
                        Tools
                      </Tag>
                    )}
                    {model.capabilities.code_interpreter && (
                      <Tag
                        color="orange"
                        icon={<AppstoreOutlined />}
                        className="text-xs"
                      >
                        Code
                      </Tag>
                    )}
                    {model.capabilities.chat && (
                      <Tag
                        color="green"
                        icon={<MessageOutlined />}
                        className="text-xs"
                      >
                        Chat
                      </Tag>
                    )}
                    {model.capabilities.text_embedding && (
                      <Tag
                        color="cyan"
                        icon={<SearchOutlined />}
                        className="text-xs"
                      >
                        Embedding
                      </Tag>
                    )}
                    {model.capabilities.image_generator && (
                      <Tag
                        color="magenta"
                        icon={<PictureOutlined />}
                        className="text-xs"
                      >
                        Image Gen
                      </Tag>
                    )}
                  </Flex>
                </div>
              )}

              {/* Tags */}
              {model.tags.length > 0 && (
                <div className="mb-2">
                  <Text type="secondary" className="text-xs mr-2">
                    Tags:
                  </Text>
                  <Flex
                    wrap
                    className="gap-1"
                    style={{ display: 'inline-flex' }}
                  >
                    {model.tags.map(tag => (
                      <Tag key={tag} color="default" className="text-xs">
                        {tag}
                      </Tag>
                    ))}
                  </Flex>
                </div>
              )}

              {/* Metadata */}
              <div className="mb-2">
                <Flex wrap className="gap-x-4 text-xs">
                  <span>
                    <Text type="secondary" className="text-xs">
                      Size:
                    </Text>{' '}
                    {model.size_gb}GB
                  </span>
                  <span>
                    <Text type="secondary" className="text-xs">
                      Format:
                    </Text>{' '}
                    {model.file_format.toUpperCase()}
                  </span>
                  {model.license && (
                    <span>
                      <Text type="secondary" className="text-xs">
                        License:
                      </Text>{' '}
                      {model.license}
                    </span>
                  )}
                </Flex>
              </div>

              {/* Progress Bar */}
              {isModelBeingDownloaded && activeDownload && (
                <div className="mt-2">
                  <DownloadItem download={activeDownload} mode="compact" />
                </div>
              )}
            </div>
          </div>
        </div>
      </Card>

      <ModelDetailsDrawer
        model={model}
        open={showDetails}
        onClose={handleCloseDetails}
      />
    </>
  )
}
