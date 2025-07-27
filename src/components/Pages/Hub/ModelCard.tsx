import {
  AppstoreOutlined,
  DownloadOutlined,
  EyeOutlined,
  FileTextOutlined,
  LockOutlined,
  ToolOutlined,
  UnlockOutlined,
} from '@ant-design/icons'
import { App, Button, Card, Flex, Select, Tag, Typography } from 'antd'
import { useState } from 'react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { isDesktopApp } from '../../../api/core.ts'
import type { HubModel } from '../../../types/api/hub'
import { Stores } from '../../../store'
import { repositoryHasCredentials } from '../../../store/repositories.ts'
import { downloadModelFromRepository } from '../../../store/modelDownload'
import { openRepositoryDrawer } from '../../../store/ui'
import { DownloadItem } from '../../shared/DownloadItem.tsx'
import { Provider } from '../../../types'
import { ModelDetailsDrawer } from './ModelDetailsDrawer'

const { Title, Text } = Typography

interface ModelCardProps {
  model: HubModel
}

export function ModelCard({ model }: ModelCardProps) {
  const { message, modal } = App.useApp()
  const { repositories } = Stores.Repositories
  const { providers } = Stores.Providers
  const { downloads } = Stores.ModelDownload
  const [showDetails, setShowDetails] = useState(false)

  // Find active download for this model
  const activeDownload = Object.values(downloads).find(
    download =>
      download.request_data.repository_path === model.repository_path &&
      (download.status === 'downloading' || download.status === 'pending'),
  )

  const isModelBeingDownloaded = !!activeDownload

  const handleDownload = async (model: HubModel) => {
    console.log('Downloading model:', model.id)
    const repo = repositories.find(repo => repo.url === model.repository_url)
    console.log({ repo })
    if (!repo) {
      message.error(
        `Repository not found for model ${model.alias}. Please check the repository configuration.`,
      )
      return
    }

    if (!model.public && !repositoryHasCredentials(repo)) {
      message.info(
        `Model ${model.alias} is private and requires credentials. Please configure the repository with valid credentials.`,
      )

      openRepositoryDrawer(repo)
      return
    }

    const localProviders = providers.filter(
      p => p.type === 'local' && p.enabled,
    )

    if (localProviders.length === 0) {
      message.error(
        `No local provider found for model ${model.alias}. Please ensure a local provider is configured.`,
      )
      return
    }

    let provider: Provider | undefined = localProviders[0]

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
      const downloadRequest = {
        provider_id: provider.id,
        repository_id: repo.id,
        repository_path: model.repository_path,
        main_filename: model.main_filename,
        repository_branch: 'main', // Default branch
        name: modelName,
        alias: model.alias,
        description:
          model.description || `Downloaded from ${model.repository_url}`,
        file_format: model.file_format,
        capabilities: model.capabilities || {},
        parameters: model.recommended_parameters || {},
        settings: {}, // Empty settings for now
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
    if (isDesktopApp) {
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
        key={model.id}
        hoverable
        className="h-full cursor-pointer"
        classNames={{
          body: 'h-full flex flex-col gap-2 !py-1',
        }}
        onClick={handleCardClick}
      >
        <Flex className={'gap-2 w-full flex-col flex-1'}>
          <div>
            <Flex justify="space-between" align="start">
              <Title level={4} className="m-0">
                {model.alias}
              </Title>
              {model.public ? (
                <UnlockOutlined className="text-green-500" />
              ) : (
                <LockOutlined className="text-red-500" />
              )}
            </Flex>
            <Text type="secondary" className="text-xs">
              {model.description}
            </Text>
          </div>

          {/* Tags */}
          <div>
            <Flex wrap className="gap-1">
              {model.tags.slice(0, 3).map(tag => (
                <Tag key={tag} color="default" className="text-xs">
                  {tag}
                </Tag>
              ))}
              {model.tags.length > 3 && (
                <Tag color="default" className="text-xs">
                  +{model.tags.length - 3}
                </Tag>
              )}
            </Flex>
          </div>

          {/* Capabilities */}
          {model.capabilities && (
            <div>
              <Flex wrap className="gap-1">
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
                  <Tag color="blue" icon={<ToolOutlined />} className="text-xs">
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
              </Flex>
            </div>
          )}

          {/* Stats */}
          <div>
            <Flex justify="space-between" align="center" className="mb-1">
              <Text type="secondary" className="text-xs">
                Size: {model.size_gb}GB
              </Text>
              <Text type="secondary" className="text-xs">
                {model.file_format.toUpperCase()}
              </Text>
            </Flex>
            {model.license && (
              <Text type="secondary" className="text-xs">
                License: {model.license}
              </Text>
            )}
          </div>
        </Flex>

        {/* Action Buttons */}
        <div className="mt-auto flex gap-1 flex-col">
          <div className="flex gap-2 mb-2">
            <Button
              size="small"
              icon={<FileTextOutlined />}
              onClick={e => {
                e.stopPropagation()
                handleViewReadme(model)
              }}
              className="flex-1"
            >
              README
            </Button>
            <Button
              type="primary"
              size="small"
              icon={<DownloadOutlined />}
              onClick={e => {
                e.stopPropagation()
                handleDownload(model)
              }}
              className="flex-[2]"
              disabled={isModelBeingDownloaded}
              loading={isModelBeingDownloaded}
            >
              {isModelBeingDownloaded ? 'Downloading...' : 'Download'}
            </Button>
          </div>

          {/* Progress Bar */}
          {isModelBeingDownloaded && activeDownload && (
            <DownloadItem download={activeDownload} mode="compact" />
          )}
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
