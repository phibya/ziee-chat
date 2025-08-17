import { useEffect, useMemo, useRef, useState } from 'react'
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
  RobotOutlined,
  SendOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import {
  addNewConversationToList,
  createNewConversation,
  deleteFile,
  loadUserAssistants,
  loadUserProvidersWithAllModels,
  Stores,
  useChatStore,
} from '../../../store'
import { FileCard } from '../../Common/FileCard'
import { useChatInputUIStore } from '../../../store/ui/chatInput.ts'
import { Conversation, Message } from '../../../types/api/chat.ts'
import { Assistant } from '../../../types/api/assistant'
import { createChatStore } from '../../../store/chat.ts'
import { BsFileEarmarkPlus } from 'react-icons/bs'
import { IoIosArrowDown } from 'react-icons/io'
import { debounce } from '../../../utils/debounce.ts'

const { TextArea } = Input
const { Text } = Typography

const UI_BREAKPOINT = 480 // Define a breakpoint for UI adjustments

const calculateIsBreaking = (width: number): boolean => width <= UI_BREAKPOINT

export const ChatInput = function ChatInput({
  editingMessage,
  onDoneEditing,
  className = '',
  style,
}: {
  editingMessage?: Message
  onDoneEditing?: () => void
  className?: string
  style?: React.CSSProperties
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [form] = Form.useForm()
  const { conversationId } = useParams<{ conversationId?: string }>()
  const { projectId } = useParams<{ projectId?: string }>()
  const { message } = App.useApp()
  const { token } = theme.useToken()
  const [isBreaking, setIsBreaking] = useState<boolean>(false)
  const containerRef = useRef<HTMLDivElement>(null)

  const store = useChatInputUIStore(editingMessage)
  const {
    files,
    newFiles,
    uploadingFiles,
    isDisabled,
    uploadFiles,
    removeFile,
    removeUploadingFile,
    setContent,
    destroy: destroyStore,
  } = store
  const isEditing = !!editingMessage

  // File upload state
  const [isFocused, setIsFocused] = useState(false)
  const [isDragging, setIsDragging] = useState(false) // Drag state for overlay control

  const { conversation, sending, isStreaming, sendMessage, editMessage } =
    useChatStore()
  const { assistants } = Stores.Assistants
  const { providers, modelsByProvider } = Stores.Providers
  const { user } = Stores.Auth

  useEffect(() => {
    loadUserAssistants()
    loadUserProvidersWithAllModels()
  }, [])

  // ResizeObserver to listen to container width changes for UI breakpoints
  useEffect(() => {
    const containerElement = containerRef.current
    if (!containerElement) return

    const updateBreaking = (width: number) => {
      const newIsBreaking = calculateIsBreaking(width)
      setIsBreaking(newIsBreaking)
    }

    // Set initial breaking state immediately
    updateBreaking(containerElement.offsetWidth)

    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        const { width } = entry.contentRect
        updateBreaking(width)
      }
    })

    resizeObserver.observe(containerElement)

    return () => {
      resizeObserver.disconnect()
    }
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
    return Array.from(assistants.values()).filter(
      (a: Assistant) => !a.is_template,
    )
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
    if (conversation) {
      form.setFieldValue('assistant', conversation.assistant_id)
      // Find the provider for this model
      let matchingModel = null
      for (const providerGroup of availableModels) {
        matchingModel = providerGroup.options.find(model =>
          model.value.endsWith(`:${conversation.model_id}`),
        )
        if (matchingModel) break
      }
      if (matchingModel) {
        form.setFieldValue('model', matchingModel.value)
      }
    }
  }, [conversation, availableModels, form])

  const handleCreateNewConversation =
    async (): Promise<Conversation | null> => {
      const formValues = form.getFieldsValue()
      const { assistant: selectedAssistant, model: selectedModel } = formValues

      if (!selectedAssistant || !selectedModel) return null

      const [, modelId] = selectedModel.split(':')

      try {
        const newConversation = await createNewConversation(
          selectedAssistant,
          modelId,
          projectId,
        )

        // Add to conversations store immediately
        addNewConversationToList({
          id: newConversation.id,
          title: newConversation.title,
          user_id: user?.id || '',
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          last_message: undefined,
          message_count: 0,
        })

        navigate(`/conversation/${newConversation.id}`)
        destroyStore()

        return newConversation
      } catch (error) {
        console.error('Failed to create conversation:', error)
        return null
      }
    }

  const handleSend = async () => {
    if (isStreaming || sending || isDisabled) return
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
        await editMessage(editingMessage.id, {
          assistantId: selectedAssistant,
          modelId: selectedModel.split(':')[1],
          content: messageToSend,
          fileIds: [...files.keys(), ...newFiles.keys()],
        })
        onDoneEditing?.() // Close the input after editing
      } catch (error) {
        console.error('Failed to edit message:', error)
      }
      return
    }

    form.setFieldValue('message', '') // Clear input immediately

    const payload = {
      content: messageToSend.trim(),
      assistantId: selectedAssistant,
      modelId: selectedModel.split(':')[1],
      fileIds: [...files.keys(), ...newFiles.keys()],
    }

    let newFilesBackup = new Map(newFiles) // Backup newFiles before clearing

    //clear newFiles
    store.__setState({
      newFiles: new Map(),
    })

    try {
      if (conversationId) {
        // If we have a conversationId, use it
        await sendMessage(payload)
      } else {
        let newConversation = await handleCreateNewConversation()
        if (!newConversation) {
          message.error(t('chat.errorCreatingConversation'))
          return
        }

        const conversationStore = createChatStore(newConversation)

        await conversationStore.__state.sendMessage(payload)
      }
    } catch (error) {
      console.error('Failed to send message:', error)
      // Restore the message if sending failed
      form.setFieldValue('message', messageToSend)
      // Restore newFiles if sending failed
      store.__setState({
        newFiles: newFilesBackup,
      })
    }
  }

  const handleFileUpload = debounce(async (files: globalThis.File[]) => {
    return await uploadFiles(files)
  }, 100)

  const handleRemoveFile = (fileId: string) => {
    removeFile(fileId)
    removeUploadingFile(fileId)
  }

  const handleCancel = () => {
    //delete all new files
    newFiles.forEach((_, fileId) => {
      deleteFile(fileId)
    })
    destroyStore()
    onDoneEditing?.()
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

  return (
    <div
      ref={containerRef}
      className={`w-full relative ${className}`}
      style={style}
    >
      {/* Drag and Drop Overlay */}
      <Upload.Dragger
        multiple
        beforeUpload={(_, fileList) => {
          if (fileList) {
            handleFileUpload(fileList)?.catch?.(error => {
              console.error('Failed to upload files:', error)
            })
          }
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
          backgroundColor: token.colorBgLayout,
          borderRadius: token.borderRadius,
        }}
      >
        <Flex
          className="h-full flex-col items-center justify-center gap-3"
          style={{ pointerEvents: 'none' }}
        >
          <BsFileEarmarkPlus className={'text-2xl'} />
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
          backgroundColor: token.colorBgContainer,
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
            <Flex className="flex-col gap-1 w-full">
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
                    style={{
                      backgroundColor: 'transparent',
                    }}
                  />
                </Form.Item>
              </div>
              <div className={`w-full flex justify-between gap-0`}>
                <Upload
                  multiple
                  beforeUpload={(_, fileList) => {
                    if (fileList) {
                      handleFileUpload(fileList)?.catch?.(error => {
                        console.error('Failed to upload files:', error)
                      })
                    }
                    return false
                  }}
                  showUploadList={false}
                >
                  <Button
                    type="default"
                    disabled={isDisabled}
                    title="Add files"
                  >
                    <BsFileEarmarkPlus />
                  </Button>
                </Upload>

                <div className={'flex items-center gap-[6px]'}>
                  <Form.Item name="assistant" noStyle>
                    <Select
                      popupMatchSelectWidth={false}
                      placeholder="Assistant"
                      options={availableAssistants.map(
                        (assistant: Assistant) => ({
                          label: assistant.name,
                          value: assistant.id,
                        }),
                      )}
                      style={{
                        width: isBreaking ? 40 : 140,
                        paddingLeft: isBreaking ? 0 : 6,
                      }}
                      labelRender={isBreaking ? () => '' : undefined}
                      variant={isBreaking ? 'borderless' : undefined}
                      prefix={
                        isBreaking && (
                          <Button>
                            <RobotOutlined />
                          </Button>
                        )
                      }
                      suffixIcon={<IoIosArrowDown />}
                    />
                  </Form.Item>

                  {/* Model selector */}
                  <Form.Item name="model" noStyle>
                    <Select
                      popupMatchSelectWidth={false}
                      placeholder="Model"
                      disabled={isDisabled}
                      options={availableModels}
                      style={{ width: isBreaking ? 40 : 140 }}
                      variant={isBreaking ? 'borderless' : undefined}
                      labelRender={isBreaking ? () => '' : undefined}
                      prefix={
                        isBreaking && (
                          <Button>
                            <SettingOutlined />
                          </Button>
                        )
                      }
                      suffixIcon={<IoIosArrowDown />}
                    />
                  </Form.Item>
                </div>

                <div className={`gap-2 flex flex-1 items-center justify-end`}>
                  {/* Send/Stop/Save/Cancel buttons */}
                  <div className="flex gap-1 items-end">
                    <div className={'items-center justify-end  gap-1 flex'}>
                      {isEditing ? (
                        <>
                          <Button
                            type="primary"
                            icon={<SendOutlined rotate={270} />}
                            onClick={handleSend}
                            disabled={
                              isStreaming ||
                              isDisabled ||
                              !form.getFieldValue('message')?.trim()
                            }
                          />
                          <Button
                            icon={<CloseOutlined />}
                            onClick={handleCancel}
                            size="small"
                          />
                        </>
                      ) : (
                        <Button
                          type="primary"
                          icon={<SendOutlined rotate={270} />}
                          onClick={handleSend}
                          disabled={isStreaming || sending || isDisabled}
                          loading={sending}
                        />
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </Flex>
          </div>

          {/* Divider and File Upload Preview at the bottom */}
          {(files.size > 0 || newFiles.size > 0 || uploadingFiles.size > 0) && (
            <>
              <Divider style={{ margin: 0 }} />
              <div style={{ padding: '8px' }}>
                <div className="flex gap-2 w-full overflow-x-auto">
                  {Array.from(files.values()).map(file => (
                    <div className={'flex-1 min-w-20 max-w-24'}>
                      <FileCard
                        key={file.id}
                        file={file}
                        canDelete={false}
                        canRemove={true}
                        onRemove={handleRemoveFile}
                      />
                    </div>
                  ))}
                  {Array.from(newFiles.values()).map(file => (
                    <div className={'flex-1 min-w-20 max-w-24'}>
                      <FileCard
                        key={file.id}
                        file={file}
                        canDelete={true}
                        onDelete={handleRemoveFile}
                      />
                    </div>
                  ))}
                  {Array.from(uploadingFiles.values()).map(uploadingFile => (
                    <div className={'flex-1 min-w-20 max-w-24'}>
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
                    </div>
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
