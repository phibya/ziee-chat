import { Button, Flex, Form, Input, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import { useState } from 'react'
import { updateExistingConversation, useChatStore } from '../../../store'
import { ChatInput } from './ChatInput'
import { ChatMessageList } from './ChatMessageList'
import { TitleBarWrapper } from '../../Common/TitleBarWrapper.tsx'
import { TauriDragRegion } from '../../Common/TauriDragRegion.tsx'
import tinycolor from 'tinycolor2'
import { CheckOutlined, CloseOutlined, EditOutlined } from '@ant-design/icons'
import { IoIosArrowBack } from 'react-icons/io'

export function ExistingChatInterface() {
  const { conversationId } = useParams<{ conversationId?: string }>()
  const { token } = theme.useToken()
  const [form] = Form.useForm()
  const [isEditing, setIsEditing] = useState(false)
  const navigate = useNavigate()

  const { t } = useTranslation()
  // Chat store
  const { conversation, loading } = useChatStore()

  const handleEditClick = () => {
    form.setFieldValue('title', conversation?.title || '')
    setIsEditing(true)
  }

  const handleSave = async () => {
    try {
      const values = await form.validateFields()
      if (conversation && values.title.trim()) {
        await updateExistingConversation(conversation.id, {
          title: values.title.trim(),
        })
        setIsEditing(false)
      }
    } catch (error) {
      console.error('Failed to update conversation title:', error)
    }
  }

  const handleCancel = () => {
    form.resetFields()
    setIsEditing(false)
  }

  if (!conversationId) {
    return null
  }

  if (loading) {
    return (
      <Flex className="flex-col items-center justify-center h-full">
        <div className="text-lg">{t('chat.loading')}</div>
      </Flex>
    )
  }

  if (!conversation) {
    return <div>Conversation not found</div>
  }

  return (
    <Flex className="flex flex-col w-full h-full overflow-hidden relative">
      <TitleBarWrapper
        className="z-2 backdrop-blur-3xl"
        style={{
          backgroundColor: tinycolor(token.colorBgLayout)
            .setAlpha(0.85)
            .toRgbString(),
        }}
      >
        <TauriDragRegion className={'h-full w-full absolute top-0 left-0'} />
        <div
          className="h-full flex items-center justify-between max-w-full overflow-hidden"
          style={{
            width: isEditing ? '100%' : undefined,
          }}
        >
          <div
            className={
              'flex items-center gap-1 flex-1 max-w-full justify-start'
            }
          >
            {isEditing ? (
              <Form
                form={form}
                className="flex items-center gap-1 flex-1 max-w-full"
              >
                <Form.Item
                  name="title"
                  className="!mb-0 flex-1"
                  rules={[
                    { required: true, message: 'Please enter a title' },
                    {
                      max: 100,
                      message: 'Title must be less than 100 characters',
                    },
                  ]}
                >
                  <Input
                    placeholder="Enter conversation title"
                    autoFocus
                    onPressEnter={handleSave}
                    size="small"
                    className="!border-none !shadow-none"
                    style={{
                      backgroundColor: 'transparent',
                      fontSize: '16px',
                      fontWeight: 600,
                    }}
                  />
                </Form.Item>
                <Button
                  type="text"
                  size="small"
                  icon={<CheckOutlined />}
                  onClick={handleSave}
                  className="!p-1"
                />
                <Button
                  type="text"
                  size="small"
                  icon={<CloseOutlined />}
                  onClick={handleCancel}
                  className="!p-1"
                />
              </Form>
            ) : (
              <>
                <Button
                  type={'text'}
                  className={'!px-1'}
                  onClick={() => navigate('/conversations')}
                >
                  <IoIosArrowBack className={'text-md'} />
                </Button>
                <Typography.Title
                  level={5}
                  ellipsis
                  className={'!m-0 !leading-tight flex-1 truncate'}
                >
                  {conversation?.title || 'Untitled Conversation'}
                </Typography.Title>
                <Button
                  type="text"
                  icon={<EditOutlined />}
                  onClick={handleEditClick}
                />
              </>
            )}
          </div>
        </div>
      </TitleBarWrapper>
      <div className="flex flex-col w-full h-full overflow-hidden z-1 absolute">
        <Flex className={'w-full flex-1 h-full overflow-auto'}>
          <div
            className={'self-center max-w-4xl w-full h-full m-auto px-4 pt-16'}
          >
            <ChatMessageList />
          </div>
        </Flex>
        <div
          className={'w-full pb-2 items-center justify-center content-center'}
        >
          <div className={'max-w-4xl w-full px-2 m-auto'}>
            <ChatInput />
          </div>
        </div>
      </div>
    </Flex>
  )
}
