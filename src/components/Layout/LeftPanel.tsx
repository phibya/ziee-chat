import { useLocation, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Divider,
  Dropdown,
  Menu,
  theme,
  Tooltip,
  Typography,
} from 'antd'
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
  const { token } = theme.useToken()

  const handleNewChat = () => {
    // Navigate to chat without a conversation ID to start a new conversation
    navigate('/')
    onItemClick?.()
  }

  const getSelectedKeys = () => {
    if (location.pathname === '/chat-history') return ['chat-history']
    if (location.pathname === '/projects') return ['projects']
    if (location.pathname === '/artifacts') return ['artifacts']
    if (location.pathname === '/hub') return ['hub']
    if (location.pathname === '/assistants') return ['assistants']
    if (location.pathname === '/models') return ['models']
    if (location.pathname === '/settings') return ['settings']
    return []
  }

  return (
    <div
      className="h-dvh flex flex-col px-1 min-w-fit"
      style={{
        borderRight: `1px solid ${token.colorBorderSecondary}`,
      }}
    >
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

      {/* New Chat Button */}
      <Button
        type="primary"
        onClick={() => {
          handleNewChat()
          onItemClick?.()
        }}
        className={'flex text-left mb-2 mx-1'}
      >
        <div className={'text-left w-full flex gap-2'}>
          <PlusOutlined />
          <div>{t('navigation.newChat')}</div>
        </div>
      </Button>

      {/* Navigation Items */}
      <Menu
        selectedKeys={getSelectedKeys()}
        items={[
          {
            key: 'chat-history',
            icon: <HistoryOutlined />,
            label: 'Chats',
            onClick: () => {
              navigate('/chat-history')
              onItemClick?.()
            },
          },
          {
            key: 'projects',
            icon: <FolderOutlined />,
            label: 'Projects',
            onClick: () => {
              navigate('/projects')
              onItemClick?.()
            },
          },
          {
            key: 'artifacts',
            icon: <BlockOutlined />,
            label: 'Artifacts',
            onClick: () => {
              navigate('/artifacts')
              onItemClick?.()
            },
          },
        ]}
        style={{ border: 'none' }}
      />

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
      <Menu
        selectedKeys={getSelectedKeys()}
        items={[
          {
            key: 'hub',
            icon: <AppstoreOutlined />,
            label: t('navigation.hub'),
            onClick: () => {
              navigate('/hub')
              onItemClick?.()
            },
          },
          {
            key: 'assistants',
            icon: <RobotOutlined />,
            label: 'Assistants',
            onClick: () => {
              navigate('/assistants')
              onItemClick?.()
            },
          },
          {
            key: 'models',
            icon: <DatabaseOutlined />,
            label: 'Models',
            onClick: () => {
              navigate('/models')
              onItemClick?.()
            },
          },
          {
            key: 'settings',
            icon: <SettingOutlined />,
            label: t('navigation.settings'),
            onClick: () => {
              navigate('/settings')
              onItemClick?.()
            },
          },
        ]}
        style={{ border: 'none' }}
      />

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
