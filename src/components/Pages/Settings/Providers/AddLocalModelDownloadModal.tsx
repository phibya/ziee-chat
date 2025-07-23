import {
  App,
  Button,
  Card,
  Form,
  Input,
  Modal,
  Progress,
  Select,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ApiClient } from '../../../../api/client'
import {
  clearModelDownload,
  clearProvidersError,
  closeAddLocalModelDownloadModal,
  closeViewDownloadModal,
  downloadModelFromRepository,
  findDownloadById,
  Stores,
} from '../../../../store'
import { Repository } from '../../../../types/api/repository'
import { LocalModelCommonFields } from './shared/LocalModelCommonFields'

const { Text } = Typography

export function AddLocalModelDownloadModal() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)
  const [repositories, setRepositories] = useState<Repository[]>([])
  const [loadingRepositories, setLoadingRepositories] = useState(false)
  const [isInViewMode, setIsInViewMode] = useState(false)
  const [currentDownloadId, setCurrentDownloadId] = useState<string | null>(
    null,
  )

  // Function to generate a unique model ID from display name
  const generateModelId = (displayName: string): string => {
    const baseId = displayName
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$/g, '')
      .substring(0, 50)

    const timestamp = Date.now().toString(36)
    return `${baseId}-${timestamp}`
  }

  // Get values from form
  const selectedRepository = Form.useWatch('repository_id', form)

  // Load available repositories
  const loadRepositories = async () => {
    try {
      setLoadingRepositories(true)
      const response = await ApiClient.Repositories.list({})
      const enabledRepos = response.repositories.filter(repo => repo.enabled)
      setRepositories(enabledRepos)
    } catch (error) {
      console.error('Failed to load repositories:', error)
      message.error(t('providers.failedToLoadRepositories'))
    } finally {
      setLoadingRepositories(false)
    }
  }

  const { downloads } = Stores.ModelDownload
  const { open: addMode, providerId } = Stores.UI.AddLocalModelDownloadModal
  const { downloadId, open: viewMode } = Stores.UI.ViewDownloadModal

  const open = viewMode || addMode

  // Helper function to close the appropriate modal
  const handleCloseModal = () => {
    if (viewMode) {
      closeViewDownloadModal()
    } else if (addMode) {
      closeAddLocalModelDownloadModal()
    }
  }

  // Get download instance - either external download or current download
  const viewDownload =
    viewMode && downloadId ? findDownloadById(downloadId) : null
  const internalDownload = currentDownloadId
    ? downloads[currentDownloadId]
    : null
  const currentDownload = viewMode ? viewDownload : internalDownload
  const downloading = currentDownload?.downloading ?? false
  const downloadProgress = currentDownload?.progress ?? null

  const handleSubmit = async () => {
    try {
      setLoading(true)
      clearProvidersError()
      const values = await form.validateFields()

      // Auto-generate model ID from display name
      const modelId = generateModelId(values.alias || 'model')

      if (!values.repository_id) {
        message.error(t('providers.repositoryRequired'))
        return
      }

      if (!values.repository_path) {
        message.error(t('providers.repositoryPathRequired'))
        return
      }

      // Get the selected repository details
      const selectedRepo = repositories.find(
        repo => repo.id === values.repository_id,
      )
      if (!selectedRepo) {
        message.error(t('providers.repositoryNotFound'))
        return
      }

      // Call the repository download API through store
      try {
        const { downloadId } = await downloadModelFromRepository({
          provider_id: providerId!,
          repository_id: values.repository_id,
          repository_path: values.repository_path,
          main_filename: values.main_filename,
          repository_branch: values.repository_branch,
          name: modelId,
          alias: values.alias,
          description: values.description,
          file_format: values.file_format,
          capabilities: values.capabilities || {},
          settings: values.settings || {},
        })

        // Track this download and switch to view mode
        setCurrentDownloadId(downloadId)
        setIsInViewMode(true)

        message.success(t('providers.downloadStarted'))
      } catch (error) {
        console.error('Failed to start download:', error)
        message.error(t('providers.downloadStartFailed'))
      }
    } catch (error) {
      console.error('Failed to create model:', error)
      message.error(t('providers.failedToCreateModel'))
    } finally {
      setLoading(false)
    }
  }

  const handleCancel = () => {
    form.resetFields()
    if (isInViewMode) {
      setIsInViewMode(false)
      setCurrentDownloadId(null)
    }
    handleCloseModal()
  }

  const handleBackToAddMode = () => {
    setIsInViewMode(false)
    setCurrentDownloadId(null)
  }

  // Current view mode state
  const currentViewMode = viewMode || isInViewMode

  // Load repositories and pre-fill form when modal opens
  useEffect(() => {
    if (open) {
      loadRepositories()
      if (viewMode && viewDownload) {
        // In view mode, populate form with download data
        form.setFieldsValue({
          alias: viewDownload.request.alias,
          description: viewDownload.request.description || '',
          file_format: viewDownload.request.file_format,
          repository_path: viewDownload.request.repository_path,
          main_filename: viewDownload.request.main_filename,
          repository_branch: viewDownload.request.repository_branch || 'main',
          capabilities: viewDownload.request.capabilities || {},
          settings: viewDownload.request.settings || {},
        })
      } else if (!viewMode && !currentViewMode) {
        // In add mode, set default values
        form.setFieldsValue({
          alias: 'TinyLlama Chat Model',
          description:
            'Small 1.1B parameter chat model for quick testing (~637MB)',
          file_format: 'safetensors',
          repository_path: 'meta-llama/Llama-3.1-8B-Instruct',
          main_filename: 'model.safetensors',
          repository_branch: 'main',
          settings: {},
        })
      }
    }
  }, [open, viewMode, viewDownload, currentViewMode, form])

  return (
    <Modal
      title={
        currentViewMode
          ? 'View Download Details'
          : t('providers.downloadLocalModel')
      }
      open={open}
      onCancel={handleCancel}
      footer={
        currentViewMode
          ? [
              <Button key="close" onClick={handleCloseModal}>
                {t('buttons.close')}
              </Button>,
              !viewMode && !currentDownload?.downloading && (
                <Button key="back" onClick={handleBackToAddMode}>
                  Add Another Model
                </Button>
              ),
              !viewMode && currentDownload?.downloading && (
                <Button
                  key="cancel-download"
                  danger
                  onClick={() => {
                    if (currentDownloadId) {
                      clearModelDownload(currentDownloadId)
                      setCurrentDownloadId(null)
                      setIsInViewMode(false)
                    }
                  }}
                >
                  {t('buttons.cancel')} Download
                </Button>
              ),
            ].filter(Boolean)
          : [
              <Button key="cancel" onClick={handleCancel}>
                {t('buttons.cancel')}
              </Button>,
              <Button
                key="submit"
                type="primary"
                loading={loading}
                onClick={handleSubmit}
              >
                {t('buttons.startDownload')}
              </Button>,
            ]
      }
      width={800}
      destroyOnHidden={true}
      maskClosable={false}
    >
      {currentViewMode && currentDownload ? (
        <Card title={t('providers.downloadProgress')} size="small">
          <Progress
            percent={Math.round(
              ((currentDownload.progress?.current || 0) /
                (currentDownload.progress?.total || 1)) *
                100,
            )}
            status={currentDownload.downloading ? 'active' : 'success'}
            format={percent =>
              `${percent}% - ${currentDownload.progress?.phase || ''}`
            }
          />
          <Text type="secondary" style={{ fontSize: '12px' }}>
            {currentDownload.progress?.message || ''}
          </Text>
        </Card>
      ) : (
        <Form
          form={form}
          layout="vertical"
          disabled={currentViewMode}
          initialValues={{
            file_format: 'safetensors',
            main_filename: '',
            repository_branch: 'main',
            settings: {},
          }}
        >
          <LocalModelCommonFields />

          <Form.Item
            name="repository_id"
            label={t('providers.selectRepository')}
            rules={[
              {
                required: true,
                message: t('providers.repositoryRequired'),
              },
            ]}
          >
            <Select
              placeholder={t('providers.selectRepositoryPlaceholder')}
              loading={loadingRepositories}
              showSearch
              optionFilterProp="children"
              options={repositories.map(repo => ({
                value: repo.id,
                label: `${repo.name} (${repo.url})`,
              }))}
            />
          </Form.Item>

          <Form.Item
            name="repository_path"
            label={t('providers.repositoryPath')}
            rules={[
              {
                required: true,
                message: t('providers.repositoryPathRequired'),
              },
            ]}
          >
            <Input
              placeholder="microsoft/DialoGPT-medium"
              addonBefore={
                selectedRepository
                  ? repositories.find(r => r.id === selectedRepository)?.url ||
                    'Repository'
                  : 'Repository'
              }
            />
          </Form.Item>

          <Form.Item
            name="main_filename"
            label={t('providers.mainFilename')}
            rules={[
              {
                required: true,
                message: t('providers.localFilenameRequired'),
              },
            ]}
          >
            <Input placeholder="model.safetensors" />
          </Form.Item>

          <Form.Item
            name="repository_branch"
            label={t('providers.repositoryBranch')}
          >
            <Input placeholder="main" />
          </Form.Item>

          {downloading && downloadProgress && (
            <Card title={t('providers.downloadProgress')} size="small">
              <Progress
                percent={Math.round(
                  (downloadProgress.current / downloadProgress.total) * 100,
                )}
                status={downloading ? 'active' : 'success'}
                format={percent => `${percent}% - ${downloadProgress.phase}`}
              />
              <Text type="secondary" style={{ fontSize: '12px' }}>
                {downloadProgress.message}
              </Text>
            </Card>
          )}
        </Form>
      )}
    </Modal>
  )
}
