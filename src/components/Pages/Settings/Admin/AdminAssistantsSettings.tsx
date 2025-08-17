import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Descriptions,
  Divider,
  Empty,
  Flex,
  Pagination,
  Popconfirm,
  Spin,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import React, { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  clearSystemAdminError,
  deleteSystemAdminAssistant,
  loadSystemAdminAssistants,
  openAssistantDrawer,
  Stores,
} from '../../../../store'
import { Assistant } from '../../../../types/api/assistant'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'
import { AssistantFormDrawer } from '../../../Common/AssistantFormDrawer'

const { Text } = Typography

export const AdminAssistantsSettings: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Admin assistants store
  const { 
    assistants, 
    total: totalAssistants,
    currentPage: storePage,
    pageSize: storePageSize,
    loading, 
    error 
  } = Stores.AdminAssistants


  useEffect(() => {
    loadSystemAdminAssistants(1, 10)
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearSystemAdminError()
    }
  }, [error, message])

  const handleDelete = async (assistant: Assistant) => {
    try {
      await deleteSystemAdminAssistant(assistant.id)
      message.success('Assistant deleted successfully')
    } catch (error) {
      console.error('Failed to delete assistant:', error)
      // Error is handled by the store
    }
  }

  const handleEdit = (assistant: Assistant) => {
    openAssistantDrawer(assistant, true)
  }

  const handleCreate = () => {
    openAssistantDrawer(undefined, true)
  }

  const getAssistantActions = (assistant: Assistant) => {
    const actions: React.ReactNode[] = []

    actions.push(
      <Tooltip key="edit" title={t('buttons.edit')}>
        <Button
          type="text"
          icon={<EditOutlined />}
          onClick={() => handleEdit(assistant)}
        />
      </Tooltip>,
    )

    actions.push(
      <Popconfirm
        key="delete"
        title={t('assistants.deleteAssistant')}
        description={t('assistants.deleteConfirm')}
        onConfirm={() => handleDelete(assistant)}
        okText="Yes"
        cancelText="No"
      >
        <Tooltip title={t('buttons.delete')}>
          <Button type="text" danger icon={<DeleteOutlined />} />
        </Tooltip>
      </Popconfirm>,
    )

    return actions.filter(Boolean)
  }

  const handlePageChange = (page: number, size?: number) => {
    const newPageSize = size || storePageSize
    const newPage = size && size !== storePageSize ? 1 : page // Reset to page 1 if page size changes

    loadSystemAdminAssistants(newPage, newPageSize)
  }

  return (
    <SettingsPageContainer
      title="Assistants"
      subtitle="Manage template assistants. Default assistants are automatically cloned for new users."
    >
      <div>
        <Card
          title="Template Assistants"
          extra={
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleCreate}
            >
              Create Assistant
            </Button>
          }
        >
          {loading ? (
            <div className="flex justify-center py-8">
              <Spin size="large" />
            </div>
          ) : assistants.length === 0 ? (
            <div>
              <Empty description="No assistants found" />
            </div>
          ) : (
            <div>
              {assistants.map((assistant, index) => (
                <div key={assistant.id}>
                  <div className="flex items-start gap-3 flex-wrap">
                    {/* Assistant Info */}
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2 flex-wrap">
                        <div className={'flex-1 min-w-48'}>
                          <Flex className="gap-2 items-center">
                            <RobotOutlined />
                            <Text className="font-medium">
                              {assistant.name}
                            </Text>
                            {assistant.is_default && (
                              <Tag color="green">Default</Tag>
                            )}
                            {!assistant.is_active && (
                              <Tag color="red">Inactive</Tag>
                            )}
                          </Flex>
                        </div>
                        <div className={'flex gap-1 items-center justify-end'}>
                          {getAssistantActions(assistant)}
                        </div>
                      </div>

                      <Descriptions
                        size="small"
                        column={{ xs: 1, sm: 2, md: 3 }}
                        colon={false}
                        labelStyle={{ fontSize: '12px', color: '#8c8c8c' }}
                        contentStyle={{ fontSize: '12px' }}
                      >
                        <Descriptions.Item label="Description">
                          {assistant.description || 'No description'}
                        </Descriptions.Item>
                        <Descriptions.Item label="Created By">
                          {assistant.created_by ? 'User' : 'System'}
                        </Descriptions.Item>
                        <Descriptions.Item label="Created">
                          {new Date(assistant.created_at).toLocaleDateString()}
                        </Descriptions.Item>
                      </Descriptions>
                    </div>
                  </div>
                  {index < assistants.length - 1 && (
                    <Divider className="my-0" />
                  )}
                </div>
              ))}
            </div>
          )}

          {assistants.length > 0 && (
            <>
              <Divider className="mb-4" />
              <div className="flex justify-end">
                <Pagination
                  current={storePage}
                  total={totalAssistants}
                  pageSize={storePageSize}
                  showSizeChanger
                  showQuickJumper
                  showTotal={(total, range) =>
                    `${range[0]}-${range[1]} of ${total} assistants`
                  }
                  onChange={handlePageChange}
                  onShowSizeChange={handlePageChange}
                  pageSizeOptions={['5', '10', '20', '50']}
                />
              </div>
            </>
          )}
        </Card>

        <AssistantFormDrawer />
      </div>
    </SettingsPageContainer>
  )
}
