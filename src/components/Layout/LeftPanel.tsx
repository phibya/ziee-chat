import { useLocation, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Button, Tooltip, Typography } from 'antd'
import {
  AppstoreOutlined,
  BlockOutlined,
  DatabaseOutlined,
  FolderOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  MessageOutlined,
  PlusOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import { useAppStore } from '../../store'
import { useSettingsStore } from '../../store/settings'
import { useTheme } from '../../hooks/useTheme'

const { Text } = Typography

interface LeftPanelProps {
  onItemClick?: () => void
}

export function LeftPanel({ onItemClick }: LeftPanelProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const location = useLocation()
  const appTheme = useTheme()
  const { threads, currentThreadId, setCurrentThreadId, createThread } =
    useAppStore()
  const { leftPanelCollapsed, setLeftPanelCollapsed } = useSettingsStore()

  const handleNewChat = () => {
    const threadId = createThread(t('thread.newChat'))
    setCurrentThreadId(threadId)
    navigate('/')
    onItemClick?.()
  }

  const handleThreadClick = (threadId: string) => {
    setCurrentThreadId(threadId)
    navigate('/')
    onItemClick?.()
  }

  const navigationItems = [
    {
      key: 'new-chat',
      icon: <PlusOutlined />,
      label: t('navigation.newChat'),
      onClick: handleNewChat,
      type: 'primary',
    },
    {
      key: 'chats',
      icon: <MessageOutlined />,
      label: 'Chats',
      onClick: () => navigate('/'),
      active: location.pathname === '/',
    },
    {
      key: 'projects',
      icon: <FolderOutlined />,
      label: 'Projects',
      onClick: () => navigate('/projects'),
      active: location.pathname === '/projects',
    },
    {
      key: 'artifacts',
      icon: <BlockOutlined />,
      label: 'Artifacts',
      onClick: () => navigate('/artifacts'),
      active: location.pathname === '/artifacts',
    },
  ]

  const bottomNavigationItems = [
    {
      key: 'hub',
      icon: <AppstoreOutlined />,
      label: t('navigation.hub'),
      onClick: () => navigate('/hub'),
      active: location.pathname === '/hub',
    },
    {
      key: 'models',
      icon: <DatabaseOutlined />,
      label: 'Models',
      onClick: () => navigate('/models'),
      active: location.pathname === '/models',
    },
    {
      key: 'settings',
      icon: <SettingOutlined />,
      label: t('navigation.settings'),
      onClick: () => navigate('/settings'),
      active: location.pathname === '/settings',
    },
  ]

  return (
    <div
      className="h-screen flex flex-col p-3 transition-all duration-200"
      style={{
        height: '100%',
        backgroundColor: appTheme.sidebarBackground,
        color: appTheme.sidebarText,
      }}
    >
      {/* Collapse Toggle */}
      <div
        className={`mb-3 flex ${leftPanelCollapsed ? 'justify-center' : 'justify-end'}`}
      >
        <Tooltip
          title={leftPanelCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          placement="right"
        >
          <Button
            type="text"
            icon={
              leftPanelCollapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />
            }
            onClick={() => setLeftPanelCollapsed(!leftPanelCollapsed)}
            className="border-none px-2 py-1"
            style={{ color: appTheme.sidebarTextSecondary }}
          />
        </Tooltip>
      </div>

      {/* Navigation Items */}
      <div className="mb-4">
        {navigationItems.map(item => (
          <Tooltip
            key={item.key}
            title={leftPanelCollapsed ? item.label : ''}
            placement="right"
            mouseEnterDelay={0.5}
          >
            <Button
              type={item.type === 'primary' ? 'primary' : 'text'}
              onClick={() => {
                item.onClick()
                onItemClick?.()
              }}
              className={`mb-1 w-full ${leftPanelCollapsed ? 'justify-center' : 'justify-start text-left'} h-9 border-none rounded-lg overflow-hidden`}
              style={{
                backgroundColor:
                  item.type === 'primary'
                    ? appTheme.primary
                    : item.active
                      ? appTheme.sidebarItemActive
                      : 'transparent',
                color:
                  item.type === 'primary'
                    ? '#ffffff'
                    : item.active
                      ? appTheme.primary
                      : appTheme.sidebarText,
              }}
            >
              <div>{item.icon}</div>
              {!leftPanelCollapsed && (
                <div className={'flex-1 text-left pl-1'}>{item.label}</div>
              )}
            </Button>
          </Tooltip>
        ))}
      </div>

      {/* Recents Section */}
      {!leftPanelCollapsed && (
        <div className="mb-4">
          <Text
            className="text-xs font-semibold uppercase tracking-wider mb-2 block"
            style={{ color: appTheme.sidebarTextSecondary }}
          >
            Recents
          </Text>
        </div>
      )}

      {/* Recent Conversations */}
      <div className="flex-1 overflow-auto">
        {!leftPanelCollapsed ? (
          threads.length === 0 ? (
            <div
              className="py-8 px-4 text-center"
              style={{ color: appTheme.sidebarTextSecondary }}
            >
              <MessageOutlined
                className="text-2xl mb-2"
                style={{ color: appTheme.sidebarTextSecondary }}
              />
              <div className="text-sm">No conversations yet</div>
            </div>
          ) : (
            threads.slice(0, 20).map(thread => (
              <div
                key={thread.id}
                onClick={() => handleThreadClick(thread.id)}
                className={`px-3 py-2 mb-0.5 rounded-lg cursor-pointer text-sm overflow-hidden text-ellipsis whitespace-nowrap transition-all duration-200 border`}
                style={{
                  backgroundColor:
                    currentThreadId === thread.id
                      ? appTheme.sidebarItemActive
                      : 'transparent',
                  color:
                    currentThreadId === thread.id
                      ? appTheme.primary
                      : appTheme.sidebarText,
                  borderColor:
                    currentThreadId === thread.id
                      ? appTheme.primary + '4D'
                      : 'transparent',
                }}
                onMouseEnter={e => {
                  if (currentThreadId !== thread.id) {
                    e.currentTarget.style.backgroundColor =
                      'rgba(255,255,255,0.05)'
                  }
                }}
                onMouseLeave={e => {
                  if (currentThreadId !== thread.id) {
                    e.currentTarget.style.backgroundColor = 'transparent'
                  }
                }}
              >
                {thread.title}
              </div>
            ))
          )
        ) : (
          // Collapsed state - show dots for threads
          threads.slice(0, 10).map(thread => (
            <Tooltip
              key={thread.id}
              title={thread.title}
              placement="right"
              mouseEnterDelay={0.5}
            >
              <div
                onClick={() => handleThreadClick(thread.id)}
                className={`w-2 h-2 rounded-full mx-auto my-1.5 cursor-pointer transition-all duration-200`}
                style={{
                  backgroundColor:
                    currentThreadId === thread.id
                      ? appTheme.primary
                      : appTheme.sidebarTextSecondary,
                }}
                onMouseEnter={e => {
                  if (currentThreadId !== thread.id) {
                    e.currentTarget.style.backgroundColor =
                      'rgba(255,255,255,0.6)'
                  }
                }}
                onMouseLeave={e => {
                  if (currentThreadId !== thread.id) {
                    e.currentTarget.style.backgroundColor =
                      'rgba(255,255,255,0.3)'
                  }
                }}
              />
            </Tooltip>
          ))
        )}
      </div>

      {/* Bottom Navigation */}
      <div
        className="border-t pt-3 mt-3"
        style={{ borderColor: appTheme.sidebarBorder }}
      >
        {bottomNavigationItems.map(item => (
          <Tooltip
            key={item.key}
            title={leftPanelCollapsed ? item.label : ''}
            placement="right"
            mouseEnterDelay={0.5}
          >
            <Button
              type="text"
              onClick={() => {
                item.onClick()
                onItemClick?.()
              }}
              className={`mb-1 w-full ${leftPanelCollapsed ? 'justify-center' : 'justify-start text-left'} h-9 border-none rounded-lg overflow-hidden`}
              style={{
                backgroundColor: item.active
                  ? appTheme.sidebarItemActive
                  : 'transparent',
                color: item.active ? appTheme.primary : appTheme.sidebarText,
              }}
            >
              <div>{item.icon}</div>
              {!leftPanelCollapsed && (
                <div className="text-sm text-left flex-1 pl-1">
                  {item.label}
                </div>
              )}
            </Button>
          </Tooltip>
        ))}
      </div>
    </div>
  )
}