import { Button, Flex, Form, Input, Result, theme, Typography } from 'antd'
import { useNavigate, useParams } from 'react-router-dom'
import { useState } from 'react'
import {
  Stores,
  updateExistingConversation,
  useChatStore,
  toggleShowTime,
} from '../../../store'
import { ChatInput } from './ChatInput'
import { ChatMessageList } from './ChatMessageList'
import { TitleBarWrapper } from '../../common/TitleBarWrapper.tsx'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'
import tinycolor from 'tinycolor2'
import { CheckOutlined, CloseOutlined, EditOutlined } from '@ant-design/icons'
import { IoIosArrowBack } from 'react-icons/io'
import { IoTimeOutline } from 'react-icons/io5'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { Permission } from '../../../types'
import { PiSmileySadLight } from 'react-icons/pi'
import { DivScrollY } from '../../common/DivScrollY.tsx'

export function ExistingChatInterface() {
  const { conversationId } = useParams<{ conversationId?: string }>()
  const { token } = theme.useToken()
  const [form] = Form.useForm()
  const [isEditing, setIsEditing] = useState(false)
  const navigate = useNavigate()

  // Chat store
  const { conversation, loading } = useChatStore()
  const { showTime } = Stores.UI.ChatUI

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

  if (!loading && !conversation) {
    return (
      <div className={'w-full h-full flex items-center justify-center'}>
        <Result
          icon={
            <div className={'w-full flex items-center justify-center text-8xl'}>
              <PiSmileySadLight />
            </div>
          }
          title="Conversation Not Found"
          subTitle="The conversation you are looking for does not exist or has been deleted."
          extra={
            <Button
              type="primary"
              onClick={() =>
                navigate(
                  Stores.UI.PathHistory.__state
                    .previousConversationListPagePath,
                )
              }
            >
              Go Back
            </Button>
          }
        />
      </div>
    )
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
        <div className="h-full flex items-center justify-between w-full overflow-hidden">
          <div className={'flex items-center gap-1 w-full'}>
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
              <div className="flex items-center justify-between w-full overflow-hidden">
                <div className="flex gap-1 items-center justify-start overflow-hidden">
                  <Button
                    type={'text'}
                    className={'!px-1'}
                    onClick={() =>
                      navigate(
                        Stores.UI.PathHistory.__state
                          .previousConversationListPagePath,
                      )
                    }
                  >
                    <IoIosArrowBack className={'text-md'} />
                  </Button>
                  <Typography.Title
                    level={5}
                    ellipsis
                    className={'!m-0 !leading-tight truncate'}
                  >
                    {conversation?.title || 'Untitled Conversation'}
                  </Typography.Title>
                  <PermissionGuard permissions={[Permission.ChatEdit]}>
                    <Button
                      type="text"
                      icon={<EditOutlined />}
                      onClick={handleEditClick}
                    />
                  </PermissionGuard>
                </div>
                <div className={'flex-shrink-0'}>
                  <Button
                    type={!showTime ? 'text' : 'primary'}
                    className={'!px-1'}
                    onClick={toggleShowTime}
                  >
                    <IoTimeOutline className={'!text-lg'} />
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>
      </TitleBarWrapper>
      <div className="flex flex-col w-full h-full overflow-hidden z-1 absolute">
        <DivScrollY className={'flex w-full flex-1 h-full flex-col'}>
          <div className={'max-w-4xl w-full px-4 pt-16 self-center'}>
            <ChatMessageList />
          </div>
        </DivScrollY>
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
