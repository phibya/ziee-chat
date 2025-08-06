import React, { useEffect, useState } from 'react'
import { App, Button, Input, Select, Typography } from 'antd'
import { FolderOutlined, PlusOutlined, SearchOutlined } from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import { PageContainer } from '../../common/PageContainer.tsx'
import {
  clearProjectsStoreError,
  loadAllUserProjects,
  openProjectDrawer,
  Stores,
} from '../../../store'
import { ProjectFormDrawer } from './ProjectFormDrawer.tsx'
import { ProjectCard } from './ProjectCard.tsx'

const { Title, Text } = Typography
const { Search } = Input

export const ProjectsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()

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

  return (
    <PageContainer
      title={'Projects'}
      extra={
        <div className={'w-full flex justify-end'}>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => openProjectDrawer()}
          >
            New project
          </Button>
        </div>
      }
    >
      {/* Search and Sort */}
      <div className="flex w-full items-center mb-6 gap-3 flex-wrap max-w-6xl px-3">
        <Search
          placeholder={t('forms.searchProjects')}
          prefix={<SearchOutlined />}
          className={'w-full items-center justify-center flex-1'}
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
      <div className="flex items-center w-full justify-center overflow-y-auto">
        <div className="max-w-6xl flex flex-wrap gap-4 p-3 w-full">
          {getFilteredAndSortedProjects().map(project => (
            <div className={'min-w-56 flex-1'}>
              <ProjectCard project={project} />
            </div>
          ))}
          {/* Placeholder divs for grid layout */}
          <div className={'min-w-56 flex-1'}></div>
          <div className={'min-w-56 flex-1'}></div>
          <div className={'min-w-56 flex-1'}></div>
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

      <ProjectFormDrawer />
    </PageContainer>
  )
}
