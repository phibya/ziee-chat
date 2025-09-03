import React, { useEffect, useState } from 'react'
import { App, Button, Dropdown, Input, Typography } from 'antd'
import {
  DatabaseOutlined,
  PlusOutlined,
  SearchOutlined,
} from '@ant-design/icons'
import { useTranslation } from 'react-i18next'
import {
  clearRAGStoreError,
  loadAllUserRAGInstances,
  openRAGInstanceDrawer,
  Stores,
} from '../../../store'
import { RagFormDrawer } from './RagFormDrawer.tsx'
import { RagCard } from './RagCard.tsx'
import { TitleBarWrapper } from '../../common/TitleBarWrapper.tsx'
import { TauriDragRegion } from '../../common/TauriDragRegion.tsx'
import { PiSortAscending } from 'react-icons/pi'
import { useMainContentMinSize } from '../../hooks/useWindowMinSize.ts'
import { Permission } from '../../../types'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'

const { Title, Text } = Typography

export const RagsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const pageMinSize = useMainContentMinSize()
  const [isSearchBoxVisible, setIsSearchBoxVisible] = useState(false)

  // RAG store
  const { ragInstances, loading, error } = Stores.RAG

  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'activity' | 'name' | 'created'>(
    'activity',
  )

  useEffect(() => {
    loadAllUserRAGInstances(true) // Always load system instances
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearRAGStoreError()
    }
  }, [error, message])

  // Get filtered and sorted RAG instances
  const getFilteredAndSortedInstances = () => {
    let filteredInstances = ragInstances

    // Apply search filter
    if (searchQuery.trim()) {
      filteredInstances = ragInstances.filter(
        instance =>
          instance.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          instance.description
            ?.toLowerCase()
            .includes(searchQuery.toLowerCase()),
      )
    }

    // Sort instances based on sortBy selection
    let sortedInstances = [...filteredInstances]
    switch (sortBy) {
      case 'activity':
        sortedInstances.sort(
          (a, b) =>
            new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
        )
        break
      case 'name':
        sortedInstances.sort((a, b) => a.name.localeCompare(b.name))
        break
      case 'created':
        sortedInstances.sort(
          (a, b) =>
            new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
        )
        break
    }

    return sortedInstances
  }

  // Separate instances by type for display
  const getInstancesByType = () => {
    const allInstances = getFilteredAndSortedInstances()
    const userInstances = allInstances.filter(r => !r.is_system)
    const systemInstances = allInstances.filter(r => r.is_system)
    return { userInstances, systemInstances, allInstances }
  }

  const searchInputComponent = (
    <Input
      placeholder={t('forms.searchRAGInstances')}
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
            RAG Instances
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
              <PermissionGuard permissions={[Permission.RagInstancesCreate]}>
                <Button
                  type="text"
                  icon={<PlusOutlined />}
                  onClick={() => openRAGInstanceDrawer()}
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
        {/* RAG Instances Grid */}
        {(() => {
          const { userInstances, systemInstances, allInstances } =
            getInstancesByType()

          if (allInstances.length === 0) {
            return null
          }

          return (
            <div className="flex flex-1 flex-col w-full justify-center overflow-hidden">
              <div className={'h-full flex flex-col overflow-y-auto'}>
                <div className="max-w-4xl flex flex-col gap-4 pt-3 w-full self-center px-3">
                  {/* Personal Instances Section */}
                  {userInstances.length > 0 && (
                    <div>
                      {systemInstances.length > 0 && (
                        <Typography.Title level={5} className="!mb-3">
                          Personal Instances
                        </Typography.Title>
                      )}
                      <div className="flex flex-wrap gap-3">
                        {userInstances.map(instance => (
                          <div key={instance.id} className={'min-w-70 flex-1'}>
                            <RagCard ragInstance={instance} />
                          </div>
                        ))}
                        {/* Placeholder divs for grid layout */}
                        <div className={'min-w-70 flex-1'}></div>
                        <div className={'min-w-70 flex-1'}></div>
                        <div className={'min-w-70 flex-1'}></div>
                      </div>
                    </div>
                  )}

                  {/* System Instances Section */}
                  {systemInstances.length > 0 && (
                    <div>
                      <Typography.Title level={5} className="!mb-3">
                        System Instances
                      </Typography.Title>
                      <div className="flex flex-wrap gap-3">
                        {systemInstances.map(instance => (
                          <div key={instance.id} className={'min-w-70 flex-1'}>
                            <RagCard ragInstance={instance} />
                          </div>
                        ))}
                        {/* Placeholder divs for grid layout */}
                        <div className={'min-w-70 flex-1'}></div>
                        <div className={'min-w-70 flex-1'}></div>
                        <div className={'min-w-70 flex-1'}></div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )
        })()}

        {/* Empty State */}
        {!loading && getInstancesByType().allInstances.length === 0 && (
          <div className="text-center py-12 m-auto">
            <DatabaseOutlined className="text-6xl mb-4" />
            <Title level={3} type="secondary">
              {searchQuery ? 'No RAG instances found' : 'No RAG instances yet'}
            </Title>
            <Text type="secondary" className="block mb-4">
              {searchQuery
                ? 'Try adjusting your search criteria'
                : 'Create your first RAG instance to get started'}
            </Text>
            {!searchQuery && (
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={() => openRAGInstanceDrawer()}
              >
                Create RAG Instance
              </Button>
            )}
          </div>
        )}
      </div>

      <RagFormDrawer />
    </div>
  )
}
