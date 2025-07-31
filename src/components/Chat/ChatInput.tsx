import { useEffect, useMemo, useState } from 'react'
import {
  Button,
  Flex,
  Input,
  Select,
  Form,
  Upload,
  theme,
  Typography,
  App,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import { SendOutlined, StopOutlined, UploadOutlined } from '@ant-design/icons'
import {
  Stores,
  sendChatMessage,
  stopMessageStreaming,
  createNewConversation,
  loadConversationById,
  addNewConversationToList,
  loadUserProvidersWithAllModels,
  loadUserAssistants,
} from '../../store'
import { ApiClient } from '../../api/client'
import type { File, FileUploadProgress } from '../../types'
import { FileCard } from '../common/FileCard'

const { TextArea } = Input
const { Text } = Typography

interface UploadedFile {
  id: string
  filename: string
  size: number
  uploading: boolean
  error?: string
  file?: File // The complete file object once uploaded
}

interface ChatInputProps {
  projectId?: string
}

export const ChatInput = function ChatInput({ projectId }: ChatInputProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [form] = Form.useForm()
  const { conversationId } = useParams<{ conversationId?: string }>()
  const { message } = App.useApp()
  const { token } = theme.useToken()

  // File upload state
  const [uploadedFiles, setUploadedFiles] = useState<UploadedFile[]>([])

  const { currentConversation, sending, isStreaming } = Stores.Chat
  const { assistants } = Stores.Assistants
  const { providers, modelsByProvider } = Stores.Providers
  const { user } = Stores.Auth

  useEffect(() => {
    console.log('here')
    loadUserAssistants()
    loadUserProvidersWithAllModels()
  }, [])

  // Get available assistants (exclude templates) - memoized for performance
  const availableAssistants = useMemo(() => {
    return assistants.filter(a => !a.is_template)
  }, [assistants])

  // Get available models grouped by provider - memoized for performance
  const availableModels = useMemo(() => {
    const options: Array<{
      label: string
      options: Array<{ label: string; value: string }>
    }> = []

    providers.forEach(provider => {
      const providerModels = modelsByProvider[provider.id] || []

      if (providerModels.length > 0) {
        options.push({
          label: provider.name,
          options: providerModels.map(model => ({
            label: model.alias || model.id,
            value: `${provider.id}:${model.id}`,
          })),
        })
      }
    })

    return options
  }, [providers, modelsByProvider])

  // Initialize default selections
  useEffect(() => {
    const currentValues = form.getFieldsValue()
    if (!currentValues.assistant && availableAssistants.length > 0) {
      form.setFieldValue('assistant', availableAssistants[0].id)
    }
  }, [availableAssistants, form])

  useEffect(() => {
    const currentValues = form.getFieldsValue()
    if (
      !currentValues.model &&
      availableModels.length > 0 &&
      availableModels[0].options.length > 0
    ) {
      form.setFieldValue('model', availableModels[0].options[0].value)
    }
  }, [availableModels, form])

  // For existing conversations, sync selections with conversation data
  useEffect(() => {
    if (currentConversation) {
      form.setFieldValue('assistant', currentConversation.assistant_id)
      // Find the provider for this model
      let matchingModel = null
      for (const providerGroup of availableModels) {
        matchingModel = providerGroup.options.find(model =>
          model.value.endsWith(`:${currentConversation.model_id}`),
        )
        if (matchingModel) break
      }
      if (matchingModel) {
        form.setFieldValue('model', matchingModel.value)
      }
    }
  }, [currentConversation, availableModels, form])

  const handleCreateNewConversation = async (): Promise<string | null> => {
    const formValues = form.getFieldsValue()
    const { assistant: selectedAssistant, model: selectedModel } = formValues

    if (!selectedAssistant || !selectedModel) return null

    const [, modelId] = selectedModel.split(':')

    try {
      const newConversationId = await createNewConversation(
        selectedAssistant,
        modelId,
        projectId,
      )

      // Add to conversations store immediately
      addNewConversationToList({
        id: newConversationId,
        title: 'New Conversation',
        user_id: user?.id || '',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        last_message: undefined,
        message_count: 0,
      })

      navigate(`/conversation/${newConversationId}`)

      return newConversationId
    } catch (error) {
      console.error('Failed to create conversation:', error)
      return null
    }
  }

  const handleSend = async () => {
    const formValues = form.getFieldsValue()
    const {
      message: messageToSend,
      assistant: selectedAssistant,
      model: selectedModel,
    } = formValues

    if (!messageToSend?.trim() || !selectedAssistant || !selectedModel) return

    form.setFieldValue('message', '') // Clear input immediately

    // Get uploaded file IDs (only non-uploading files)
    const fileIds = uploadedFiles
      .filter(f => !f.uploading && !f.error)
      .map(f => f.id)

    try {
      if (conversationId) {
        // Existing conversation: send message directly
        await sendChatMessage({
          conversationId,
          content: messageToSend.trim(),
          assistantId: selectedAssistant,
          modelId: selectedModel.split(':')[1],
          fileIds: fileIds.length > 0 ? fileIds : undefined,
        })
      } else {
        // New conversation: create conversation then send message
        const newConversationId = await handleCreateNewConversation()
        if (newConversationId) {
          await sendChatMessage({
            conversationId: newConversationId,
            content: messageToSend.trim(),
            assistantId: selectedAssistant,
            modelId: selectedModel.split(':')[1],
            fileIds: fileIds.length > 0 ? fileIds : undefined,
          })
          await loadConversationById(newConversationId, false)
        }
      }

      // Clear uploaded files after successful send
      setUploadedFiles([])
    } catch (error) {
      console.error('Failed to send message:', error)
      // Restore the message if sending failed
      form.setFieldValue('message', messageToSend)
    }
  }

  const handleFileUpload = async (files: globalThis.File[]) => {
    const newFiles: UploadedFile[] = files.map(file => ({
      id: crypto.randomUUID(),
      filename: file.name,
      size: file.size,
      uploading: true,
    }))

    setUploadedFiles(prev => [...prev, ...newFiles])

    // Upload files one by one
    for (const file of files) {
      const fileInfo = newFiles.find(f => f.filename === file.name)
      if (!fileInfo) continue

      try {
        const formData = new FormData()
        formData.append('file', file)

        const response = await ApiClient.Files.upload(formData)

        // Update file status to completed
        setUploadedFiles(prev =>
          prev.map(f =>
            f.id === fileInfo.id
              ? {
                  ...f,
                  id: response.file.id,
                  uploading: false,
                  file: response.file,
                }
              : f,
          ),
        )

        message.success(`${file.name} uploaded successfully`)
      } catch (error) {
        console.error('Failed to upload file:', error)

        // Update file status to error
        setUploadedFiles(prev =>
          prev.map(f =>
            f.id === fileInfo.id
              ? { ...f, uploading: false, error: 'Upload failed' }
              : f,
          ),
        )

        message.error(`Failed to upload ${file.name}`)
      }
    }
  }

  const handleRemoveFile = (fileId: string) => {
    setUploadedFiles(prev => prev.filter(f => f.id !== fileId))
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const isDisabled = sending
  const showStop = sending || isStreaming

  return (
    <div className="w-full relative">
      {/* Drag and Drop Overlay */}
      <Upload.Dragger
        multiple
        beforeUpload={(_, fileList) => {
          handleFileUpload(fileList).catch(error => {
            console.error('Failed to upload files:', error)
          })
          return false
        }}
        showUploadList={false}
        className={`
          opacity-0
          [&_.ant-upload-drag]:!cursor-default
          [&_.ant-upload-drag]:!border-none
          [&_.ant-upload-drag-hover]:!border-dashed
          [&:has(.ant-upload-drag-hover)]:z-50
          [&:has(.ant-upload-drag-hover)]:opacity-100
          absolute inset-0
          transition-opacity duration-300 ease-in-out
        `}
        openFileDialogOnClick={false}
        style={{
          backgroundColor: token.colorBgContainer,
          borderRadius: token.borderRadius,
        }}
      >
        <Flex
          className="h-full flex-col items-center justify-center gap-2"
          style={{ pointerEvents: 'none' }}
        >
          <UploadOutlined style={{ fontSize: '24px' }} />
          <Text type="secondary">Drop files here to upload</Text>
        </Flex>
      </Upload.Dragger>

      <Form
        form={form}
        layout="vertical"
        className="w-full"
        initialValues={{
          message: '',
          assistant: undefined,
          model: undefined,
        }}
      >
        <Flex vertical className="w-full gap-2">
          <Flex gap="small">
            <Form.Item name="assistant" className="flex-1 mb-0">
              <Select
                placeholder="Select assistant"
                style={{ width: 200 }}
                disabled={isDisabled}
                options={availableAssistants.map(assistant => ({
                  label: assistant.name,
                  value: assistant.id,
                }))}
              />
            </Form.Item>
            <Form.Item name="model" className="flex-1 mb-0">
              <Select
                placeholder="Select model"
                style={{ width: 250 }}
                disabled={isDisabled}
                options={availableModels}
              />
            </Form.Item>
          </Flex>

          {/* File Upload Preview */}
          {uploadedFiles.length > 0 && (
            <div className="flex flex-wrap gap-2 p-3 rounded">
              {uploadedFiles.map(uploadedFile => {
                if (uploadedFile.uploading) {
                  // Show uploading state using FileCard
                  const uploadProgress: FileUploadProgress = {
                    id: uploadedFile.id,
                    filename: uploadedFile.filename,
                    progress: 75, // Show indeterminate progress
                    status: 'uploading',
                    size: uploadedFile.size,
                  }
                  return (
                    <FileCard
                      key={uploadedFile.id}
                      uploadingFile={uploadProgress}
                      onRemove={handleRemoveFile}
                      removeId={uploadedFile.id}
                    />
                  )
                } else if (uploadedFile.error) {
                  // Show error state using FileCard
                  const uploadProgress: FileUploadProgress = {
                    id: uploadedFile.id,
                    filename: uploadedFile.filename,
                    progress: 0,
                    status: 'error',
                    error: uploadedFile.error,
                    size: uploadedFile.size,
                  }
                  return (
                    <FileCard
                      key={uploadedFile.id}
                      uploadingFile={uploadProgress}
                      onRemove={handleRemoveFile}
                      removeId={uploadedFile.id}
                    />
                  )
                } else {
                  // Show completed file using FileCard
                  return (
                    <FileCard
                      key={uploadedFile.id}
                      file={uploadedFile.file}
                      onRemove={handleRemoveFile}
                      removeId={uploadedFile.id}
                    />
                  )
                }
              })}
            </div>
          )}

          <Flex className="flex items-end gap-1 w-full">
            <div className="flex-1">
              <Form.Item name="message" className="mb-0">
                <TextArea
                  onKeyPress={handleKeyPress}
                  placeholder={t('chat.messageAI')}
                  autoSize={{ minRows: 1, maxRows: 6 }}
                  disabled={isDisabled}
                  className="resize-none"
                />
              </Form.Item>
            </div>
            <div className="flex gap-2">
              {showStop && (
                <Button
                  type="text"
                  icon={<StopOutlined />}
                  onClick={stopMessageStreaming}
                >
                  {t('chat.stop')}
                </Button>
              )}
              <Button
                type="primary"
                icon={<SendOutlined />}
                onClick={handleSend}
                disabled={
                  !form.getFieldValue('message')?.trim() ||
                  isDisabled ||
                  !form.getFieldValue('assistant') ||
                  !form.getFieldValue('model') ||
                  uploadedFiles.some(f => f.uploading)
                }
                loading={sending}
              >
                {t('chat.send')}
              </Button>
            </div>
          </Flex>
        </Flex>
      </Form>
    </div>
  )
}
