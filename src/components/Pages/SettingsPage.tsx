import {
  CloudDownloadOutlined,
  EyeOutlined,
  GlobalOutlined,
  LockOutlined,
  RobotOutlined,
  SettingOutlined,
  TeamOutlined,
  ToolOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { Button, Dropdown, Flex, Menu, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { isTauriView } from '../../api/core'
import { Permission, usePermissions } from '../../permissions'
import { useMainContentMinSize } from '../hooks/useWindowMinSize'
import { TitleBarWrapper } from '../Common/TitleBarWrapper'
import { TauriDragRegion } from '../Common/TauriDragRegion'
import { IoIosArrowDown, IoIosArrowForward } from 'react-icons/io'
import { useEffect } from 'react'
import { setPreviousSettingPagePath } from '../../store/ui/navigate.ts'
import { Stores } from '../../store'

export function SettingsPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const location = useLocation()
  const { hasPermission } = usePermissions()
  const mainContentMinSize = useMainContentMinSize()
  const { token } = theme.useToken()
  const { isDesktop } = Stores.Auth

  const baseMenuItems = [
    {
      key: 'general',
      icon: <UserOutlined />,
      label: t('settings.general'),
    },
    {
      key: 'appearance',
      icon: <EyeOutlined />,
      label: t('settings.appearance'),
    },
    {
      key: 'privacy',
      icon: <LockOutlined />,
      label: t('settings.privacy'),
    },
    // Providers only shows in main menu for desktop apps
    ...(isDesktop
      ? [
          {
            key: 'providers',
            icon: <ToolOutlined />,
            label: t('settings.providers'),
          },
          {
            key: 'repositories',
            icon: <CloudDownloadOutlined />,
            label: t('settings.modelRepository.title'),
          },
        ]
      : []),

    // RAG Providers for desktop apps
    ...(isDesktop
      ? [
          {
            key: 'rag-providers',
            icon: <RobotOutlined />,
            label: 'RAG Providers',
          },
          {
            key: 'rag-repositories',
            icon: <CloudDownloadOutlined />,
            label: 'RAG Repositories',
          },
        ]
      : []),
    {
      key: 'hardware',
      icon: <ToolOutlined />,
      label: t('settings.hardware'),
    },
    // HTTPS Proxy only shows in main menu for desktop apps
    ...(isDesktop
      ? [
          {
            key: 'https-proxy',
            icon: <ToolOutlined />,
            label: t('settings.httpsProxy'),
          },
          {
            key: 'api-proxy-server',
            icon: <GlobalOutlined />,
            label: t('settings.apiProxyServer'),
          },
        ]
      : []),
    ...(isDesktop && isTauriView
      ? [
          // Ngrok only shows in main menu for desktop apps
          {
            key: 'web-app',
            icon: <GlobalOutlined />,
            label: t('settings.ngrok'),
          },
        ]
      : []),
  ]

  // Build admin menu items based on permissions
  const adminMenuItems = !isDesktop
    ? (() => {
        const items = []

        // Check if user has any admin permissions
        const hasUserManagement = hasPermission(Permission.users.read)
        const hasGroupManagement = hasPermission(Permission.groups.read)
        const hasAppearanceManagement = hasPermission(
          Permission.config.experimental.edit,
        )

        const hasProviderManagement = hasPermission(
          Permission.config.providers.read,
        )
        const hasRepositoryManagement = hasPermission(
          Permission.config.repositories.read,
        )
        const hasProxyManagement = hasPermission(Permission.config.proxy.read)
        const hasApiProxyManagement = hasPermission(
          Permission.config.apiProxyServer.read,
        )
        const hasAssistantsManagement = hasPermission(
          Permission.config.assistants.read,
        )
        const hasEngineManagement = hasPermission(
          Permission.config.engines.read,
        )

        if (
          hasUserManagement ||
          hasGroupManagement ||
          hasAppearanceManagement ||
          hasProviderManagement ||
          hasRepositoryManagement ||
          hasProxyManagement ||
          hasApiProxyManagement ||
          hasAssistantsManagement ||
          hasEngineManagement
        ) {
          items.push({
            type: 'divider' as const,
          })
          items.push({
            key: 'admin',
            icon: <SettingOutlined />,
            label: t('settings.admin'),
            type: 'group' as const,
          })

          if (hasAppearanceManagement) {
            items.push({
              key: 'admin-general',
              icon: <UserOutlined />,
              label: t('settings.general'),
            })
          }

          if (hasAppearanceManagement) {
            items.push({
              key: 'admin-appearance',
              icon: <EyeOutlined />,
              label: t('settings.appearance'),
            })
          }

          if (hasProviderManagement) {
            items.push({
              key: 'providers',
              icon: <ToolOutlined />,
              label: t('settings.providers'),
            })
          }

          if (hasRepositoryManagement) {
            items.push({
              key: 'repositories',
              icon: <CloudDownloadOutlined />,
              label: t('settings.modelRepository.title'),
            })
          }

          if (hasProviderManagement) {
            items.push({
              key: 'rag-providers',
              icon: <RobotOutlined />,
              label: 'RAG Providers',
            })
          }

          if (hasRepositoryManagement) {
            items.push({
              key: 'rag-repositories',
              icon: <CloudDownloadOutlined />,
              label: 'RAG Repositories',
            })
          }

          if (hasProxyManagement) {
            items.push({
              key: 'https-proxy',
              icon: <ToolOutlined />,
              label: t('settings.httpsProxy'),
            })
          }

          if (hasApiProxyManagement) {
            items.push({
              key: 'api-proxy-server',
              icon: <GlobalOutlined />,
              label: t('settings.apiProxyServer'),
            })
          }

          if (hasAssistantsManagement) {
            items.push({
              key: 'admin-assistants',
              icon: <RobotOutlined />,
              label: t('settings.assistants'),
            })
          }

          if (hasEngineManagement) {
            items.push({
              key: 'engines',
              icon: <ToolOutlined />,
              label: 'Engines',
            })
          }

          if (hasUserManagement) {
            items.push({
              key: 'users',
              icon: <UserOutlined />,
              label: t('settings.users'),
            })
          }

          if (hasGroupManagement) {
            items.push({
              key: 'user-groups',
              icon: <TeamOutlined />,
              label: t('settings.userGroups'),
            })
          }
        }

        return items
      })()
    : []

  const menuItems = [...baseMenuItems, ...adminMenuItems]

  // Extract the current settings section from the URL and validate it
  const urlSection = location.pathname.match(/\/settings\/([^/]+)/)?.[1]
  const validSections = menuItems
    .filter(item => 'key' in item && item.key)
    .map(item => (item as any).key)

  const currentSection = validSections.includes(urlSection)
    ? urlSection
    : 'general'

  useEffect(() => {
    setPreviousSettingPagePath(location.pathname)
  }, [location.pathname])

  const handleMenuClick = (key: string) => {
    navigate(`/settings/${key}`)
  }

  // Get current section display info
  const getCurrentSectionInfo = () => {
    const currentItem = menuItems.find(
      item => 'key' in item && item.key === currentSection,
    )
    return (
      currentItem || { icon: <SettingOutlined />, label: t('settings.title') }
    )
  }

  const SettingsMenu = () => (
    <Menu
      className={`
      w-fit
      h-full
      overflow-y-auto
      !p-1
      [&_.ant-menu]:!px-2
      [&_.ant-menu-item]:!h-8
      [&_.ant-menu-item]:!leading-[32px]
      `}
      style={{
        lineHeight: 1,
      }}
      selectedKeys={[currentSection || 'general']}
      items={menuItems}
      onClick={({ key }) => handleMenuClick(key)}
    />
  )

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Page Header */}
      <TitleBarWrapper>
        <div className="h-full flex items-center justify-between w-full">
          <TauriDragRegion className={'h-full w-full absolute top-0 left-0'} />
          <Typography.Title level={4} className="!m-0 !leading-tight truncate">
            {t('settings.title')}
          </Typography.Title>
          {mainContentMinSize.xs && (
            <div className="flex flex-1 items-center px-2">
              <IoIosArrowForward />
              <Dropdown
                overlayStyle={{
                  border: '1px solid ' + token.colorBorderSecondary,
                }}
                overlayClassName={`
                  rounded-md
                  `}
                menu={{
                  items: menuItems.map((item: any) => {
                    if ('type' in item && item.type === 'divider') {
                      return { type: 'divider' }
                    }
                    if ('type' in item && item.type === 'group') {
                      return {
                        type: 'group',
                        label: (
                          <div className={'-ml-1'}>
                            <Typography.Text
                              strong
                              type={'secondary'}
                              className={'!text-xs'}
                            >
                              {item.label}
                            </Typography.Text>
                          </div>
                        ),
                      }
                    }
                    return {
                      key: item.key,
                      label: (
                        <Flex className={'gap-2'}>
                          {item.icon}
                          {item.label}
                        </Flex>
                      ),
                    }
                  }),
                  onClick: ({ key }) => {
                    handleMenuClick(key)
                  },
                  selectedKeys: [currentSection || 'general'],
                }}
                trigger={['click']}
              >
                <Button type="text" className={'mt-[2px]'}>
                  {getCurrentSectionInfo().icon} {getCurrentSectionInfo().label}{' '}
                  <IoIosArrowDown />
                </Button>
              </Dropdown>
            </div>
          )}
        </div>
      </TitleBarWrapper>

      {/* Page Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Desktop Sidebar */}
        {!mainContentMinSize.xs && (
          <div className="w-fit">
            <SettingsMenu />
          </div>
        )}

        {/* Main Content Area */}
        <div className="flex-1 overflow-hidden">
          <Outlet />
        </div>
      </div>
    </div>
  )
}
