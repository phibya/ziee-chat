import {
  CopyOutlined,
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  RobotOutlined,
} from '@ant-design/icons'
import {
  App,
  Avatar,
  Button,
  Card,
  Col,
  Modal,
  Popconfirm,
  Row,
  Space,
  Table,
  Tag,
  Tooltip,
  Typography,
} from 'antd'
import React, { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useShallow } from 'zustand/react/shallow'
import { useAssistantsStore } from '../../store/assistants'
import { Assistant } from '../../types/api/assistant'
import { PageContainer } from '../common/PageContainer'
import { AssistantFormModal } from '../shared/AssistantFormModal'

const { Title, Text } = Typography

export const AssistantsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Assistants store
  const {
    assistants: allAssistants,
    adminAssistants: templateAssistants,
    loading,
    creating,
    updating,
    deleting,
    error,
    loadAssistants,
    createAssistant,
    updateAssistant,
    deleteAssistant,
    clearError,
  } = useAssistantsStore(
    useShallow(state => ({
      assistants: state.assistants,
      adminAssistants: state.adminAssistants,
      loading: state.loading,
      creating: state.creating,
      updating: state.updating,
      deleting: state.deleting,
      error: state.error,
      loadAssistants: state.loadAssistants,
      createAssistant: state.createAssistant,
      updateAssistant: state.updateAssistant,
      deleteAssistant: state.deleteAssistant,
      clearError: state.clearError,
    })),
  )

  const assistants = allAssistants.filter(a => !a.is_template)

  const [modalVisible, setModalVisible] = useState(false)
  const [templateModalVisible, setTemplateModalVisible] = useState(false)
  const [editingAssistant, setEditingAssistant] = useState<Assistant | null>(
    null,
  )
  const [cloneSource, setCloneSource] = useState<Assistant | null>(null)

  useEffect(() => {
    loadAssistants()
  }, [loadAssistants])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearError()
    }
  }, [error, message, clearError])

  const handleFormSubmit = async (values: any) => {
    try {
      const requestData = {
        name: values.name,
        description: values.description || '',
        instructions: values.instructions || '',
        parameters: values.parameters ? JSON.parse(values.parameters) : {},
        is_enabled: values.is_active ?? true,
      }

      if (editingAssistant) {
        await updateAssistant(editingAssistant.id, requestData)
        message.success(t('assistants.assistantUpdated'))
      } else if (cloneSource) {
        await createAssistant(requestData)
        message.success(t('assistants.assistantCloned'))
      } else {
        await createAssistant(requestData)
        message.success(t('assistants.assistantCreated'))
      }

      handleModalClose()
    } catch (error) {
      // Error is already handled by the store
      console.error('Failed to save assistant:', error)
    }
  }

  const handleModalClose = () => {
    setModalVisible(false)
    setEditingAssistant(null)
    setCloneSource(null)
  }

  const handleDelete = async (assistant: Assistant) => {
    try {
      await deleteAssistant(assistant.id)
      message.success(t('assistants.assistantDeleted'))
    } catch (error) {
      // Error is already handled by the store
      console.error('Failed to delete assistant:', error)
    }
  }

  const handleEdit = (assistant: Assistant) => {
    setEditingAssistant(assistant)
    setCloneSource(null)
    setModalVisible(true)
  }

  const handleCreate = () => {
    setEditingAssistant(null)
    setCloneSource(null)
    setModalVisible(true)
  }

  const handleCloneFromTemplate = () => {
    setTemplateModalVisible(true)
  }

  const handleSelectTemplateAssistant = (assistant: Assistant) => {
    setCloneSource(assistant)
    setEditingAssistant(null)
    setTemplateModalVisible(false)
    setModalVisible(true)
  }

  const renderAssistantCard = (assistant: Assistant) => (
    <Card
      key={assistant.id}
      hoverable
      actions={[
        <Tooltip title={t('buttons.edit')} key="edit">
          <EditOutlined onClick={() => handleEdit(assistant)} />
        </Tooltip>,
        <Popconfirm
          title={t('assistants.deleteAssistant')}
          description={t('assistants.deleteConfirm')}
          onConfirm={() => handleDelete(assistant)}
          okText="Yes"
          cancelText="No"
          key="delete"
          okButtonProps={{ loading: deleting }}
        >
          <Tooltip title={t('buttons.delete')}>
            <DeleteOutlined />
          </Tooltip>
        </Popconfirm>,
      ]}
    >
      <Card.Meta
        avatar={
          <Avatar
            size={48}
            icon={<RobotOutlined />}
            style={{ backgroundColor: '#1890ff' }}
          />
        }
        title={
          <div className="flex items-center gap-2">
            <Text strong>{assistant.name}</Text>
            <Tag color="green">{t('assistants.personal')}</Tag>
            {!assistant.is_active && (
              <Tag color="red">{t('assistants.inactive')}</Tag>
            )}
          </div>
        }
        description={
          <div>
            <Text type="secondary" className="block mb-2">
              {assistant.description || 'No description'}
            </Text>
            <Text type="secondary" className="text-xs">
              Created {new Date(assistant.created_at).toLocaleDateString()}
            </Text>
          </div>
        }
      />
    </Card>
  )

  return (
    <PageContainer>
      <Row gutter={[24, 24]}>
        <Col span={24}>
          <div className="flex justify-between items-center mb-6">
            <div>
              <Title level={2}>{t('assistants.title')}</Title>
              <Text type="secondary">{t('assistants.subtitle')}</Text>
            </div>
            <Space>
              <Button
                type="default"
                icon={<CopyOutlined />}
                onClick={handleCloneFromTemplate}
              >
                Clone from Template
              </Button>
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={handleCreate}
              >
                Create New
              </Button>
            </Space>
          </div>

          {loading ? (
            <div className="flex justify-center items-center py-12">
              <div>Loading assistants...</div>
            </div>
          ) : assistants.length === 0 ? (
            <Card>
              <div className="text-center py-12">
                <RobotOutlined className="text-4xl mb-4" />
                <Title level={4} type="secondary">
                  No assistants yet
                </Title>
                <Text type="secondary">
                  Create your first assistant to get started
                </Text>
              </div>
            </Card>
          ) : (
            <Row gutter={[16, 16]}>
              {assistants.map(assistant => (
                <Col xs={24} sm={12} md={8} lg={6} key={assistant.id}>
                  {renderAssistantCard(assistant)}
                </Col>
              ))}
            </Row>
          )}
        </Col>
      </Row>

      <AssistantFormModal
        visible={modalVisible}
        editingAssistant={editingAssistant}
        cloneSource={cloneSource}
        loading={creating || updating}
        onSubmit={handleFormSubmit}
        onCancel={handleModalClose}
      />

      {/* Template Assistants Modal */}
      <Modal
        title={t('assistants.cloneFromTemplateAssistants')}
        open={templateModalVisible}
        onCancel={() => setTemplateModalVisible(false)}
        footer={null}
        width={900}
        maskClosable={false}
      >
        <div className="mb-4">
          <Text type="secondary">
            Select a template assistant to clone and customize for your use
          </Text>
        </div>
        <Table
          columns={[
            {
              title: t('labels.name'),
              dataIndex: 'name',
              key: 'name',
              render: (text: string) => (
                <Space>
                  <RobotOutlined />
                  <Text strong>{text}</Text>
                  <Tag color="blue">Template</Tag>
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
              title: t('assistants.instructionsPreview'),
              dataIndex: 'instructions',
              key: 'instructions',
              render: (text: string) => (
                <Text type="secondary" ellipsis={{ tooltip: text }}>
                  {text
                    ? text.substring(0, 100) + (text.length > 100 ? '...' : '')
                    : 'No instructions'}
                </Text>
              ),
              width: 200,
            },
            {
              title: t('labels.actions'),
              key: 'actions',
              render: (_: any, record: Assistant) => (
                <Space>
                  <Tooltip title={t('assistants.previewDetails')}>
                    <Button
                      type="text"
                      icon={<RobotOutlined />}
                      onClick={() => {
                        Modal.info({
                          title: `Preview: ${record.name}`,
                          content: (
                            <div>
                              <div className="mb-3">
                                <Text strong>{t('labels.description')}</Text>
                                <div>
                                  {record.description || 'No description'}
                                </div>
                              </div>
                              <div className="mb-3">
                                <Text strong>{t('labels.instructions')}</Text>
                                <div style={{ whiteSpace: 'pre-wrap' }}>
                                  {record.instructions || 'No instructions'}
                                </div>
                              </div>
                              <div className="mb-3">
                                <Text strong>{t('labels.parameters')}</Text>
                                <pre
                                  style={{
                                    backgroundColor: '#f5f5f5',
                                    padding: '8px',
                                    borderRadius: '4px',
                                  }}
                                >
                                  {record.parameters
                                    ? JSON.stringify(record.parameters, null, 2)
                                    : 'No parameters'}
                                </pre>
                              </div>
                            </div>
                          ),
                          width: 600,
                        })
                      }}
                    />
                  </Tooltip>
                  <Button
                    type="primary"
                    icon={<CopyOutlined />}
                    onClick={() => handleSelectTemplateAssistant(record)}
                  >
                    Clone
                  </Button>
                </Space>
              ),
            },
          ]}
          dataSource={templateAssistants}
          rowKey="id"
          pagination={{ pageSize: 5 }}
        />
      </Modal>
    </PageContainer>
  )
}
