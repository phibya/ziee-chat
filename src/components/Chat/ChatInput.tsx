import { useEffect, useMemo, useState } from 'react'
import {
  App,
  Button,
  Card,
  Divider,
  Flex,
  Form,
  Input,
  Select,
  theme,
  Typography,
  Upload,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import {
  CloseOutlined,
  FileOutlined,
  SendOutlined,
  StopOutlined,
} from '@ant-design/icons'
import {
  addNewConversationToList,
  createNewConversation,
  editChatMessage,
  loadUserAssistants,
  loadUserProvidersWithAllModels,
  sendChatMessage,
  stopMessageStreaming,
  Stores,
} from '../../store'
import { FileCard } from '../common/FileCard'
import { createChatInputUIStore } from '../../store/ui/chatInput.ts'
import { Message } from '../../types/api/chat.ts'

const { TextArea } = Input
const { Text } = Typography

export const ChatInput = function ChatInput({
  editingMessage,
  onCancel,
}: {
  editingMessage?: Message
  onCancel?: () => void
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [form] = Form.useForm()
  const { conversationId } = useParams<{ conversationId?: string }>()
  const { projectId } = useParams<{ projectId?: string }>()
  const { message } = App.useApp()
  const { token } = theme.useToken()

  const store = useMemo(() => {
    const id = conversationId || projectId || 'new-chat'
    return createChatInputUIStore(id, editingMessage)
  }, [])
  const {
    files,
    newFiles,
    uploadingFiles,
    isDisabled,
    uploadFiles,
    removeFile,
    removeUploadingFile,
    setContent,
    destroy,
  } = store
  const isEditing = !!editingMessage

  // File upload state
  const [isFocused, setIsFocused] = useState(false)
  const [isDragging, setIsDragging] = useState(false) // Drag state for overlay control

  const { currentConversation, sending, isStreaming } = Stores.Chat
  const { assistants } = Stores.Assistants
  const { providers, modelsByProvider } = Stores.Providers
  const { user } = Stores.Auth

  useEffect(() => {
    loadUserAssistants()
    loadUserProvidersWithAllModels()
  }, [])

  // Initialize form and files when in editing mode
  useEffect(() => {
    form.setFieldValue(
      'message',
      editingMessage?.content || store.__state.content || '',
    )
  }, [])

  // Get available assistants (exclude templates) - memoized for performance
  const availableAssistants = useMemo(() => {
    return assistants.filter(a => !a.is_template)
  }, [assistants])

  // Get available models grouped by provider - memoized for performance
  const availableModels = useMemo(() => {
    const options: Array<{
      label: string
      options: Array<{
        label: string
        value: string
        description?: string
      }>
    }> = []

    providers.forEach(provider => {
      const providerModels = modelsByProvider[provider.id] || []

      if (providerModels.length > 0) {
        options.push({
          label: provider.name,
          options: providerModels.map(model => ({
            label: model.alias || model.id,
            value: `${provider.id}:${model.id}`,
            description: model.description || '',
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

    if (!messageToSend?.trim()) {
      message.info('Please enter a message')
      return
    }

    if (!selectedAssistant) {
      message.info('Please select an assistant')
      return
    }

    if (!selectedModel) {
      message.info('Please select a model')
      return
    }

    // Handle editing mode
    if (isEditing) {
      try {
        await editChatMessage(editingMessage.id, {
          assistantId: selectedAssistant,
          modelId: selectedModel.split(':')[1],
          content: messageToSend,
          fileIds: [...files.map(f => f.id), ...newFiles.map(f => f.id)],
        })
        destroy()
      } catch (error) {
        console.error('Failed to edit message:', error)
      }
      return
    }

    form.setFieldValue('message', '') // Clear input immediately

    try {
      let currentConversationId =
        conversationId || (await handleCreateNewConversation())
      if (!currentConversationId) {
        message.error(t('chat.errorCreatingConversation'))
        return
      }

      await sendChatMessage({
        conversationId: currentConversationId,
        content: messageToSend.trim(),
        assistantId: selectedAssistant,
        modelId: selectedModel.split(':')[1],
        fileIds: [...files.map(f => f.id), ...newFiles.map(f => f.id)],
      })

      if (!conversationId) {
        destroy()
      }
    } catch (error) {
      console.error('Failed to send message:', error)
      // Restore the message if sending failed
      form.setFieldValue('message', messageToSend)
    }
  }

  const handleFileUpload = async (files: globalThis.File[]) => {
    uploadFiles(files)
  }

  const handleRemoveFile = (fileId: string) => {
    removeFile(fileId)
    removeUploadingFile(fileId)
  }

  const handleCancel = () => {
    if (isEditing) {
      destroy()
    }
    onCancel?.()
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const handleTextAreaFocus = () => {
    setIsFocused(true)
  }

  const handleTextAreaBlur = () => {
    setIsFocused(false)
  }

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
          [&:has(.ant-upload-drag-hover)]:opacity-100
          [&:has(.ant-upload-drag-hover)]:!z-500
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
          className="h-full flex-col items-center justify-center gap-3"
          style={{ pointerEvents: 'none' }}
        >
          <FileOutlined className={'text-4xl'} />
          <Text type="secondary">Drop files here to upload</Text>
        </Flex>
      </Upload.Dragger>

      <Card
        onDragEnter={() => {
          setIsDragging(true)
        }}
        onDragLeave={() => {
          setIsDragging(false)
        }}
        classNames={{
          body: '!p-0',
        }}
        style={{
          borderColor: isFocused
            ? token.colorPrimaryBorder
            : token.colorBorderSecondary,
          transition: 'border-color 0.2s, box-shadow 0.2s',
          pointerEvents: isDragging ? 'none' : 'auto',
        }}
      >
        <Form
          form={form}
          layout="vertical"
          className="w-full"
          initialValues={{
            message: '',
            assistant: undefined,
            model: undefined,
          }}
          disabled={isDisabled}
        >
          {/* Main input row with add file button on left and selectors + send on right */}
          <div style={{ padding: '8px' }}>
            <Flex className="flex-col gap-2 w-full">
              <div className="w-full">
                <Form.Item name="message" className="mb-0" noStyle>
                  <TextArea
                    onKeyDown={handleKeyDown}
                    onFocus={handleTextAreaFocus}
                    onBlur={handleTextAreaBlur}
                    placeholder={t('chat.messageAI')}
                    autoSize={{ minRows: 1, maxRows: 6 }}
                    disabled={isDisabled}
                    className="resize-none !border-none focus:!border-none focus:!outline-none focus:!shadow-none"
                    onChange={() => {
                      setContent(form.getFieldValue('message') || '')
                    }}
                  />
                </Form.Item>
              </div>
              <div className="w-full flex gap-3 justify-between">
                <Upload
                  multiple
                  beforeUpload={(_, fileList) => {
                    handleFileUpload(fileList).catch(error => {
                      console.error('Failed to upload files:', error)
                    })
                    return false
                  }}
                  showUploadList={false}
                >
                  <Button
                    icon={<FileOutlined />}
                    disabled={isDisabled}
                    title="Add files"
                  />
                </Upload>

                <div className={'gap-2 flex items-center'}>
                  <Form.Item name="assistant" noStyle>
                    <Select
                      placeholder="Assistant"
                      style={{ width: 140 }}
                      disabled={isDisabled}
                      size="small"
                      options={availableAssistants.map(assistant => ({
                        label: assistant.name,
                        value: assistant.id,
                      }))}
                      className={`
                      [&_.ant-select-selector]:!border-none
                      [&_.ant-select-selection-wrap]:!text-center
                      `}
                    />
                  </Form.Item>

                  {/* Model selector */}
                  <Form.Item name="model" noStyle>
                    <Select
                      placeholder="Model"
                      style={{ width: 160 }}
                      disabled={isDisabled}
                      size="small"
                      options={availableModels}
                      className={`
                      [&_.ant-select-selector]:!border-none
                      [&_.ant-select-selection-wrap]:!text-center
                      `}
                    />
                  </Form.Item>

                  {/* Send/Stop/Save/Cancel buttons */}
                  <div className="flex gap-1">
                    {isEditing ? (
                      <>
                        <Button
                          type="primary"
                          icon={<SendOutlined rotate={270} />}
                          onClick={handleSend}
                          disabled={!form.getFieldValue('message')?.trim()}
                          size="small"
                        />
                        <Button
                          icon={<CloseOutlined />}
                          onClick={handleCancel}
                          size="small"
                        />
                      </>
                    ) : (
                      <>
                        {showStop && (
                          <Button
                            type="text"
                            icon={<StopOutlined />}
                            onClick={stopMessageStreaming}
                            size="small"
                          />
                        )}
                        <Button
                          type="primary"
                          icon={<SendOutlined rotate={270} />}
                          onClick={handleSend}
                          disabled={isDisabled}
                          loading={sending}
                          size="small"
                        />
                      </>
                    )}
                  </div>
                </div>
              </div>
            </Flex>
          </div>

          {/* Divider and File Upload Preview at the bottom */}
          {(files.length > 0 ||
            newFiles.length > 0 ||
            uploadingFiles.length > 0) && (
            <>
              <Divider style={{ margin: 0 }} />
              <div style={{ padding: '8px' }}>
                <div className="flex flex-wrap gap-2">
                  {files.map(file => (
                    <FileCard
                      key={file.id}
                      file={file}
                      canDelete={false}
                      canRemove={true}
                      onRemove={handleRemoveFile}
                    />
                  ))}
                  {newFiles.map(file => (
                    <FileCard
                      key={file.id}
                      file={file}
                      canDelete={true}
                      onDelete={handleRemoveFile}
                    />
                  ))}
                  {uploadingFiles.map(uploadingFile => (
                    <FileCard
                      key={uploadingFile.id}
                      uploadingFile={{
                        id: uploadingFile.id,
                        filename: uploadingFile.filename,
                        progress: uploadingFile.progress || 0,
                        status: 'uploading',
                        size: uploadingFile.size,
                      }}
                      onRemove={handleRemoveFile}
                    />
                  ))}
                </div>
              </div>
            </>
          )}
        </Form>
      </Card>
    </div>
  )
}
