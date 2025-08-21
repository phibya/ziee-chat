import React, { useEffect, useState } from 'react'
import { App, Button, Dropdown, Input, Typography } from 'antd'
import { FolderOutlined, PlusOutlined, SearchOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import {
  clearProjectsStoreError,
  loadAllUserProjects,
  openProjectDrawer,
  Stores,
} from '../../../store'
import { ProjectFormDrawer } from './ProjectFormDrawer.tsx'
import { ProjectCard } from './ProjectCard.tsx'
import { TitleBarWrapper } from '../../common/TitleBarWrapper.tsx'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'
import { PiSortAscending } from 'react-icons/pi'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { Permission } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'

const { Title, Text } = Typography

export const ProjectsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const pageMinSize = useMainContentMinSize()
  const [isSearchBoxVisible, setIsSearchBoxVisible] = useState(false)

  // Projects store
  const { projects, loading, error } = Stores.Projects

  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'activity' | 'name' | 'created'>(
    'activity',
  )

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

  const searchInputComponent = (
    <Input
      placeholder={t('forms.searchProjects')}
      prefix={<SearchOutlined />}
      className={'w-full items-center justify-center flex-1 pr-1'}
      value={searchQuery}
      onChange={e => setSearchQuery(e.target.value)}
      allowClear
    />
  )

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Page Header */}
      <TitleBarWrapper>
        <div className="h-full flex items-center justify-between w-full ">
          <TauriDragRegion className={'h-full w-full absolute top-0 left-0'} />
          <Typography.Title level={4} className="!m-0 !leading-tight">
            Projects
          </Typography.Title>
          <div className="h-full flex items-center justify-between">
            {!pageMinSize.xs ? (
              <div className={'pr-1'}>{searchInputComponent}</div>
            ) : (
              <Button
                type={isSearchBoxVisible ? 'primary' : 'text'}
                icon={<SearchOutlined />}
                style={{
                  fontSize: '18px',
                }}
                onClick={() => setIsSearchBoxVisible(!isSearchBoxVisible)}
              />
            )}
            <div className={'flex gap-0'}>
              <Dropdown
                menu={{
                  items: [
                    {
                      key: 'activity',
                      label: t('labels.activity'),
                      onClick: () => setSortBy('activity'),
                    },
                    {
                      key: 'name',
                      label: t('labels.name'),
                      onClick: () => setSortBy('name'),
                    },
                    {
                      key: 'created',
                      label: t('labels.created'),
                      onClick: () => setSortBy('created'),
                    },
                  ],
                  selectedKeys: [sortBy],
                }}
                trigger={['click']}
              >
                <Button
                  type="text"
                  icon={<PiSortAscending />}
                  style={{
                    fontSize: '20px',
                  }}
                />
              </Dropdown>
              <PermissionGuard permissions={[Permission.ProjectsCreate]}>
                <Button
                  type="text"
                  icon={<PlusOutlined />}
                  onClick={() => openProjectDrawer()}
                  style={{
                    fontSize: '16px',
                  }}
                />
              </PermissionGuard>
            </div>
          </div>
        </div>
      </TitleBarWrapper>

      {/* Page Content */}
      <div className="flex-1 flex flex-col overflow-hidden items-center">
        {pageMinSize.xs && isSearchBoxVisible && (
          <div className={'w-full max-w-96 px-3 pt-3'}>
            {searchInputComponent}
          </div>
        )}
        {/* Projects Grid */}
        <div className="flex flex-1 flex-col w-full justify-center overflow-hidden">
          <div className={'h-full flex flex-col overflow-y-auto'}>
            <div className="max-w-4xl flex flex-wrap gap-3 pt-3 w-full self-center px-3">
              {getFilteredAndSortedProjects().map(project => (
                <div className={'min-w-70 flex-1'}>
                  <ProjectCard project={project} />
                </div>
              ))}
              {/* Placeholder divs for grid layout */}
              <div className={'min-w-70 flex-1'}></div>
              <div className={'min-w-70 flex-1'}></div>
              <div className={'min-w-70 flex-1'}></div>
            </div>
          </div>
        </div>

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
                onClick={() => openProjectDrawer()}
              >
                Create project
              </Button>
            )}
          </div>
        )}
      </div>

      <ProjectFormDrawer />
    </div>
  )
}
