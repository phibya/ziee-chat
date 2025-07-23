import React, { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  App,
  Button,
  Card,
  Popconfirm,
  Space,
  Table,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import { useShallow } from 'zustand/react/shallow'
import { Assistant } from '../../../../types/api/assistant'
import { PageContainer } from '../../../common/PageContainer'
import { useAdminStore, loadSystemAdminAssistants, deleteSystemAdminAssistant, clearSystemAdminError } from '../../../../store'
import { openAssistantModal } from '../../../../store/ui/modals'
import { AssistantFormModal } from '../../../shared/AssistantFormModal'

const { Title, Text } = Typography

export const AdminAssistantsSettings: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Admin store
  const {
    assistants,
    loading,
    error,
  } = useAdminStore(
    useShallow(state => ({
      assistants: state.assistants,
      loading: state.loading,
      creating: state.creating,
      updating: state.updating,
      deleting: state.deleting,
      error: state.error,
    })),
  )

  useEffect(() => {
    loadSystemAdminAssistants()
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
    openAssistantModal(assistant)
  }

  const handleCreate = () => {
    openAssistantModal()
  }

  const columns = [
    {
      title: t('labels.name'),
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: Assistant) => (
        <Space>
          <RobotOutlined />
          <Text strong>{text}</Text>
          {record.is_default && <Tag color="green">Default</Tag>}
          {!record.is_active && <Tag color="red">Inactive</Tag>}
        </Space>
      ),
    },
    {
      title: t('labels.description'),
      dataIndex: 'description',
      key: 'description',
      render: (text: string) => (
        <Text type="secondary">{text || 'No description'}</Text>
      ),
    },
    {
      title: t('admin.assistants.createdBy'),
      dataIndex: 'created_by',
      key: 'created_by',
      render: (userId: string) => (
        <Text type="secondary">{userId ? 'User' : 'System'}</Text>
      ),
    },
    {
      title: t('labels.created'),
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: t('labels.actions'),
      key: 'actions',
      render: (_: any, record: Assistant) => (
        <Space>
          <Tooltip title={t('buttons.edit')}>
            <Button
              type="text"
              icon={<EditOutlined />}
              onClick={() => handleEdit(record)}
            />
          </Tooltip>
          <Popconfirm
            title={t('assistants.deleteAssistant')}
            description={t('assistants.deleteConfirm')}
            onConfirm={() => handleDelete(record)}
            okText="Yes"
            cancelText="No"
          >
            <Tooltip title={t('buttons.delete')}>
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Tooltip>
          </Popconfirm>
        </Space>
      ),
    },
  ]

  return (
    <PageContainer>
      <div>
        <div className="flex justify-between items-center mb-6">
          <div>
            <Title level={3}>Assistants</Title>
            <Text type="secondary">
              Manage template assistants. Default assistants are automatically
              cloned for new users.
            </Text>
          </div>
          <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
            Create Assistant
          </Button>
        </div>

        <Card>
          <Table
            columns={columns}
            dataSource={assistants}
            loading={loading}
            rowKey="id"
            pagination={{ pageSize: 10 }}
          />
        </Card>

        <AssistantFormModal />
      </div>
    </PageContainer>
  )
}
