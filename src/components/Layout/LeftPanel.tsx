import { useLocation, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Button, Divider, Dropdown, Tooltip, Typography } from 'antd'
import {
  AppstoreOutlined,
  BlockOutlined,
  DatabaseOutlined,
  FolderOutlined,
  HistoryOutlined,
  LogoutOutlined,
  MenuFoldOutlined,
  PlusOutlined,
  RobotOutlined,
  SettingOutlined,
  UserOutlined,
} from '@ant-design/icons'
import { useAuthStore, useUISettings } from '../../store'
import { RecentConversations } from '../Chat/RecentConversations.tsx'

interface LeftPanelProps {
  onItemClick?: () => void
  isMobile?: boolean
  mobileOverlayOpen?: boolean
  setMobileOverlayOpen?: (open: boolean) => void
}

export function LeftPanel({
  onItemClick,
  isMobile,
  mobileOverlayOpen,
  setMobileOverlayOpen,
}: LeftPanelProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const location = useLocation()
  const { leftPanelCollapsed, setLeftPanelCollapsed } = useUISettings()
  const { user, logout, isDesktop } = useAuthStore()

  const handleNewChat = () => {
    // Navigate to chat without a conversation ID to start a new conversation
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
      key: 'chat-history',
      icon: <HistoryOutlined />,
      label: 'Chats',
      onClick: () => navigate('/chat-history'),
      active: location.pathname === '/chat-history',
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
      key: 'assistants',
      icon: <RobotOutlined />,
      label: 'Assistants',
      onClick: () => navigate('/assistants'),
      active: location.pathname === '/assistants',
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
    <div className="h-screen flex flex-col p-1 min-w-fit">
      {/* Collapse Toggle - Only show when panel is open */}
      <div className="flex justify-end">
        <Tooltip
          title={isMobile ? 'Close sidebar' : 'Collapse sidebar'}
          placement="right"
        >
          <Button
            type="text"
            icon={<MenuFoldOutlined />}
            onClick={() => {
              if (isMobile && setMobileOverlayOpen) {
                setMobileOverlayOpen(false)
              } else {
                setLeftPanelCollapsed(true)
              }
            }}
          />
        </Tooltip>
      </div>

      {/* Navigation Items */}
      <div className={'flex-col flex'}>
        {navigationItems.map(item => (
          <Button
            type={item.type === 'primary' ? 'primary' : 'text'}
            onClick={() => {
              item.onClick()
              onItemClick?.()
            }}
          >
            <div>{item.icon}</div>
            <div className={'flex-1 text-left pl-1'}>{item.label}</div>
          </Button>
        ))}
      </div>

      <Divider size={'small'} />

      {/* Recents Section */}
      <Typography.Text type="secondary" className={'p-2 pt-1'}>
        Recents
      </Typography.Text>

      {/*/!* Recent Conversations *!/*/}
      <RecentConversations
        collapsed={leftPanelCollapsed}
        isMobile={isMobile}
        mobileOverlayOpen={mobileOverlayOpen}
        onConversationClick={onItemClick}
      />

      <Divider size={'small'} />

      {/* Bottom Navigation */}
      <div className={'flex-col flex'}>
        {bottomNavigationItems.map(item => (
          <Tooltip
            key={item.key}
            title={
              (isMobile ? !mobileOverlayOpen : leftPanelCollapsed)
                ? item.label
                : ''
            }
            placement="right"
            mouseEnterDelay={0.5}
          >
            <Button
              type="text"
              onClick={() => {
                item.onClick()
                onItemClick?.()
              }}
              className={`${leftPanelCollapsed ? 'justify-center' : 'justify-start'}`}
              block
            >
              <div>{item.icon}</div>
              <div className="flex-1 text-left pl-1">{item.label}</div>
            </Button>
          </Tooltip>
        ))}
      </div>

      {/* User Profile Section */}
      {user && !isDesktop && (
        <>
          <Divider size={'small'} />
          <Dropdown
            menu={{
              items: [
                {
                  key: 'profile',
                  icon: <UserOutlined />,
                  label: 'Profile',
                  onClick: () => {
                    // Navigate to profile page or open profile modal
                    console.log('Profile clicked')
                  },
                },
                {
                  key: 'logout',
                  icon: <LogoutOutlined />,
                  label: 'Logout',
                  onClick: async () => {
                    await logout()
                    onItemClick?.()
                  },
                },
              ],
            }}
            placement="topLeft"
            trigger={['click']}
          >
            <Button type="text" className="flex items-start text-left w-full">
              <div>
                <UserOutlined />
              </div>
              <div className="flex-1 text-left pl-1">
                <Typography.Text strong ellipsis>
                  {user.username}
                </Typography.Text>
              </div>
            </Button>
          </Dropdown>
        </>
      )}
    </div>
  )
}
