import { useEffect } from 'react'
import { Card, Table, Tag, Typography, Flex, Alert } from 'antd'
import type { TableColumnsType } from 'antd'
import { initializeEngines, Stores } from '../../../store'
import type { EngineInfo } from '../../../types'

const { Title, Text } = Typography

interface EnginesTableRecord extends EngineInfo {
  key: string
}

export function AdminEngines() {
  const { engines, loading, error, initialized } = Stores.AdminEngines

  useEffect(() => {
    if (!initialized) {
      initializeEngines().catch(console.error)
    }
  }, [initialized])

  const tableData: EnginesTableRecord[] = engines.map(engine => ({
    ...engine,
    key: engine.engine_type,
  }))

  const columns: TableColumnsType<EnginesTableRecord> = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (name: string, record: EnginesTableRecord) => (
        <Flex vertical gap={4}>
          <Text strong>{name}</Text>
          <Text type="secondary" style={{ fontSize: '12px' }}>
            Type: {record.engine_type}
          </Text>
        </Flex>
      ),
    },
    {
      title: 'Version',
      dataIndex: 'version',
      key: 'version',
      width: 120,
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      width: 120,
      render: (status: string) => {
        const color =
          status === 'available'
            ? 'green'
            : status === 'unavailable'
              ? 'red'
              : 'orange'
        return <Tag color={color}>{status}</Tag>
      },
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
      render: (description: string | undefined) => (
        <Text type="secondary">
          {description || 'No description available'}
        </Text>
      ),
    },
    {
      title: 'Supported Architectures',
      dataIndex: 'supported_architectures',
      key: 'supported_architectures',
      render: (architectures: string[] | undefined) => (
        <Flex gap={4} wrap="wrap">
          {architectures?.map(arch => <Tag key={arch}>{arch}</Tag>) || (
            <Text type="secondary">N/A</Text>
          )}
        </Flex>
      ),
    },
  ]

  if (error) {
    return (
      <Alert
        message="Error Loading Engines"
        description={error}
        type="error"
        showIcon
        style={{ margin: '24px' }}
      />
    )
  }

  return (
    <div style={{ padding: '24px' }}>
      <Flex
        justify="space-between"
        align="center"
        style={{ marginBottom: '24px' }}
      >
        <div>
          <Title level={2} style={{ margin: 0 }}>
            Engine Management
          </Title>
          <Text type="secondary">Manage local model execution engines</Text>
        </div>
      </Flex>

      <Card>
        <Table
          columns={columns}
          dataSource={tableData}
          loading={loading}
          pagination={false}
          size="small"
          rowKey="key"
        />
      </Card>

      {engines.length === 0 && !loading && (
        <Alert
          message="No Engines Available"
          description="No local execution engines are currently available on this system."
          type="info"
          showIcon
          style={{ marginTop: '24px' }}
        />
      )}
    </div>
  )
}
