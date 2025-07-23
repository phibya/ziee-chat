import React, { useEffect, useState } from 'react'
import {
  App,
  Button,
  Card,
  Col,
  Dropdown,
  Form,
  Input,
  MenuProps,
  Modal,
  Row,
  Select,
  Typography,
} from 'antd'
import {
  CalendarOutlined,
  DeleteOutlined,
  EditOutlined,
  FolderOutlined,
  MoreOutlined,
  PlusOutlined,
  SearchOutlined,
} from '@ant-design/icons'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Project } from '../../types/api/projects'
import { PageContainer } from '../common/PageContainer'
import {
  Stores,
  loadAllUserProjects,
  createNewProject,
  updateExistingProject,
  deleteExistingProject,
  clearProjectsStoreError,
} from '../../store'

const { Title, Text } = Typography
const { Search } = Input
const { TextArea } = Input

interface ProjectFormData {
  name: string
  description?: string
  is_private?: boolean
}

export const ProjectsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const navigate = useNavigate()

  // Projects store
  const { projects, loading, creating, updating, error } = Stores.Projects

  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'activity' | 'name' | 'created'>(
    'activity',
  )
  const [newProjectModalVisible, setNewProjectModalVisible] = useState(false)
  const [editProjectModalVisible, setEditProjectModalVisible] = useState(false)
  const [editingProject, setEditingProject] = useState<Project | null>(null)
  const [form] = Form.useForm<ProjectFormData>()

  useEffect(() => {
    loadAllUserProjects()
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProjectsStoreError()
    }
  }, [error, message])

  // Get filtered and sorted projects
  const getFilteredAndSortedProjects = () => {
    let filteredProjects = projects

    // Apply search filter
    if (searchQuery.trim()) {
      filteredProjects = projects.filter(
        project =>
          project.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          project.description
            ?.toLowerCase()
            .includes(searchQuery.toLowerCase()),
      )
    }

    // Sort projects based on sortBy selection
    let sortedProjects = [...filteredProjects]
    switch (sortBy) {
      case 'activity':
        sortedProjects.sort(
          (a, b) =>
            new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
        )
        break
      case 'name':
        sortedProjects.sort((a, b) => a.name.localeCompare(b.name))
        break
      case 'created':
        sortedProjects.sort(
          (a, b) =>
            new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
        )
        break
    }

    return sortedProjects
  }

  const handleCreateProject = async (values: ProjectFormData) => {
    try {
      await createNewProject({
        name: values.name,
        description: values.description || '',
      })
      setNewProjectModalVisible(false)
      form.resetFields()
      message.success(t('projects.projectCreated'))
    } catch (error) {
      // Error is handled by the store
      console.error('Failed to create project:', error)
    }
  }

  const handleEditProject = async (values: ProjectFormData) => {
    if (!editingProject) return

    try {
      await updateExistingProject(editingProject.id, {
        name: values.name,
        description: values.description,
      })
      setEditProjectModalVisible(false)
      setEditingProject(null)
      form.resetFields()
      message.success(t('projects.projectUpdated'))
    } catch (error) {
      // Error is handled by the store
      console.error('Failed to update project:', error)
    }
  }

  const handleDeleteProject = async (project: Project) => {
    try {
      await deleteExistingProject(project.id)
      message.success(t('projects.projectDeleted'))
    } catch (error) {
      // Error is handled by the store
      console.error('Failed to delete project:', error)
    }
  }

  const openEditModal = (project: Project) => {
    setEditingProject(project)
    form.setFieldsValue({
      name: project.name,
      description: project.description,
      is_private: project.is_private,
    })
    setEditProjectModalVisible(true)
  }

  const formatTimeAgo = (date: string) => {
    const now = new Date()
    const past = new Date(date)
    const diffMs = now.getTime() - past.getTime()
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))
    const diffMonths = Math.floor(diffDays / 30)

    if (diffDays === 0) return 'Today'
    if (diffDays === 1) return '1 day ago'
    if (diffDays < 7) return `${diffDays} days ago`
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`
    if (diffMonths === 1) return '1 month ago'
    if (diffMonths < 12) return `${diffMonths} months ago`
    return `${Math.floor(diffMonths / 12)} years ago`
  }

  const getProjectMenuItems = (project: Project): MenuProps['items'] => [
    {
      key: 'edit',
      icon: <EditOutlined />,
      label: t('buttons.edit'),
      onClick: () => openEditModal(project),
    },
    {
      key: 'delete',
      icon: <DeleteOutlined />,
      label: t('buttons.delete'),
      danger: true,
      onClick: () => {
        Modal.confirm({
          title: t('projects.deleteProject'),
          content: `Are you sure you want to delete "${project.name}"? This action cannot be undone.`,
          okText: 'Delete',
          okType: 'danger',
          onOk: () => handleDeleteProject(project),
        })
      },
    },
  ]

  return (
    <PageContainer>
      {/* Header */}
      <div className="flex justify-between items-center mb-6">
        <Title level={2} className="!mb-0">
          Projects
        </Title>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setNewProjectModalVisible(true)}
        >
          New project
        </Button>
      </div>

      {/* Search and Sort */}
      <div className="flex justify-between items-center mb-6">
        <Search
          placeholder={t('forms.searchProjects')}
          prefix={<SearchOutlined />}
          style={{ width: 400 }}
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          allowClear
        />
        <div className="flex items-center gap-2">
          <Text type="secondary">Sort by</Text>
          <Select
            value={sortBy}
            onChange={setSortBy}
            style={{ width: 120 }}
            options={[
              { label: t('labels.activity'), value: 'activity' },
              { label: t('labels.name'), value: 'name' },
              { label: t('labels.created'), value: 'created' },
            ]}
          />
        </div>
      </div>

      {/* Projects Grid */}
      <Row gutter={[16, 16]}>
        {getFilteredAndSortedProjects().map(project => (
          <Col xs={24} sm={12} lg={8} xl={6} key={project.id}>
            <Card
              hoverable
              className="h-full cursor-pointer"
              onClick={() => navigate(`/projects/${project.id}`)}
              actions={[
                <Dropdown
                  menu={{ items: getProjectMenuItems(project) }}
                  trigger={['click']}
                >
                  <Button
                    type="text"
                    icon={<MoreOutlined />}
                    onClick={e => e.stopPropagation()}
                  />
                </Dropdown>,
              ]}
            >
              <div className="flex flex-col h-full">
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <FolderOutlined />
                    <Text strong className="text-base">
                      {project.name}
                    </Text>
                  </div>
                  {project.is_private && (
                    <Text type="secondary" className="text-xs">
                      Private
                    </Text>
                  )}
                </div>

                {project.description && (
                  <Text type="secondary" className="text-sm mb-4 line-clamp-2">
                    {project.description}
                  </Text>
                )}

                <div className="mt-auto">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1">
                      <CalendarOutlined />
                      <span>Updated {formatTimeAgo(project.updated_at)}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-4 mt-2">
                    <span>{project.document_count || 0} documents</span>
                    <span>{project.conversation_count || 0} conversations</span>
                  </div>
                </div>
              </div>
            </Card>
          </Col>
        ))}
      </Row>

      {/* Empty State */}
      {!loading && getFilteredAndSortedProjects().length === 0 && (
        <div className="text-center py-12">
          <FolderOutlined className="text-6xl mb-4" />
          <Title level={3} type="secondary">
            {searchQuery ? 'No projects found' : 'No projects yet'}
          </Title>
          <Text type="secondary" className="block mb-4">
            {searchQuery
              ? 'Try adjusting your search criteria'
              : 'Create your first project to get started'}
          </Text>
          {!searchQuery && (
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => setNewProjectModalVisible(true)}
            >
              Create project
            </Button>
          )}
        </div>
      )}

      {/* New Project Modal */}
      <Modal
        title={t('projects.createPersonalProject')}
        open={newProjectModalVisible}
        onCancel={() => {
          setNewProjectModalVisible(false)
          form.resetFields()
        }}
        footer={null}
        width={600}
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={handleCreateProject}
          initialValues={{ is_private: true }}
        >
          <Form.Item
            label={t('projects.whatAreYouWorkingOn')}
            name="name"
            rules={[{ required: true, message: 'Please enter a project name' }]}
          >
            <Input placeholder={t('forms.nameYourProject')} size="large" />
          </Form.Item>

          <Form.Item
            label={t('projects.whatAreYouTryingToAchieve')}
            name="description"
          >
            <TextArea
              placeholder={t('forms.describeYourProject')}
              rows={4}
              size="large"
            />
          </Form.Item>

          <div className="flex justify-end gap-2 mt-6">
            <Button
              onClick={() => {
                setNewProjectModalVisible(false)
                form.resetFields()
              }}
            >
              Cancel
            </Button>
            <Button type="primary" htmlType="submit" loading={creating}>
              Create project
            </Button>
          </div>
        </Form>
      </Modal>

      {/* Edit Project Modal */}
      <Modal
        title={t('projects.editProject')}
        open={editProjectModalVisible}
        onCancel={() => {
          setEditProjectModalVisible(false)
          setEditingProject(null)
          form.resetFields()
        }}
        footer={null}
        width={600}
      >
        <Form form={form} layout="vertical" onFinish={handleEditProject}>
          <Form.Item
            label={t('projects.projectName')}
            name="name"
            rules={[{ required: true, message: 'Please enter a project name' }]}
          >
            <Input placeholder={t('forms.nameYourProject')} size="large" />
          </Form.Item>

          <Form.Item label={t('labels.description')} name="description">
            <TextArea
              placeholder={t('forms.describeYourProject')}
              rows={4}
              size="large"
            />
          </Form.Item>

          <div className="flex justify-end gap-2 mt-6">
            <Button
              onClick={() => {
                setEditProjectModalVisible(false)
                setEditingProject(null)
                form.resetFields()
              }}
            >
              Cancel
            </Button>
            <Button type="primary" htmlType="submit" loading={updating}>
              Update project
            </Button>
          </div>
        </Form>
      </Modal>
    </PageContainer>
  )
}
