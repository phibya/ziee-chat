import {
  CalendarOutlined,
  DeleteOutlined,
  EditOutlined,
  MoreOutlined,
} from '@ant-design/icons'
import { App, Button, Card, Dropdown, Flex, Typography } from 'antd'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import type { Project } from '../../../types/api/projects'
import { deleteExistingProject, openProjectDrawer } from '../../../store'

const { Title, Text } = Typography

interface ProjectCardProps {
  project: Project
}

export function ProjectCard({ project }: ProjectCardProps) {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { message, modal } = App.useApp()

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

  const handleCardClick = () => {
    navigate(`/projects/${project.id}`)
  }

  const handleDeleteProject = async (project: Project) => {
    try {
      await deleteExistingProject(project.id)
      message.success(t('projects.projectDeleted'))
    } catch (error) {
      console.error('Failed to delete project:', error)
    }
  }

  return (
    <Card
      hoverable
      className="h-full"
      classNames={{
        body: 'h-full flex flex-col',
      }}
      styles={{ body: { padding: '16px' } }}
      onClick={handleCardClick}
    >
      <Flex className="h-full flex-col flex-1">
        {/* Header with name and actions */}
        <Title level={4} className="m-0">
          {project.name}
        </Title>

        {/* Description */}
        {project.description && (
          <div className="mb-3">
            <Text type="secondary" className="text-sm line-clamp-2">
              {project.description}
            </Text>
          </div>
        )}

        {/* Stats and date - pushed to bottom */}
        <div className="mt-auto">
          {/* Last updated */}
          <div className="mb-2">
            <Flex align="center" gap="small">
              <CalendarOutlined className="text-gray-400" />
              <Text type="secondary" className="text-xs">
                Updated {formatTimeAgo(project.updated_at)}
              </Text>
            </Flex>
          </div>

          {/* Document and conversation counts */}
          <Flex justify="space-between" className="text-xs">
            <Text type="secondary">
              {project.document_count || 0} documents
            </Text>
            <Text type="secondary">
              {project.conversation_count || 0} conversations
            </Text>
          </Flex>
        </div>

        <div className="absolute top-2 right-2">
          <Dropdown
            menu={{
              items: [
                {
                  key: 'edit',
                  icon: <EditOutlined />,
                  label: 'Edit',
                  onClick: e => {
                    e.domEvent.stopPropagation()
                    e.domEvent.preventDefault()
                    openProjectDrawer(project)
                  },
                },
                {
                  key: 'delete',
                  icon: <DeleteOutlined />,
                  label: 'Delete',
                  danger: true,
                  onClick: e => {
                    e.domEvent.stopPropagation()
                    e.domEvent.preventDefault()
                    modal.confirm({
                      title: 'Delete Assistant',
                      content: `Are you sure?`,
                      okText: 'Delete',
                      okType: 'danger',
                      onOk: () => handleDeleteProject(project),
                    })
                  },
                },
              ],
            }}
            trigger={['click']}
          >
            <Button
              type="text"
              icon={<MoreOutlined />}
              onClick={e => e.stopPropagation()}
              size="small"
            />
          </Dropdown>
        </div>
      </Flex>
    </Card>
  )
}
