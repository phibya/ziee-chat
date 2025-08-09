import React, { useEffect, useState } from 'react'
import { App, Button, Dropdown, Input, Typography } from 'antd'
import {
  CopyOutlined,
  PlusOutlined,
  RobotOutlined,
  SearchOutlined,
} from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import {
  clearAssistantsStoreError,
  loadUserAssistants,
  openAssistantDrawer,
  Stores,
} from '../../../store'
import { Assistant } from '../../../types/api/assistant'
import { AssistantFormDrawer } from '../../Common/AssistantFormDrawer.tsx'
import { isDesktopApp } from '../../../api/core.ts'
import { AssistantCard } from './AssistantCard.tsx'
import { TemplateAssistantDrawer } from './TemplateAssistantDrawer.tsx'
import { TitleBarWrapper } from '../../Common/TitleBarWrapper.tsx'
import { TauriDragRegion } from '../../Common/TauriDragRegion.tsx'
import { PiSortAscending } from 'react-icons/pi'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'

const { Title, Text } = Typography

export const AssistantsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const pageMinSize = useMainContentMinSize()
  const [isSearchBoxVisible, setIsSearchBoxVisible] = useState(false)

  // Assistants store
  const { assistants: allAssistants, loading, error } = Stores.Assistants

  const assistants = Array.from(allAssistants.values()).filter(
    (a: Assistant) => !a.is_template,
  )
  const templateAssistants = Array.from(allAssistants.values()).filter(
    (a: Assistant) => a.is_template,
  )

  const [templateModalVisible, setTemplateModalVisible] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'activity' | 'name' | 'created'>(
    'activity',
  )

  useEffect(() => {
    loadUserAssistants()
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearAssistantsStoreError()
    }
  }, [error, message])

  const handleCreate = () => {
    openAssistantDrawer()
  }

  const handleCloneFromTemplate = () => {
    setTemplateModalVisible(true)
  }

  // Get filtered and sorted assistants
  const getFilteredAndSortedAssistants = () => {
    let filteredAssistants = assistants

    // Apply search filter
    if (searchQuery.trim()) {
      filteredAssistants = assistants.filter(
        assistant =>
          assistant.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          assistant.description
            ?.toLowerCase()
            .includes(searchQuery.toLowerCase()),
      )
    }

    // Sort assistants based on sortBy selection
    let sortedAssistants = [...filteredAssistants]
    switch (sortBy) {
      case 'activity':
        sortedAssistants.sort(
          (a, b) =>
            new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
        )
        break
      case 'name':
        sortedAssistants.sort((a, b) => a.name.localeCompare(b.name))
        break
      case 'created':
        sortedAssistants.sort(
          (a, b) =>
            new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
        )
        break
    }

    return sortedAssistants
  }

  const searchInputComponent = (
    <Input
      placeholder={t('forms.searchAssistants')}
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
            {t('assistants.title')}
          </Typography.Title>
          <div className="h-full flex items-center justify-between">
            {!pageMinSize.xs ? (
              searchInputComponent
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
              {!isDesktopApp && (
                <Button
                  type="text"
                  icon={<CopyOutlined />}
                  onClick={handleCloneFromTemplate}
                  style={{
                    fontSize: '16px',
                  }}
                />
              )}
              <Button
                type="text"
                icon={<PlusOutlined />}
                onClick={handleCreate}
                style={{
                  fontSize: '16px',
                }}
              />
            </div>
          </div>
        </div>
      </TitleBarWrapper>

      {/* Page Content */}
      <div className="flex-1 flex flex-col overflow-hidden items-center p-3">
        {pageMinSize.xs && isSearchBoxVisible && (
          <div className={'w-full max-w-96'}>{searchInputComponent}</div>
        )}
        {/* Assistants Grid */}
        <div className="flex flex-1 flex-col w-full justify-center overflow-hidden">
          {loading ? (
            <div className="flex justify-center items-center py-12">
              <div>Loading assistants...</div>
            </div>
          ) : (
            <div className="max-w-4xl flex flex-wrap gap-3 pt-4 w-full h-auto self-center overflow-y-auto">
              {getFilteredAndSortedAssistants().map((assistant: Assistant) => (
                <div key={assistant.id} className={'min-w-72 flex-1'}>
                  <AssistantCard assistant={assistant} />
                </div>
              ))}
              {/* Placeholder divs for grid layout */}
              <div className={'min-w-72 flex-1'}></div>
              <div className={'min-w-72 flex-1'}></div>
              <div className={'min-w-72 flex-1'}></div>
            </div>
          )}
          <div className={'w-full flex-1'} />
        </div>

        {/* Empty State */}
        {!loading && getFilteredAndSortedAssistants().length === 0 && (
          <div className="text-center py-12">
            <RobotOutlined className="text-6xl mb-4" />
            <Title level={3} type="secondary">
              {searchQuery ? 'No assistants found' : 'No assistants yet'}
            </Title>
            <Text type="secondary" className="block mb-4">
              {searchQuery
                ? 'Try adjusting your search criteria'
                : 'Create your first assistant to get started'}
            </Text>
            {!searchQuery && (
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={handleCreate}
              >
                Create assistant
              </Button>
            )}
          </div>
        )}
      </div>

      <AssistantFormDrawer />

      <TemplateAssistantDrawer
        open={templateModalVisible}
        onClose={() => setTemplateModalVisible(false)}
        templateAssistants={templateAssistants}
      />
    </div>
  )
}
