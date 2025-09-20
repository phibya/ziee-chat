import { PlusOutlined } from '@ant-design/icons'
import {
  App,
  Button,
  Dropdown,
  Empty,
  Flex,
  Menu,
  Spin,
  Typography,
} from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import {
  clearRAGProvidersError,
  openAddRAGProviderDrawer,
  Stores,
} from '../../../../store'
import { RAG_PROVIDER_ICONS } from '../../../../constants/ragProviders'
import { AddRAGProviderDrawer } from './AddRAGProviderDrawer'
import { EditRAGProviderDrawer } from './EditRAGProviderDrawer'
import { AddSystemInstanceDrawer } from './AddSystemInstanceDrawer'
import { EditSystemInstanceDrawer } from './EditSystemInstanceDrawer'
import { RAGProviderSettings } from './RAGProviderSettings'
import { useMainContentMinSize } from '../../../hooks/useWindowMinSize'
import { IoIosArrowDown } from 'react-icons/io'
import { DivScrollY } from '../../../common/DivScrollY.tsx'

const { Title } = Typography

export function RAGProvidersSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()
  const navigate = useNavigate()
  const mainContentMinSize = useMainContentMinSize()

  // RAG providers store
  const { providers, loading, error } = Stores.AdminRAGProviders

  const currentProvider = providers.find(p => p.id === providerId)

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearRAGProvidersError()
    }
  }, [error, message])

  // Handle URL parameter and provider selection
  useEffect(() => {
    if (providers.length > 0) {
      if (providerId) {
        // If URL has providerId, check if it's valid
        const providerExists = providers.find(p => p.id === providerId)
        if (!providerExists) {
          // Provider doesn't exist, redirect to first provider
          navigate(`/settings/rag-providers/${providers[0].id}`, {
            replace: true,
          })
        }
      } else {
        // No URL parameter, navigate to first provider
        navigate(`/settings/rag-providers/${providers[0].id}`, {
          replace: true,
        })
      }
    }
  }, [providers, providerId])

  const menuItems = providers.map(provider => {
    const IconComponent = RAG_PROVIDER_ICONS[provider.type]
    return {
      key: provider.id,
      label: (
        <Flex className={'flex-row gap-2 items-center h-full'}>
          <IconComponent className={'text-lg'} />
          <div className={'flex-1 flex items-center h-full overflow-x-hidden'}>
            <Typography.Text ellipsis>{provider.name}</Typography.Text>
          </div>
        </Flex>
      ),
    }
  })

  menuItems.push({
    key: 'add-provider',
    //@ts-ignore
    icon: <PlusOutlined />,
    label: <Typography.Text>Add RAG Provider</Typography.Text>,
  })

  const ProviderMenu = () => (
    <Menu
      className={`
      w-full
      h-full
      !m-0
      [&_.ant-menu]:!px-0
      [&_.ant-menu-item]:!h-8
      [&_.ant-menu-item]:!leading-[32px]
      !bg-transparent
      !border-none`}
      selectedKeys={providerId ? [providerId] : []}
      items={menuItems}
      onClick={({ key }) => {
        if (key === 'add-provider') {
          openAddRAGProviderDrawer()
        } else {
          navigate(`/settings/rag-providers/${key}`)
        }
      }}
    />
  )

  const renderProviderSettings = () => {
    if (loading) {
      return (
        <div style={{ textAlign: 'center', padding: '50px' }}>
          <Spin size="large" />
        </div>
      )
    }

    if (!currentProvider) {
      return (
        <Empty
          description={t('providers.noProviderSelected')}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      )
    }

    return <RAGProviderSettings />
  }

  return (
    <div className="flex flex-col gap-3 h-full overflow-y-hidden">
      <DivScrollY className={'flex w-full flex-1 relative justify-center'}>
        <div className={'w-full h-full flex self-center'}>
          {!mainContentMinSize.sm && (
            <div className={'w-42 flex flex-col gap-2 h-full pt-3'}>
              <div className={'w-full px-3'}>
                <Title level={4} className="!m-0 !leading-tight">
                  RAG Providers
                </Title>
              </div>
              <DivScrollY className={'flex-1 pl-2'}>
                <ProviderMenu />
              </DivScrollY>
            </div>
          )}
          {/* Main Content */}
          <div className={'flex flex-1 max-w-full'}>
            <DivScrollY
              className={'flex w-full flex-col py-3 px-3 overflow-x-hidden'}
            >
              <div className={'flex flex-col flex-1 max-w-3xl m-auto'}>
                {mainContentMinSize.sm && (
                  <div
                    className={'w-full flex flex-row gap-2 items-center mb-4'}
                  >
                    <Dropdown
                      className={'w-full'}
                      menu={{
                        items: menuItems.map(item => ({
                          // @ts-ignore
                          icon: item.icon,
                          key: item.key,
                          label: item.label,
                        })),
                        onClick: ({ key }) => {
                          if (key === 'add-provider') {
                            openAddRAGProviderDrawer()
                          } else {
                            navigate(`/settings/rag-providers/${key}`)
                          }
                        },
                        selectedKeys: providerId ? [providerId] : [],
                      }}
                      trigger={['click']}
                    >
                      <Button className="w-fit" size={'large'}>
                        {currentProvider ? (
                          <Flex className="gap-2 items-center">
                            {(() => {
                              const IconComponent =
                                RAG_PROVIDER_ICONS[currentProvider.type]
                              return <IconComponent className="text-lg" />
                            })()}
                            {currentProvider.name}
                          </Flex>
                        ) : (
                          'Select RAG Provider'
                        )}
                        <IoIosArrowDown />
                      </Button>
                    </Dropdown>
                  </div>
                )}
                {renderProviderSettings()}
                <div className={'w-full h-3 block'} />
              </div>
            </DivScrollY>
          </div>
        </div>

        {/* Modals */}
        <AddRAGProviderDrawer />
        <EditRAGProviderDrawer />
        <AddSystemInstanceDrawer />
        <EditSystemInstanceDrawer />
      </DivScrollY>
    </div>
  )
}
