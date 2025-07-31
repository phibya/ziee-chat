import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  ApiOutlined,
} from '@ant-design/icons'
import {
  App,
  Button,
  Card,
  Dropdown,
  Empty,
  Flex,
  Modal,
  Switch,
  Table,
  Typography,
} from 'antd'
import { useEffect } from 'react'
import { isDesktopApp } from '../../../../api/core'
import { Permission, usePermissions } from '../../../../permissions'
import {
  Stores,
  loadAllRAGRepositories,
  deleteRAGRepository,
  testRAGRepositoryConnection,
  clearRAGRepositoriesError,
} from '../../../../store'
import { SettingsPageContainer } from '../SettingsPageContainer'
import { AddRAGRepositoryDrawer } from './AddRAGRepositoryDrawer'
import { EditRAGRepositoryDrawer } from './EditRAGRepositoryDrawer'

const { Title } = Typography

export function RAGRepositoriesSettings() {
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  // RAG repositories store
  const { repositories, loading, error } = Stores.AdminRAGRepositories

  // Check permissions
  const canEditRepositories =
    isDesktopApp || hasPermission(Permission.config.providers.edit)
  const canViewRepositories =
    isDesktopApp || hasPermission(Permission.config.providers.read)

  // If user doesn't have view permissions, don't render the component
  if (!canViewRepositories) {
    return (
      <div style={{ padding: '24px', textAlign: 'center' }}>
        <Title level={3}>Access Denied</Title>
        <Typography.Text type="secondary">
          You do not have permission to view RAG repository settings.
        </Typography.Text>
      </div>
    )
  }

  useEffect(() => {
    loadAllRAGRepositories()
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearRAGRepositoriesError()
    }
  }, [error, message])

  const handleDelete = (repositoryId: string, repositoryName: string) => {
    if (!canEditRepositories) {
      message.error('No permission to delete RAG repositories')
      return
    }

    Modal.confirm({
      title: 'Delete RAG Repository',
      content: `Are you sure you want to delete "${repositoryName}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await deleteRAGRepository(repositoryId)
          message.success('RAG repository deleted successfully')
        } catch (error: any) {
          console.error('Failed to delete RAG repository:', error)
        }
      },
    })
  }

  const handleTestConnection = async (repositoryId: string) => {
    try {
      await testRAGRepositoryConnection(repositoryId)
      message.success('Connection test successful')
    } catch (error: any) {
      console.error('Connection test failed:', error)
      message.error('Connection test failed')
    }
  }

  const columns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: any) => (
        <div>
          <div>{text}</div>
          {record.description && (
            <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
              {record.description}
            </Typography.Text>
          )}
        </div>
      ),
    },
    {
      title: 'URL',
      dataIndex: 'url',
      key: 'url',
      render: (text: string) => (
        <Typography.Text code style={{ fontSize: '12px' }}>
          {text}
        </Typography.Text>
      ),
    },
    {
      title: 'Priority',
      dataIndex: 'priority',
      key: 'priority',
      width: 100,
    },
    {
      title: 'Auth',
      key: 'auth',
      width: 80,
      render: (_: any, record: any) => (
        <Typography.Text type={record.requires_auth ? 'warning' : 'secondary'}>
          {record.requires_auth ? 'Required' : 'None'}
        </Typography.Text>
      ),
    },
    {
      title: 'Status',
      key: 'status',
      width: 100,
      render: (_: any, record: any) => (
        <Switch
          size="small"
          checked={record.enabled}
          disabled={!canEditRepositories}
          onChange={(enabled) => {
            // TODO: Update repository enabled status
            console.log('Update repository enabled:', record.id, enabled)
          }}
        />
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      width: 120,
      render: (_: any, record: any) => {
        const menuItems: any[] = [
          {
            key: 'test',
            icon: <ApiOutlined />,
            label: 'Test Connection',
            onClick: async () => handleTestConnection(record.id),
          },
        ]

        if (canEditRepositories) {
          menuItems.push(
            {
              key: 'edit',
              icon: <EditOutlined />,
              label: 'Edit',
              onClick: async () => {
                // TODO: Open edit drawer
                console.log('Edit repository:', record.id)
              },
            },
            {
              key: 'delete',
              icon: <DeleteOutlined />,
              label: 'Delete',
              onClick: async () => handleDelete(record.id, record.name),
              danger: true,
            }
          )
        }

        return (
          <Dropdown menu={{ items: menuItems }} trigger={['click']}>
            <Button size="small">Actions</Button>
          </Dropdown>
        )
      },
    },
  ]

  return (
    <SettingsPageContainer title="RAG Repositories">
      <Card>
        <Flex justify="space-between" align="center" style={{ marginBottom: 16 }}>
          <Title level={4} style={{ margin: 0 }}>
            RAG Repositories
          </Title>
          {canEditRepositories && (
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => {
                // TODO: Open add drawer
                console.log('Add repository')
              }}
            >
              Add Repository
            </Button>
          )}
        </Flex>

        <Typography.Paragraph type="secondary">
          Manage RAG repositories that provide pre-configured databases for download.
          Repositories are prioritized by their priority value (higher values first).
        </Typography.Paragraph>

        {repositories.length === 0 ? (
          <Empty
            description="No RAG repositories configured"
            image={Empty.PRESENTED_IMAGE_SIMPLE}
          />
        ) : (
          <Table
            columns={columns}
            dataSource={repositories}
            rowKey="id"
            loading={loading}
            pagination={{
              pageSize: 10,
              showSizeChanger: true,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total} repositories`,
            }}
          />
        )}
      </Card>

      {/* Modals */}
      <AddRAGRepositoryDrawer />
      <EditRAGRepositoryDrawer />
    </SettingsPageContainer>
  )
}