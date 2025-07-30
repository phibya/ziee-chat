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
  Progress,
  App,
  Card,
  Skeleton,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import {
  SendOutlined,
  StopOutlined,
  UploadOutlined,
  CloseOutlined,
} from '@ant-design/icons'
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
import { formatFileSize } from '../../utils/fileUtils'
import type { File } from '../../types'

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

// File card component for chat input
const ChatFileCard: React.FC<{
  uploadedFile: UploadedFile
  onRemove: (fileId: string) => void
}> = ({ uploadedFile, onRemove }) => {
  const { token } = theme.useToken()

  if (uploadedFile.uploading) {
    // Show skeleton loading state
    return (
      <div style={{ width: '111px' }}>
        <Card
          size="small"
          style={{ height: '111px' }}
          styles={{
            body: {
              height: '100%',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
              position: 'relative',
              padding: '8px',
            },
          }}
        >
          <Skeleton.Image style={{ width: '100%', height: '60px' }} />
          <div className="mt-2">
            <Progress
              size="small"
              percent={100}
              status="active"
              showInfo={false}
            />
          </div>
        </Card>
        <div className="w-full text-center text-xs text-ellipsis overflow-hidden mt-1">
          <Text className="whitespace-nowrap">{uploadedFile.filename}</Text>
        </div>
      </div>
    )
  }

  if (uploadedFile.error) {
    // Show error state
    return (
      <div style={{ width: '111px' }}>
        <Card
          size="small"
          style={{
            height: '111px',
            border: `1px solid ${token.colorError}`,
          }}
          styles={{
            body: {
              height: '100%',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
              alignItems: 'center',
              position: 'relative',
            },
          }}
        >
          <Button
            danger
            size="small"
            icon={<CloseOutlined />}
            onClick={() => onRemove(uploadedFile.id)}
            className="!absolute top-1 right-1"
          />

          <Text className="text-4xl mb-2">❌</Text>
          <Text type="danger" style={{ fontSize: '10px', textAlign: 'center' }}>
            {uploadedFile.error}
          </Text>
        </Card>
        <div className="w-full text-center text-xs text-ellipsis overflow-hidden mt-1">
          <Text className="whitespace-nowrap" type="danger">
            {uploadedFile.filename}
          </Text>
        </div>
      </div>
    )
  }

  // Show completed file card (similar to FileCard but simplified for chat)
  return (
    <div style={{ width: '111px' }}>
      <Card
        size="small"
        className="group relative cursor-default"
        style={{ height: '111px' }}
        styles={{
          body: {
            height: '100%',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'center',
            position: 'relative',
          },
        }}
      >
        {/* Remove button - only visible on hover */}
        <Button
          danger
          size="small"
          icon={<CloseOutlined />}
          onClick={() => onRemove(uploadedFile.id)}
          className="!absolute top-1 right-1 opacity-0
                    group-hover:opacity-100 transition-opacity"
        />

        <Text
          className="absolute top-1 left-1 rounded px-1 !text-[9px]"
          style={{
            backgroundColor: token.colorBgContainer,
          }}
          strong
        >
          {uploadedFile.filename.split('.').pop()?.toUpperCase() || 'FILE'}
        </Text>

        <div className="flex flex-col items-center justify-center h-full">
          <Text className="text-2xl mb-1">📄</Text>
          <Text style={{ fontSize: '8px', textAlign: 'center' }}>
            Ready to send
          </Text>
        </div>

        <Text
          className="absolute bottom-1 right-1 rounded px-1 !text-[9px]"
          style={{
            backgroundColor: token.colorBgContainer,
          }}
        >
          {formatFileSize(uploadedFile.size)}
        </Text>
      </Card>
      <div className="w-full text-center text-xs text-ellipsis overflow-hidden mt-1">
        <Text className="whitespace-nowrap">{uploadedFile.filename}</Text>
      </div>
    </div>
  )
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
              {uploadedFiles.map(uploadedFile => (
                <ChatFileCard
                  key={uploadedFile.id}
                  uploadedFile={uploadedFile}
                  onRemove={handleRemoveFile}
                />
              ))}
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
