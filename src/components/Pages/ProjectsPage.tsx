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
import { ApiClient } from '../../api/client'
import {
  CreateProjectRequest,
  Project,
  UpdateProjectRequest,
} from '../../types/api/projects'
import { PageContainer } from '../common/PageContainer'

const { Title, Text } = Typography
const { Search } = Input
const { TextArea } = Input

interface ProjectFormData {
  name: string
  description?: string
  is_private?: boolean
}

export const ProjectsPage: React.FC = () => {
  const { message } = App.useApp()
  const navigate = useNavigate()
  const [projects, setProjects] = useState<Project[]>([])
  const [loading, setLoading] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'activity' | 'name' | 'created'>(
    'activity',
  )
  const [newProjectModalVisible, setNewProjectModalVisible] = useState(false)
  const [editProjectModalVisible, setEditProjectModalVisible] = useState(false)
  const [editingProject, setEditingProject] = useState<Project | null>(null)
  const [form] = Form.useForm<ProjectFormData>()

  useEffect(() => {
    fetchProjects()
  }, [searchQuery, sortBy])

  const fetchProjects = async () => {
    try {
      setLoading(true)
      const response = await ApiClient.Projects.list({
        page: 1,
        per_page: 100,
        search: searchQuery || undefined,
      })

      // Sort projects based on sortBy selection
      let sortedProjects = [...response.projects]
      switch (sortBy) {
        case 'activity':
          sortedProjects.sort(
            (a, b) =>
              new Date(b.updated_at).getTime() -
              new Date(a.updated_at).getTime(),
          )
          break
        case 'name':
          sortedProjects.sort((a, b) => a.name.localeCompare(b.name))
          break
        case 'created':
          sortedProjects.sort(
            (a, b) =>
              new Date(b.created_at).getTime() -
              new Date(a.created_at).getTime(),
          )
          break
      }

      setProjects(sortedProjects)
    } catch (error) {
      message.error('Failed to fetch projects')
    } finally {
      setLoading(false)
    }
  }

  const handleCreateProject = async (values: ProjectFormData) => {
    try {
      const request: CreateProjectRequest = {
        name: values.name,
        description: values.description,
        is_private: values.is_private ?? true,
      }

      const newProject = await ApiClient.Projects.create(request)
      setProjects([newProject, ...projects])
      setNewProjectModalVisible(false)
      form.resetFields()
      message.success('Project created successfully')
    } catch (error) {
      message.error('Failed to create project')
    }
  }

  const handleEditProject = async (values: ProjectFormData) => {
    if (!editingProject) return

    try {
      const request: UpdateProjectRequest = {
        name: values.name,
        description: values.description,
        is_private: values.is_private,
      }

      const updatedProject = await ApiClient.Projects.update({
        project_id: editingProject.id,
        ...request,
      })

      setProjects(
        projects.map(p => (p.id === editingProject.id ? updatedProject : p)),
      )
      setEditProjectModalVisible(false)
      setEditingProject(null)
      form.resetFields()
      message.success('Project updated successfully')
    } catch (error) {
      message.error('Failed to update project')
    }
  }

  const handleDeleteProject = async (project: Project) => {
    try {
      await ApiClient.Projects.delete({ project_id: project.id })
      setProjects(projects.filter(p => p.id !== project.id))
      message.success('Project deleted successfully')
    } catch (error) {
      message.error('Failed to delete project')
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
      label: 'Edit',
      onClick: () => openEditModal(project),
    },
    {
      key: 'delete',
      icon: <DeleteOutlined />,
      label: 'Delete',
      danger: true,
      onClick: () => {
        Modal.confirm({
          title: 'Delete Project',
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
          placeholder="Search projects..."
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
              { label: 'Activity', value: 'activity' },
              { label: 'Name', value: 'name' },
              { label: 'Created', value: 'created' },
            ]}
          />
        </div>
      </div>

      {/* Projects Grid */}
      <Row gutter={[16, 16]}>
        {projects.map(project => (
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
      {!loading && projects.length === 0 && (
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
        title="Create a personal project"
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
            label="What are you working on?"
            name="name"
            rules={[{ required: true, message: 'Please enter a project name' }]}
          >
            <Input placeholder="Name your project" size="large" />
          </Form.Item>

          <Form.Item label="What are you trying to achieve?" name="description">
            <TextArea
              placeholder="Describe your project, goals, subject, etc..."
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
            <Button type="primary" htmlType="submit">
              Create project
            </Button>
          </div>
        </Form>
      </Modal>

      {/* Edit Project Modal */}
      <Modal
        title="Edit Project"
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
            label="Project Name"
            name="name"
            rules={[{ required: true, message: 'Please enter a project name' }]}
          >
            <Input placeholder="Name your project" size="large" />
          </Form.Item>

          <Form.Item label="Description" name="description">
            <TextArea
              placeholder="Describe your project, goals, subject, etc..."
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
            <Button type="primary" htmlType="submit">
              Update project
            </Button>
          </div>
        </Form>
      </Modal>
    </PageContainer>
  )
}
