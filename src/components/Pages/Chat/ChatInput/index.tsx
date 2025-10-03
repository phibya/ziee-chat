import { useEffect, useMemo, useRef, useState } from 'react'
import { App, Button, Card, Flex, Form, Input, theme, Upload } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import { BsFileEarmarkPlus } from 'react-icons/bs'
import { CloseOutlined, SendOutlined } from '@ant-design/icons'
import {
  addNewConversationToList,
  createNewConversation,
  deleteFile,
  Stores,
  useChatStore,
} from '../../../../store'
import { useChatInputUIStore } from '../../../../store/ui/chatInput'
import { Assistant, Conversation, Message, Permission } from '../../../../types'
import { createChatStore, getMessageText } from '../../../../store/chat'
import { debounce } from '../../../../utils/debounce'
import { PermissionGuard } from '../../../Auth/PermissionGuard'
import { FileUploadArea } from './FileUploadArea'
import { FilePreviewList } from './FilePreviewList'
import { ToolSelector } from './ToolSelector'
import { Selectors } from './Selectors.tsx'

const { TextArea } = Input

const UI_BREAKPOINT = 480

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

  const [isFocused, setIsFocused] = useState(false)
  const [isDragging, setIsDragging] = useState(false)

  const { conversation, sending, isStreaming, sendMessage, editMessage } =
    useChatStore()
  const { assistants } = Stores.Assistants
  const { providers, modelsByProvider } = Stores.Providers
  const { user } = Stores.Auth

  useEffect(() => {
    const containerElement = containerRef.current
    if (!containerElement) return

    const updateBreaking = (width: number) => {
      setIsBreaking(calculateIsBreaking(width))
    }

    updateBreaking(containerElement.offsetWidth)

    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        updateBreaking(entry.contentRect.width)
      }
    })

    resizeObserver.observe(containerElement)

    return () => resizeObserver.disconnect()
  }, [])

  useEffect(() => {
    form.setFieldValue(
      'message',
      (editingMessage ? getMessageText(editingMessage) : '') ||
        store.__state.content ||
        '',
    )
    if (editingMessage) {
      store.__setState({
        files: new Map(editingMessage.files.map(file => [file.id, file])),
        newFiles: new Map(),
        uploadingFiles: new Map(),
      })
    }
  }, [])

  const availableAssistants = useMemo(() => {
    return Array.from(assistants.values()).filter(
      (a: Assistant) => !a.is_template,
    )
  }, [assistants])

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
      const providerModels = (modelsByProvider[provider.id] || []).filter(
        model => model.capabilities?.chat === true,
      )

      if (providerModels.length > 0) {
        options.push({
          label: provider.name,
          options: providerModels.map(model => ({
            label: model.display_name || model.name,
            value: `${provider.id}:${model.id}`,
            description: model.description || '',
          })),
        })
      }
    })

    return options
  }, [providers, modelsByProvider])

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

  useEffect(() => {
    if (conversation) {
      form.setFieldValue('assistant', conversation.assistant_id)
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
      enabled_tools: enabledTools,
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

    if (isEditing) {
      try {
        await editMessage(editingMessage.id, {
          assistant_id: selectedAssistant,
          model_id: selectedModel.split(':')[1],
          content: messageToSend,
          file_ids: [...files.keys(), ...newFiles.keys()],
          enabled_tools:
            enabledTools && enabledTools.length > 0 ? enabledTools : undefined,
        })
        onDoneEditing?.()
      } catch (error) {
        console.error('Failed to edit message:', error)
      }
      return
    }

    form.setFieldValue('message', '')

    const payload = {
      content: messageToSend.trim(),
      assistant_id: selectedAssistant,
      model_id: selectedModel.split(':')[1],
      file_ids: [...files.keys(), ...newFiles.keys()],
      enabled_tools:
        enabledTools && enabledTools.length > 0 ? enabledTools : undefined,
    }

    let newFilesBackup = new Map(newFiles)

    store.__setState({
      newFiles: new Map(),
    })

    try {
      if (conversationId) {
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
      form.setFieldValue('message', messageToSend)
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

  return (
    <div
      ref={containerRef}
      className={`w-full relative ${className}`}
      style={style}
    >
      <FileUploadArea onFileUpload={handleFileUpload} />

      <Card
        onDragEnter={() => setIsDragging(true)}
        onDragLeave={() => setIsDragging(false)}
        classNames={{ body: '!p-0' }}
        style={{
          borderColor: isFocused
            ? token.colorPrimaryBorder
            : token.colorBorderSecondary,
          transition: 'border-color 0.2s, box-shadow 0.2s',
          pointerEvents: isDragging ? 'none' : 'auto',
          backgroundColor: token.colorBgContainer,
        }}
      >
        <PermissionGuard
          permissions={[Permission.ChatCreate]}
          type={'disabled'}
        >
          <Form
            form={form}
            layout="vertical"
            className="w-full"
            initialValues={{
              message: '',
              assistant: undefined,
              model: undefined,
              enabled_tools: [],
              enabled_rag_ids: [],
            }}
            disabled={isDisabled}
          >
            <div style={{ padding: '8px' }}>
              <Flex className="flex-col gap-3 w-full">
                <div className="w-full">
                  <Form.Item name="message" className="mb-0" noStyle>
                    <TextArea
                      onKeyDown={handleKeyDown}
                      onFocus={() => setIsFocused(true)}
                      onBlur={() => setIsFocused(false)}
                      placeholder={t('chat.messageAI')}
                      autoSize={{ minRows: 1, maxRows: 6 }}
                      disabled={isDisabled}
                      className="resize-none !border-none focus:!border-none focus:!outline-none focus:!shadow-none !pt-1"
                      onChange={() =>
                        setContent(form.getFieldValue('message') || '')
                      }
                      style={{ backgroundColor: 'transparent' }}
                    />
                  </Form.Item>
                </div>
                <div className="w-full flex justify-between gap-0">
                  <div className="flex gap-1">
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

                    <ToolSelector isDisabled={isDisabled} />
                  </div>

                  <div className={'flex items-center gap-[6px]'}>
                    <Selectors
                      isBreaking={isBreaking}
                      isDisabled={isDisabled}
                      availableAssistants={availableAssistants}
                      availableModels={availableModels}
                    />

                    <div className={'items-center justify-end gap-1 flex'}>
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
              </Flex>
            </div>

            <FilePreviewList
              files={files}
              newFiles={newFiles}
              uploadingFiles={uploadingFiles}
              onRemoveFile={handleRemoveFile}
            />

            <Form.Item name="enabled_tools" noStyle />
            <Form.Item name="enabled_rag_ids" noStyle />
          </Form>
        </PermissionGuard>
      </Card>
    </div>
  )
}
