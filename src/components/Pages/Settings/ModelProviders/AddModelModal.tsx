import {
  App,
  Button,
  Form,
  Input,
  Modal,
  Radio,
  Select,
  Typography,
  Upload,
} from 'antd'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ModelProviderType } from '../../../../types/api/modelProvider'
import { useModelProvidersStore } from '../../../../store/modelProviders'
import { useShallow } from 'zustand/react/shallow'
import { UploadOutlined } from '@ant-design/icons'
import { ModelCapabilitiesSection } from './shared/ModelCapabilitiesSection'
import { ModelParametersSection } from './shared/ModelParametersSection'
import { BASIC_MODEL_FIELDS, CANDLE_PARAMETERS } from './shared/constants'
import {
  CANDLE_ARCHITECTURE_OPTIONS,
  CANDLE_FILE_TYPE_OPTIONS,
  DEFAULT_CANDLE_ARCHITECTURE,
  DEFAULT_CANDLE_FILE_TYPE,
} from '../../../../constants/candleModelTypes'

interface AddModelModalProps {
  open: boolean
  providerId: string
  providerType: ModelProviderType
  provider?: any
  onClose: () => void
  onSubmit: (modelData: any) => void
}

export function AddModelModal({
  open,
  providerId,
  providerType,
  provider,
  onClose,
  onSubmit,
}: AddModelModalProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)
  const [selectedFiles, setSelectedFiles] = useState<File[]>([])
  const [selectedFileFormat, setSelectedFileFormat] = useState(
    DEFAULT_CANDLE_FILE_TYPE,
  )
  const [modelSource, setModelSource] = useState<'upload' | 'huggingface'>(
    'upload',
  )

  const { createUploadModel, uploadModelFiles } = useModelProvidersStore(
    useShallow(state => ({
      createUploadModel: state.createUploadModel,
      uploadModelFiles: state.uploadModelFiles,
    })),
  )

  const handleSubmit = async () => {
    try {
      setLoading(true)
      const values = await form.validateFields()

      if (providerType === 'candle') {
        if (values.model_source === 'upload') {
          // For folder upload workflow
          if (selectedFiles.length === 0) {
            message.error(t('modelProviders.selectModelFolderRequired'))
            return
          }

          if (!values.local_filename) {
            message.error(t('modelProviders.localFilenameRequired'))
            return
          }

          // Validate that the specified main file exists in selected files
          const mainFile = selectedFiles.find(
            file => file.name === values.local_filename,
          )
          if (!mainFile) {
            message.error(t('modelProviders.mainFileNotFound'))
            return
          }

          // Create the upload model record with folder metadata
          const uploadResult = await createUploadModel(
            providerId,
            values.name,
            values.alias,
            values.description,
            values.architecture,
            values.file_format,
            {
              source: 'local_folder',
              folder_path: values.local_folder_path,
              main_filename: values.local_filename,
              total_files: selectedFiles.length,
              file_list: selectedFiles.map(f => f.name),
            },
          )

          // Upload all the files
          await uploadModelFiles(
            uploadResult.id,
            selectedFiles,
            values.local_filename,
          )

          message.success(t('modelProviders.modelFolderUploadedSuccessfully'))
        } else if (values.model_source === 'huggingface') {
          // For Hugging Face download workflow
          // Get HF token from provider settings
          const hfToken =
            provider?.settings?.huggingFaceAccessToken || provider?.api_key

          await createUploadModel(
            providerId,
            values.name,
            values.alias,
            values.description,
            values.architecture,
            values.file_format,
            {
              source: 'huggingface',
              hf_repo: values.hf_repo,
              hf_filename: values.hf_filename,
              hf_branch: values.hf_branch || 'main',
              hf_token: hfToken,
            },
          )

          message.success(t('modelProviders.modelDownloadStarted'))
        }
      } else {
        // For other providers, use the existing workflow
        const modelData = {
          id: `model-${Date.now()}`,
          ...values,
          enabled: true,
          capabilities: {
            vision: values.vision || false,
            audio: values.audio || false,
            tools: values.tools || false,
            codeInterpreter: values.codeInterpreter || false,
          },
        }

        // Remove capability checkboxes from main data
        delete modelData.vision
        delete modelData.audio
        delete modelData.tools
        delete modelData.codeInterpreter

        await onSubmit(modelData)
      }

      form.resetFields()
      setSelectedFiles([])
      onClose()
    } catch (error) {
      console.error('Failed to add model:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleFolderSelect = (info: any) => {
    const fileList = info.fileList || []
    const files = fileList.map(
      (file: any) => file.originFileObj || file.file || file,
    )

    if (files.length > 0) {
      // Get the common folder path from the first file
      const firstFile = files[0]
      let folderPath = ''

      if (firstFile.webkitRelativePath) {
        const pathParts = firstFile.webkitRelativePath.split('/')
        folderPath = pathParts.slice(0, -1).join('/')
      } else if (firstFile.path) {
        const pathParts = firstFile.path.split('/')
        folderPath = pathParts.slice(0, -1).join('/')
      }

      setSelectedFiles(files)
      form.setFieldsValue({
        local_folder_path: folderPath || 'Selected folder',
      })

      // Find model files and suggest the main one
      const modelFiles = files.filter((file: File) => {
        const name = file.name.toLowerCase()
        return (
          name.includes('model') ||
          name.includes('pytorch') ||
          name.endsWith('.bin') ||
          name.endsWith('.safetensors') ||
          name.endsWith('.gguf')
        )
      })

      if (modelFiles.length > 0) {
        form.setFieldsValue({
          local_filename: modelFiles[0].name,
        })
      }

      message.success(`Selected ${files.length} files from folder`)
    }
  }

  const handleFileFormatChange = (value: string) => {
    setSelectedFileFormat(value)

    // Clear the current filename when format changes to guide user
    form.setFieldsValue({
      local_filename: '',
      hf_filename: '',
    })

    console.log(
      'File format changed to:',
      value,
      'Current format:',
      selectedFileFormat,
    )
  }

  const getFilenamePlaceholder = (fileFormat: string) => {
    switch (fileFormat) {
      case 'safetensors':
        return 'model.safetensors'
      case 'pytorch':
        return 'pytorch_model.bin'
      case 'gguf':
        return 'model.gguf'
      default:
        return 'pytorch_model.bin'
    }
  }

  const validateFilename = (filename: string, fileFormat: string) => {
    if (!filename) return false

    const validExtensions = {
      safetensors: ['.safetensors'],
      pytorch: ['.bin', '.pt', '.pth'],
      gguf: ['.gguf'],
    }

    const extensions = validExtensions[
      fileFormat as keyof typeof validExtensions
    ] || ['.bin']
    return extensions.some(ext => filename.toLowerCase().endsWith(ext))
  }

  return (
    <Modal
      title={t('modelProviders.addModel')}
      open={open}
      onCancel={onClose}
      footer={[
        <Button key="cancel" onClick={onClose}>
          {t('buttons.cancel')}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
        >
          {t('modelProviders.addModel')}
        </Button>,
      ]}
      width={600}
      maskClosable={false}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          architecture: DEFAULT_CANDLE_ARCHITECTURE,
          file_format: DEFAULT_CANDLE_FILE_TYPE,
          model_source: 'upload',
          local_folder_path: '',
          local_filename: '',
          parameters: {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 50,
            max_tokens: 512,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
          },
        }}
      >
        <ModelParametersSection parameters={BASIC_MODEL_FIELDS} />

        {providerType === 'candle' && (
          <Form.Item
            name="architecture"
            label={t('modelProviders.modelArchitecture')}
            rules={[
              {
                required: true,
                message: t('modelProviders.modelArchitectureRequired'),
              },
            ]}
          >
            <Select
              placeholder={t('modelProviders.selectModelArchitecture')}
              options={CANDLE_ARCHITECTURE_OPTIONS.map(option => ({
                value: option.value,
                label: option.label,
                description: option.description,
              }))}
              optionRender={option => (
                <div className={'flex flex-col'}>
                  <Typography.Text>{option.label}</Typography.Text>
                  <Typography.Text type="secondary">
                    {option.data.description}
                  </Typography.Text>
                </div>
              )}
            />
          </Form.Item>
        )}

        {providerType === 'candle' && (
          <Form.Item
            name="model_source"
            label={t('modelProviders.modelSource')}
            rules={[
              {
                required: true,
                message: t('modelProviders.modelSourceRequired'),
              },
            ]}
          >
            <Radio.Group
              onChange={e => setModelSource(e.target.value)}
              value={modelSource}
            >
              <Radio value="upload">{t('modelProviders.uploadFile')}</Radio>
              <Radio value="huggingface">
                {t('modelProviders.downloadFromHuggingFace')}
              </Radio>
            </Radio.Group>
          </Form.Item>
        )}

        {providerType === 'candle' && (
          <Form.Item
            name="file_format"
            label={t('modelProviders.fileFormat')}
            rules={[
              {
                required: true,
                message: t('modelProviders.fileFormatRequired'),
              },
            ]}
          >
            <Select
              placeholder={t('modelProviders.selectFileFormat')}
              onChange={handleFileFormatChange}
              options={CANDLE_FILE_TYPE_OPTIONS.map(option => ({
                value: option.value,
                label: option.label,
                description: option.description,
              }))}
              optionRender={option => (
                <div className={'flex flex-col'}>
                  <Typography.Text>{option.label}</Typography.Text>
                  <Typography.Text type="secondary">
                    {option.data.description}
                  </Typography.Text>
                </div>
              )}
            />
          </Form.Item>
        )}

        {providerType === 'candle' && modelSource === 'upload' && (
          <>
            <Form.Item
              name="local_folder_path"
              label={t('modelProviders.localFolderPath')}
              rules={[
                {
                  required: true,
                  message: t('modelProviders.selectModelFolderRequired'),
                },
              ]}
            >
              <Input
                placeholder={t('modelProviders.selectModelFolder')}
                addonBefore="📁"
                addonAfter={
                  <Upload
                    showUploadList={false}
                    beforeUpload={() => false}
                    onChange={handleFolderSelect}
                    directory
                    multiple
                  >
                    <Button icon={<UploadOutlined />} size="small">
                      {t('modelProviders.browse')}
                    </Button>
                  </Upload>
                }
              />
            </Form.Item>

            <Form.Item
              name="local_filename"
              label={t('modelProviders.localFilename')}
              rules={[
                {
                  required: true,
                  message: t('modelProviders.localFilenameRequired'),
                },
                {
                  validator: (_, value) => {
                    if (!value) return Promise.resolve()
                    if (validateFilename(value, selectedFileFormat)) {
                      return Promise.resolve()
                    }
                    const placeholder = getFilenamePlaceholder(selectedFileFormat)
                    return Promise.reject(
                      new Error(`Filename must match selected format (e.g., ${placeholder})`),
                    )
                  },
                },
              ]}
              help={t('modelProviders.localFilenameHelp')}
            >
              <Input placeholder={getFilenamePlaceholder(selectedFileFormat)} />
            </Form.Item>
          </>
        )}

        {providerType === 'candle' && modelSource === 'huggingface' && (
          <>
            <Form.Item
              name="hf_repo"
              label={t('modelProviders.huggingFaceRepo')}
              rules={[
                {
                  required: true,
                  message: t('modelProviders.huggingFaceRepoRequired'),
                },
                {
                  pattern: /^[a-zA-Z0-9_-]+\/[a-zA-Z0-9_.-]+$/,
                  message: t('modelProviders.huggingFaceRepoFormat'),
                },
              ]}
            >
              <Input placeholder="microsoft/DialoGPT-medium" addonBefore="🤗" />
            </Form.Item>

            <Form.Item
              name="hf_filename"
              label={t('modelProviders.huggingFaceFilename')}
              rules={[
                {
                  required: true,
                  message: t('modelProviders.huggingFaceFilenameRequired'),
                },
                {
                  validator: (_, value) => {
                    if (!value) return Promise.resolve()
                    if (validateFilename(value, selectedFileFormat)) {
                      return Promise.resolve()
                    }
                    const placeholder = getFilenamePlaceholder(selectedFileFormat)
                    return Promise.reject(
                      new Error(`Filename must match selected format (e.g., ${placeholder})`),
                    )
                  },
                },
              ]}
            >
              <Input placeholder={getFilenamePlaceholder(selectedFileFormat)} />
            </Form.Item>

            <Form.Item
              name="hf_branch"
              label={t('modelProviders.huggingFaceBranch')}
            >
              <Input placeholder="main" />
            </Form.Item>
          </>
        )}

        <ModelCapabilitiesSection />

        {providerType === 'candle' && (
          <ModelParametersSection
            title={t('modelProviders.parameters')}
            parameters={CANDLE_PARAMETERS}
          />
        )}
      </Form>
    </Modal>
  )
}
