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
  clearProvidersError,
  openAddProviderDrawer,
  Stores,
} from '../../../../store'
import { PROVIDER_ICONS } from '../../../../constants/providers'
import { AddLocalModelDownloadDrawer } from './AddLocalModelDownloadDrawer.tsx'
import { AddLocalModelUploadDrawer } from './AddLocalModelUploadDrawer.tsx'
import { AddProviderDrawer } from './AddProviderDrawer.tsx'
import { AddRemoteModelDrawer } from './AddRemoteModelDrawer.tsx'
import { EditLocalModelDrawer } from './EditLocalModelDrawer.tsx'
import { EditRemoteModelDrawer } from './EditRemoteModelDrawer.tsx'
import { LocalProviderSettings } from './LocalProviderSettings'
import { RemoteProviderSettings } from './RemoteProviderSettings'
import { useMainContentMinSize } from '../../../hooks/useWindowMinSize.ts'
import { IoIosArrowDown } from 'react-icons/io'

const { Title } = Typography

export function ProvidersSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { providerId } = useParams<{ providerId?: string }>()
  const navigate = useNavigate()
  const mainContentMinSize = useMainContentMinSize()

  // Model providers store
  const { providers, loading, error } = Stores.AdminProviders

  const currentProvider = providers.find(p => p.id === providerId)

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearProvidersError()
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
          navigate(`/settings/providers/${providers[0].id}`, {
            replace: true,
          })
        }
      } else {
        // No URL parameter, navigate to first provider
        navigate(`/settings/providers/${providers[0].id}`, {
          replace: true,
        })
      }
    }
  }, [providers, providerId])

  const menuItems = providers.map(provider => {
    const IconComponent = PROVIDER_ICONS[provider.type]
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
    label: <Typography.Text>Add Provider</Typography.Text>,
  })

  const ProviderMenu = () => (
    <Menu
      className={`
      w-full
      h-full
      !m-0
      overflow-y-auto
      [&_.ant-menu]:!px-0
      [&_.ant-menu-item]:!h-8
      [&_.ant-menu-item]:!leading-[32px]
      !bg-transparent
      !border-none`}
      selectedKeys={providerId ? [providerId] : []}
      items={menuItems}
      onClick={({ key }) => {
        if (key === 'add-provider') {
          openAddProviderDrawer()
        } else {
          navigate(`/settings/providers/${key}`)
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

    // Render appropriate provider settings component based on type
    if (currentProvider.type === 'local') {
      return <LocalProviderSettings />
    }

    return <RemoteProviderSettings />
  }

  return (
    <div className="flex flex-col gap-3 h-full overflow-y-hidden">
      <div
        className={'flex w-full flex-1 overflow-y-auto relative justify-center'}
      >
        <div className={'w-full h-full max-w-4xl flex self-center'}>
          {!mainContentMinSize.sm && (
            <div
              className={'w-42 flex flex-col gap-2 overflow-y-auto h-full pt-3'}
            >
              <div className={'w-full px-3'}>
                <Title level={4} className="!m-0 !leading-tight">
                  Providers
                </Title>
              </div>
              <div className={'flex-1 pl-2'}>
                <ProviderMenu />
              </div>
            </div>
          )}
          {/* Main Content */}
          <div className={'flex flex-1'}>
            <div className={'flex w-full flex-col py-3 px-3 overflow-y-auto'}>
              {mainContentMinSize.sm && (
                <div className={'w-full flex flex-row gap-2 items-center mb-4'}>
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
                          openAddProviderDrawer()
                        } else {
                          navigate(`/settings/providers/${key}`)
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
                              PROVIDER_ICONS[currentProvider.type]
                            return <IconComponent className="text-lg" />
                          })()}
                          {currentProvider.name}
                        </Flex>
                      ) : (
                        'Select Provider'
                      )}
                      <IoIosArrowDown />
                    </Button>
                  </Dropdown>
                </div>
              )}
              {renderProviderSettings()}
              <div className={'w-full h-3 block'} />
            </div>
          </div>
        </div>

        {/* Modals */}
        <AddProviderDrawer />

        <AddLocalModelUploadDrawer />
        <AddLocalModelDownloadDrawer />
        <AddRemoteModelDrawer />

        <EditLocalModelDrawer />
        <EditRemoteModelDrawer />
      </div>
    </div>
  )
}
