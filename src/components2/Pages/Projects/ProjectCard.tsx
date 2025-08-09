import {
  CalendarOutlined,
  DeleteOutlined,
  EditOutlined,
} from '@ant-design/icons'
import { App, Button, Card, Dropdown, Flex, Typography } from 'antd'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import type { Project } from '../../../types/api/projects'
import { deleteExistingProject, openProjectDrawer } from '../../../store'
import { CgMenuRightAlt } from 'react-icons/cg'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

const { Text } = Typography

interface ProjectCardProps {
  project: Project
}

export function ProjectCard({ project }: ProjectCardProps) {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { message, modal } = App.useApp()

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
        body: 'h-full flex flex-col !p-3 !pb-1',
      }}
      onClick={handleCardClick}
    >
      <Flex className="h-full flex-col flex-1">
        {/* Header with name and actions */}
        <Typography.Text strong className="m-0 pr-2">
          {project.name}
        </Typography.Text>

        {/* Description */}
        {project.description && (
          <div className="mb-3">
            <Text type="secondary" className="text-sm line-clamp-2">
              {project.description}
            </Text>
          </div>
        )}

        {/* Stats and date - pushed to bottom */}
        <div
          style={{
            marginTop: project.description ? 'auto' : '12px',
          }}
        >
          {/* Last updated */}
          <div className="mb-2">
            <Flex align="center" gap="small">
              <CalendarOutlined className="text-gray-400" />
              <Text type="secondary" className="text-xs">
                Updated {dayjs(project.updated_at).fromNow()}
              </Text>
            </Flex>
          </div>
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
              icon={<CgMenuRightAlt />}
              onClick={e => e.stopPropagation()}
              size="small"
            />
          </Dropdown>
        </div>
      </Flex>
    </Card>
  )
}
