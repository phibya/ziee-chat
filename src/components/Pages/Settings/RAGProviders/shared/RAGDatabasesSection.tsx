import {
  DeleteOutlined,
  DownloadOutlined,
  EditOutlined,
  PauseCircleOutlined,
  PlayCircleOutlined,
  PlusOutlined,
} from '@ant-design/icons'
import {
  App,
  Badge,
  Button,
  Card,
  Dropdown,
  Empty,
  Flex,
  Modal,
  Spin,
  Switch,
  Table,
  Typography,
} from 'antd'
import { useEffect } from 'react'
import { RAGProvider } from '../../../../../types'
import {
  deleteExistingRAGDatabase,
  disableRAGDatabase,
  enableRAGDatabase,
  loadDatabasesForRAGProvider,
  openAddRAGDatabaseDownloadDrawer,
  openAddRAGDatabaseDrawer,
  openEditRAGDatabaseDrawer,
  startRAGDatabase,
  stopRAGDatabase,
  Stores,
} from '../../../../../store'

const { Title } = Typography

interface RAGDatabasesSectionProps {
  provider: RAGProvider
}

export function RAGDatabasesSection({ provider }: RAGDatabasesSectionProps) {
  const { message } = App.useApp()
  const { databasesByProvider, loadingDatabases, databaseOperations } =
    Stores.AdminRAGProviders

  const databases = databasesByProvider[provider.id] || []
  const isLoading = loadingDatabases[provider.id]

  useEffect(() => {
    loadDatabasesForRAGProvider(provider.id)
  }, [provider.id])

  const handleStart = async (databaseId: string) => {
    try {
      await startRAGDatabase(databaseId)
      message.success('RAG database started')
    } catch (error) {
      console.error('Failed to start RAG database:', error)
    }
  }

  const handleStop = async (databaseId: string) => {
    try {
      await stopRAGDatabase(databaseId)
      message.success('RAG database stopped')
    } catch (error) {
      console.error('Failed to stop RAG database:', error)
    }
  }

  const handleEnable = async (databaseId: string, enabled: boolean) => {
    try {
      if (enabled) {
        await enableRAGDatabase(databaseId)
      } else {
        await disableRAGDatabase(databaseId)
      }
      message.success(`RAG database ${enabled ? 'enabled' : 'disabled'}`)
    } catch (error) {
      console.error('Failed to update RAG database:', error)
    }
  }

  const handleDelete = (databaseId: string, databaseName: string) => {
    Modal.confirm({
      title: 'Delete RAG Database',
      content: `Are you sure you want to delete "${databaseName}"? This action cannot be undone.`,
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        try {
          await deleteExistingRAGDatabase(databaseId)
          message.success('RAG database deleted')
        } catch (error) {
          console.error('Failed to delete RAG database:', error)
        }
      },
    })
  }

  const columns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: any) => (
        <div>
          <div>{text}</div>
          <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
            {record.alias}
          </Typography.Text>
        </div>
      ),
    },
    {
      title: 'Collection',
      dataIndex: 'collection_name',
      key: 'collection_name',
      render: (text: string) => text || '-',
    },
    {
      title: 'Embedding Model',
      dataIndex: 'embedding_model',
      key: 'embedding_model',
      render: (text: string) => text || '-',
    },
    {
      title: 'Chunk Size',
      dataIndex: 'chunk_size',
      key: 'chunk_size',
    },
    {
      title: 'Status',
      key: 'status',
      render: (_: any, record: any) => (
        <Flex gap="small">
          {provider.type === 'local' && (
            <Badge
              status={record.is_active ? 'success' : 'default'}
              text={record.is_active ? 'Active' : 'Inactive'}
            />
          )}
          <Badge
            status={record.enabled ? 'success' : 'error'}
            text={record.enabled ? 'Enabled' : 'Disabled'}
          />
        </Flex>
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: any) => {
        const menuItems = []

        if (provider.type === 'local') {
          if (record.is_active) {
            menuItems.push({
              key: 'stop',
              icon: <PauseCircleOutlined />,
              label: 'Stop',
              onClick: () => handleStop(record.id),
              disabled: databaseOperations[record.id],
            })
          } else {
            menuItems.push({
              key: 'start',
              icon: <PlayCircleOutlined />,
              label: 'Start',
              onClick: () => handleStart(record.id),
              disabled: databaseOperations[record.id],
            })
          }
        }

        menuItems.push(
          {
            key: 'edit',
            icon: <EditOutlined />,
            label: 'Edit',
            onClick: () => {
              openEditRAGDatabaseDrawer(record)
            },
          },
          {
            key: 'delete',
            icon: <DeleteOutlined />,
            label: 'Delete',
            onClick: () => handleDelete(record.id, record.name),
            danger: true,
          },
        )

        return (
          <Flex gap="small">
            <Switch
              checked={record.enabled}
              onChange={enabled => handleEnable(record.id, enabled)}
              loading={databaseOperations[record.id]}
            />
            <Dropdown menu={{ items: menuItems }} trigger={['click']}>
              <Button>Actions</Button>
            </Dropdown>
          </Flex>
        )
      },
    },
  ]

  return (
    <Card>
      <Flex justify="space-between" align="center" style={{ marginBottom: 16 }}>
        <Title level={4} style={{ margin: 0 }}>
          RAG Databases
        </Title>
        <Flex gap="small">
          <Button
            icon={<DownloadOutlined />}
            onClick={() => {
              openAddRAGDatabaseDownloadDrawer(provider?.id)
            }}
          >
            Download from Repository
          </Button>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => {
              openAddRAGDatabaseDrawer(provider?.id)
            }}
          >
            Add Database
          </Button>
        </Flex>
      </Flex>

      {isLoading ? (
        <div style={{ textAlign: 'center', padding: '50px' }}>
          <Spin size="large" />
        </div>
      ) : databases.length === 0 ? (
        <Empty description="No RAG databases found" />
      ) : (
        <Table
          columns={columns}
          dataSource={databases}
          rowKey="id"
          pagination={false}
        />
      )}
    </Card>
  )
}
