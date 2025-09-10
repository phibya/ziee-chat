import { UploadOutlined } from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Form,
  List,
  Progress,
  Select,
  Tag,
  Typography,
  Upload,
} from 'antd'
import { Drawer } from '../../../common/Drawer'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { LOCAL_FILE_TYPE_OPTIONS } from '../../../../constants/localModelTypes'
import {
  clearLocalUploadError,
  closeAddLocalModelUploadDrawer,
  Stores,
  uploadLocalModel,
} from '../../../../store'
import { formatBytes } from '../../../../utils/downloadUtils'
import { LocalModelCommonFields } from './common/LocalModelCommonFields'

const { Text } = Typography

export function AddLocalModelUploadDrawer() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)
  const [selectedFiles, setSelectedFiles] = useState<File[]>([])
  const [filteredFiles, setFilteredFiles] = useState<
    { file: File; purpose: string; required: boolean }[]
  >([])

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
  const selectedFileFormat = Form.useWatch('file_format', form) || 'safetensors'

  const { uploading, uploadProgress, overallUploadProgress } =
    Stores.LocalUpload
  const { open, providerId } = Stores.UI.AddLocalModelUploadDrawer

  const handleSubmit = async () => {
    try {
      setLoading(true)
      clearLocalUploadError()
      const values = await form.validateFields()

      // Auto-generate model ID from display name
      const modelId = generateModelId(values.display_name || 'model')

      if (selectedFiles.length === 0) {
        message.error(t('providers.selectModelFolderRequired'))
        return
      }

      if (!values.main_filename) {
        message.error(t('providers.localFilenameRequired'))
        return
      }

      // Comprehensive validation of selected files
      const validation = validateModelFiles(selectedFiles, values.file_format)

      if (!validation.isValid) {
        validation.errors.forEach(error => {
          message.error(error)
        })
        return
      }

      // Show warnings but allow upload to continue
      if (validation.warnings.length > 0) {
        validation.warnings.forEach(warning => {
          message.warning(warning)
        })
      }

      // Validate that the specified main file exists in filtered files
      const filesToUpload = filteredFiles.map(item => item.file)
      const mainFile = filesToUpload.find(
        file => file.name === values.main_filename,
      )
      if (!mainFile) {
        message.error(t('providers.mainFileNotFound'))
        return
      }

      // Upload and auto-commit the files as a model in a single request
      await uploadLocalModel({
        name: modelId,
        provider_id: providerId!,
        display_name: values.display_name,
        description: values.description,
        main_filename: values.main_filename,
        file_format: values.file_format,
        capabilities: values.capabilities || {},
        engine_type: values.engine_type || 'mistralrs',
        engine_settings: values.engine_settings || {},
        files: filesToUpload,
      })

      message.success(t('providers.modelFolderUploadedSuccessfully'))
      form.resetFields()
      setSelectedFiles([])
      setFilteredFiles([])
      closeAddLocalModelUploadDrawer()
    } catch (error) {
      console.error('Failed to upload model:', error)
      message.error(t('providers.failedToCreateModel'))
    } finally {
      setLoading(false)
    }
  }

  const handleCancel = () => {
    // Prevent closing if upload is in progress
    if (uploading) {
      message.warning(t('providers.uploadInProgressCannotClose'))
      return
    }
    form.resetFields()
    setSelectedFiles([])
    setFilteredFiles([])
    closeAddLocalModelUploadDrawer()
  }

  // Validation function for model files
  const validateModelFiles = (
    files: File[],
    fileFormat: string,
  ): { isValid: boolean; errors: string[]; warnings: string[] } => {
    const errors: string[] = []
    const warnings: string[] = []

    // Get expected extensions based on file format
    const expectedExtensions =
      LOCAL_FILE_TYPE_OPTIONS.find(option => option.value === fileFormat)
        ?.extensions || []

    // Check for main model files
    const hasMainFile = files.some(file =>
      expectedExtensions.some(ext => file.name.endsWith(ext)),
    )

    if (!hasMainFile) {
      errors.push(
        `No main model file found with expected extensions: ${expectedExtensions.join(', ')}`,
      )
    }

    // Check for potentially useful files
    const hasTokenizerFiles = files.some(
      file =>
        file.name.includes('tokenizer') ||
        file.name.endsWith('.json') ||
        file.name.endsWith('.txt'),
    )

    if (!hasTokenizerFiles) {
      warnings.push(
        'No tokenizer or configuration files detected. Model may not work properly.',
      )
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    }
  }

  const file_format = Form.useWatch('file_format', form)
  useEffect(() => {
    if (selectedFiles.length > 0) {
      const newFilteredFiles = filterFilesByFormat(selectedFiles, file_format)
      setFilteredFiles(newFilteredFiles)
    }
  }, [file_format])

  // Filter files based on the selected format
  const filterFilesByFormat = (
    files: File[],
    format: string,
  ): { file: File; purpose: string; required: boolean }[] => {
    return files.map(file => {
      let purpose = 'other'
      let required = false

      const fileName = file.name.toLowerCase()
      const fileExtension = fileName.split('.').pop() || ''

      // Determine file purpose based on name and extension
      if (fileName.includes('tokenizer')) {
        purpose = 'tokenizer'
        required = true
      } else if (fileName.endsWith('.json')) {
        if (fileName.includes('config')) {
          purpose = 'config'
          required = true
        } else {
          purpose = 'metadata'
        }
      } else if (fileName.endsWith('.txt')) {
        purpose = 'vocab'
      } else {
        // Check if it matches the selected format
        const formatOptions = LOCAL_FILE_TYPE_OPTIONS.find(
          opt => opt.value === format,
        )
        if (formatOptions?.extensions.includes(`.${fileExtension}`)) {
          purpose = 'model'
          required = true
        }
      }

      return { file, purpose, required }
    })
  }

  // Handle folder upload
  const handleFolderUpload = (info: any) => {
    const files = info.fileList.map((item: any) => item.originFileObj)
    setSelectedFiles(files)

    // Filter files based on current format
    const filtered = filterFilesByFormat(files, selectedFileFormat)
    setFilteredFiles(filtered)

    // Auto-detect main file
    const mainFiles = filtered.filter(
      item => item.purpose === 'model' && item.required,
    )
    if (mainFiles.length > 0) {
      form.setFieldValue('main_filename', mainFiles[0].file.name)
    }

    // Update form field
    form.setFieldValue('local_folder_path', `${files.length} files selected`)
  }

  return (
    <Drawer
      title={t('providers.uploadLocalModel')}
      open={open}
      onClose={handleCancel}
      footer={[
        <Button key="cancel" onClick={handleCancel} disabled={uploading}>
          {t('buttons.cancel')}
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={handleSubmit}
          disabled={uploading}
        >
          {uploading ? t('providers.uploading') : t('buttons.upload')}
        </Button>,
      ]}
      width={600}
      maskClosable={!uploading}
      closable={!uploading}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          file_format: 'safetensors',
          local_folder_path: '',
          main_filename: '',
          settings: {},
        }}
      >
        <LocalModelCommonFields />

        <Form.Item
          name="local_folder_path"
          label={t('providers.localFolderPath')}
          rules={[
            {
              required: true,
              message: t('providers.selectModelFolderRequired'),
            },
          ]}
          valuePropName={'file'}
        >
          <Upload.Dragger
            directory
            multiple
            beforeUpload={() => false}
            onChange={handleFolderUpload}
            showUploadList={false}
          >
            <p className="ant-upload-drag-icon">
              <UploadOutlined />
            </p>
            <p className="ant-upload-text">
              {t('providers.dragOrSelectModelFolder')}
            </p>
            <p className="ant-upload-hint">
              {t('providers.selectModelFolderHint')}
            </p>
          </Upload.Dragger>
        </Form.Item>

        {filteredFiles.length > 0 && (
          <Card title={t('providers.selectedFiles')}>
            <List
              dataSource={filteredFiles}
              renderItem={item => (
                <List.Item
                  extra={
                    <div
                      style={{
                        display: 'flex',
                        gap: '8px',
                        alignItems: 'center',
                      }}
                    >
                      <Tag
                        color={item.required ? 'green' : 'blue'}
                        style={{ margin: 0 }}
                      >
                        {item.purpose}
                      </Tag>
                      <Text type="secondary" style={{ fontSize: '12px' }}>
                        {formatBytes(item.file.size)}
                      </Text>
                    </div>
                  }
                >
                  <Text
                    style={{
                      fontWeight: item.required ? 'bold' : 'normal',
                    }}
                  >
                    {item.file.name}
                  </Text>
                </List.Item>
              )}
            />
          </Card>
        )}

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
          <Select
            placeholder={t('providers.selectMainFile')}
            showSearch
            optionFilterProp="children"
            options={filteredFiles
              .filter(item => item.purpose === 'model')
              .map(item => ({
                value: item.file.name,
                label: item.file.name,
              }))}
          />
        </Form.Item>

        {uploading &&
          (uploadProgress.length > 0 || overallUploadProgress > 0) && (
            <Card title={t('providers.uploadProgress')}>
              {overallUploadProgress > 0 && (
                <div style={{ marginBottom: '12px' }}>
                  <Text strong>Overall Progress:</Text>
                  <Progress
                    percent={Math.round(overallUploadProgress)}
                    status="active"
                  />
                  <Text type="secondary" style={{ fontSize: '12px' }}>
                    {
                      uploadProgress.filter(f => f.status === 'completed')
                        .length
                    }{' '}
                    of {uploadProgress.length} files uploaded
                  </Text>
                </div>
              )}
              {uploadProgress.length > 0 && (
                <div>
                  {uploadProgress.map((fileProgress, index) => (
                    <div key={index} style={{ marginBottom: '8px' }}>
                      <Text strong>{fileProgress.filename}</Text>
                      <Progress
                        percent={Math.round(fileProgress.progress)}
                        status={
                          fileProgress.status === 'error'
                            ? 'exception'
                            : 'active'
                        }
                      />
                      {fileProgress.size && (
                        <Text type="secondary" style={{ fontSize: '12px' }}>
                          {formatBytes(
                            Math.round(
                              (fileProgress.progress * fileProgress.size) / 100,
                            ),
                          )}{' '}
                          of {formatBytes(fileProgress.size)} uploaded
                        </Text>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </Card>
          )}
      </Form>
    </Drawer>
  )
}
