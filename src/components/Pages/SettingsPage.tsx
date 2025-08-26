import { Button, Dropdown, Flex, Menu, theme, Typography } from 'antd'
import { useTranslation } from 'react-i18next'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { isTauriView } from '../../api/core'
import { useMainContentMinSize } from '../hooks/useWindowMinSize'
import { TitleBarWrapper } from '../common/TitleBarWrapper'
import { TauriDragRegion } from '../common/TauriDragRegion'
import {
  IoIosArrowDown,
  IoIosArrowForward,
  IoMdEye,
  IoMdGlobe,
  IoMdLock,
  IoMdPeople,
  IoMdPerson,
  IoMdSettings,
} from 'react-icons/io'
import {
  FaCogs,
  FaMicrochip,
  FaNetworkWired,
  FaRobot,
  FaServer,
  FaShieldAlt,
} from 'react-icons/fa'
import { MdStorage } from 'react-icons/md'
import { useEffect } from 'react'
import { setPreviousSettingPagePath } from '../../store/ui/navigate.ts'
import { Stores } from '../../store'
import { hasPermission } from '../../permissions/utils.ts'
import { Permission } from '../../types'

export function SettingsPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const location = useLocation()
  const mainContentMinSize = useMainContentMinSize()
  const { token } = theme.useToken()
  const { isDesktop } = Stores.Auth

  const baseMenuItems = [
    {
      key: 'general',
      icon: <IoMdPerson />,
      label: t('settings.general'),
    },
    {
      key: 'appearance',
      icon: <IoMdEye />,
      label: t('settings.appearance'),
    },
    {
      key: 'privacy',
      icon: <IoMdLock />,
      label: t('settings.privacy'),
    },
    // Providers only shows in main menu for desktop apps
    ...(isDesktop
      ? [
          {
            key: 'providers',
            icon: <FaServer />,
            label: t('settings.providers'),
          },
          {
            key: 'repositories',
            icon: <MdStorage />,
            label: t('settings.modelRepository.title'),
          },
          {
            key: 'rag-providers',
            icon: <FaRobot />,
            label: 'RAG Providers',
          },
          // {
          //   key: 'rag-repositories',
          //   icon: <FaDatabase />,
          //   label: 'RAG Repositories',
          // },
          {
            key: 'engines',
            icon: <FaCogs />,
            label: 'Engines',
          },
          {
            key: 'https-proxy',
            icon: <FaShieldAlt />,
            label: t('settings.httpsProxy'),
          },
          {
            key: 'api-proxy-server',
            icon: <IoMdGlobe />,
            label: t('settings.apiProxyServer'),
          },
          {
            key: 'hardware',
            icon: <FaMicrochip />,
            label: t('settings.hardware'),
          },
        ]
      : []),

    ...(isDesktop && isTauriView
      ? [
          {
            key: 'web-app',
            icon: <FaNetworkWired />,
            label: t('settings.ngrok'),
          },
        ]
      : []),
  ]

  // Build admin menu items based on permissions
  const adminMenuItems = !isDesktop
    ? (() => {
        const items = []

        items.push({
          type: 'divider' as const,
        })
        items.push({
          key: 'admin',
          icon: <IoMdSettings />,
          label: t('settings.admin'),
          type: 'group' as const,
        })

        items.push({
          key: 'admin-general',
          icon: <IoMdPerson />,
          label: t('settings.general'),
        })

        if (hasPermission([Permission.ConfigAppearanceRead])) {
          items.push({
            key: 'admin-appearance',
            icon: <IoMdEye />,
            label: t('settings.appearance'),
          })
        }
        if (hasPermission([Permission.ProvidersRead])) {
          items.push({
            key: 'providers',
            icon: <FaServer />,
            label: t('settings.providers'),
          })
        }
        if (hasPermission([Permission.RepositoriesRead])) {
          items.push({
            key: 'repositories',
            icon: <MdStorage />,
            label: t('settings.modelRepository.title'),
          })
        }
        if (hasPermission([Permission.RagProvidersRead])) {
          items.push({
            key: 'rag-providers',
            icon: <FaRobot />,
            label: 'RAG Providers',
          })
        }
        // if (hasPermission([Permission.RagRepositoriesRead])) {
        //   items.push({
        //     key: 'rag-repositories',
        //     icon: <FaDatabase />,
        //     label: 'RAG Repositories',
        //   })
        // }
        if (hasPermission([Permission.ConfigProxyRead])) {
          items.push({
            key: 'https-proxy',
            icon: <FaShieldAlt />,
            label: t('settings.httpsProxy'),
          })
        }
        if (hasPermission([Permission.ApiProxyRead])) {
          items.push({
            key: 'api-proxy-server',
            icon: <IoMdGlobe />,
            label: t('settings.apiProxyServer'),
          })
        }
        if (hasPermission([Permission.AssistantsAdminRead])) {
          items.push({
            key: 'admin-assistants',
            icon: <FaRobot />,
            label: t('settings.assistants'),
          })
        }
        if (hasPermission([Permission.EnginesRead])) {
          items.push({
            key: 'engines',
            icon: <FaCogs />,
            label: 'Engines',
          })
        }
        if (hasPermission([Permission.UsersRead])) {
          items.push({
            key: 'users',
            icon: <IoMdPerson />,
            label: t('settings.users'),
          })
        }
        if (hasPermission([Permission.GroupsRead])) {
          items.push({
            key: 'user-groups',
            icon: <IoMdPeople />,
            label: t('settings.userGroups'),
          })
        }
        if (hasPermission([Permission.HardwareRead])) {
          items.push({
            key: 'hardware',
            icon: <FaMicrochip />,
            label: t('settings.hardware'),
          })
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
    return currentItem || { icon: <IoMdSettings />, label: t('settings.title') }
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
                        <Flex className={'gap-2 items-center'}>
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
