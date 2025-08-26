import { Link, useLocation } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Divider, Dropdown, theme, Typography } from 'antd'
import {
  AppstoreOutlined,
  FolderOutlined,
  HistoryOutlined,
  LogoutOutlined,
  PlusOutlined,
  SettingOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { logoutUser, setSidebarCollapsed, Stores } from '../../../store'
import { DownloadIndicator } from './DownloadIndicator.tsx'
import { isTauriView } from '../../../api/core.ts'
import { RecentConversations } from './RecentConversations'
import { TauriDragRegion } from '../../common/TauriDragRegion'
import { useWindowMinSize } from '../../hooks/useWindowMinSize.ts'
import { HiOutlineFaceSmile } from 'react-icons/hi2'
import { PermissionGuard } from '../../Auth/PermissionGuard.tsx'
import { Permission } from '../../../types'
import { GoDatabase } from 'react-icons/go'

const { Text } = Typography

interface SidebarItemProps {
  icon: React.ReactNode
  label: string
  isActive?: boolean
  to?: string
  onClick?: () => void
}

function SidebarItem({ icon, label, isActive, to }: SidebarItemProps) {
  const { token } = theme.useToken()
  const windowMinSize = useWindowMinSize()
  return (
    <Link
      to={to || '#'}
      onClick={() => {
        if (windowMinSize.xs) {
          setSidebarCollapsed(true)
        }
      }}
      className="flex items-center px-3 py-1 mx-2 rounded-md cursor-pointer transition-colors duration-150 no-underline"
      style={{
        textDecoration: 'none',
        backgroundColor: isActive ? token.colorPrimary : 'transparent',
        color: isActive ? token.colorTextLightSolid : token.colorTextBase,
      }}
      onMouseEnter={e => {
        if (!isActive) {
          e.currentTarget.style.backgroundColor = token.colorPrimaryHover
        }
      }}
      onMouseLeave={e => {
        if (!isActive) {
          e.currentTarget.style.backgroundColor = 'transparent'
        }
      }}
    >
      <div
        className="w-4 h-4 mr-1.5 flex items-center justify-center"
        style={{
          color: isActive ? token.colorTextLightSolid : token.colorTextBase,
          transition: 'color 0.15s ease',
        }}
      >
        {icon}
      </div>
      <Text style={{ color: 'inherit' }}>{label}</Text>
    </Link>
  )
}

interface SectionHeaderProps {
  children: React.ReactNode
}

function SectionHeader({ children }: SectionHeaderProps) {
  const { token } = theme.useToken()
  return (
    <Text
      className="px-3 pb-0.5 block font-semibold tracking-wide"
      style={{
        fontSize: '11px',
        color: token.colorTextSecondary,
      }}
    >
      {children}
    </Text>
  )
}

export function LeftSidebar() {
  const { t } = useTranslation()
  const location = useLocation()
  const { token } = theme.useToken()
  const windowMinSize = useWindowMinSize()

  const { user, isDesktop } = Stores.Auth
  const { previousSettingPagePath } = Stores.UI.PathHistory

  const isActive = (path: string) => {
    if (path === '/conversations')
      return location.pathname.startsWith('/conversations')
    if (path === '/projects') return location.pathname.startsWith('/projects')
    if (path === '/rags') return location.pathname.startsWith('/rags')
    if (path === '/artifacts') return location.pathname.startsWith('/artifacts')
    if (path === '/hub') return location.pathname.startsWith('/hub')
    if (path === '/assistants')
      return location.pathname.startsWith('/assistants')
    if (path === '/settings') return location.pathname.startsWith('/settings')
    return false
  }

  return (
    <div
      className="h-full flex flex-col overflow-hidden"
      style={{
        width: '100%', // Take full width of container
        borderRight: windowMinSize.xs
          ? 'none'
          : '1px solid ' + token.colorBorderSecondary,
        backgroundColor: isTauriView ? 'transparent' : token.colorBgContainer,
      }}
    >
      <TauriDragRegion className={'h-[50px]'} />
      {/* Sidebar content - always rendered */}
      {/* New Chat Button */}

      <PermissionGuard permissions={[Permission.ChatCreate]}>
        <div className="mb-4">
          <SidebarItem icon={<PlusOutlined />} label="New Chat" to="/" />
        </div>
      </PermissionGuard>

      {/* Navigation Section */}
      <div className="mb-4">
        <SectionHeader>Navigation</SectionHeader>
        <div className="space-y-0">
          <PermissionGuard permissions={[Permission.ChatRead]}>
            <SidebarItem
              icon={<HistoryOutlined />}
              label={t('navigation.chats')}
              isActive={isActive('/conversations')}
              to="/conversations"
            />
          </PermissionGuard>
          <PermissionGuard permissions={[Permission.ProjectsRead]}>
            <SidebarItem
              icon={<FolderOutlined />}
              label={t('navigation.projects')}
              isActive={isActive('/projects')}
              to="/projects"
            />
          </PermissionGuard>
          <PermissionGuard permissions={[Permission.RagInstancesRead]}>
            <SidebarItem
              icon={<GoDatabase />}
              label={'RAG'}
              isActive={isActive('/rags')}
              to="/rags"
            />
          </PermissionGuard>
          {/*<SidebarItem*/}
          {/*  icon={<BlockOutlined />}*/}
          {/*  label={t('navigation.artifacts')}*/}
          {/*  isActive={isActive('/artifacts')}*/}
          {/*  to="/artifacts"*/}
          {/*/>*/}
        </div>
      </div>

      {/* Recent Section */}
      <div className="flex-1 overflow-hidden flex flex-col">
        <PermissionGuard permissions={[Permission.ChatRead]}>
          <SectionHeader>Recent</SectionHeader>
          <RecentConversations />
        </PermissionGuard>
      </div>

      {/* Tools Section */}
      <div>
        <SectionHeader>Tools</SectionHeader>
        <div className="space-y-0 mb-2">
          <PermissionGuard
            permissions={[
              Permission.HubModelsRead,
              Permission.HubAssistantsRead,
            ]}
            match={'any'}
          >
            <SidebarItem
              icon={<AppstoreOutlined />}
              label={t('navigation.hub')}
              isActive={isActive('/hub')}
              to="/hub"
            />
          </PermissionGuard>
          <PermissionGuard permissions={[Permission.AssistantsRead]}>
            <SidebarItem
              icon={<HiOutlineFaceSmile />}
              label={t('navigation.assistants')}
              isActive={isActive('/assistants')}
              to="/assistants"
            />
          </PermissionGuard>
          <SidebarItem
            icon={<SettingOutlined />}
            label={t('navigation.settings')}
            isActive={isActive('/settings')}
            to={previousSettingPagePath}
          />
        </div>

        {/* Download Indicator */}
        <div className="px-2">
          <DownloadIndicator />
        </div>

        {/* User Profile Section */}
        {user && !isTauriView && (
          <div>
            <Divider className={'!m-0'} />
            {isDesktop ? (
              <div onClick={logoutUser}>
                <SidebarItem icon={<LogoutOutlined />} label={'Logout'} />
              </div>
            ) : (
              <Dropdown
                menu={{
                  items: [
                    {
                      key: 'profile',
                      icon: <UserOutlined />,
                      label: t('navigation.profile'),
                      onClick: () => console.log('Profile clicked'),
                    },
                    {
                      key: 'logout',
                      icon: <LogoutOutlined />,
                      label: t('navigation.logout'),
                      onClick: async () => await logoutUser(),
                    },
                  ].filter(Boolean),
                }}
                placement="topLeft"
                trigger={['click']}
              >
                <div>
                  <SidebarItem icon={<UserOutlined />} label={user.username} />
                </div>
              </Dropdown>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
