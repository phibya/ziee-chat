import { Link, useLocation, useNavigate } from 'react-router-dom'
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
  FolderOutlined,
  HistoryOutlined,
  LogoutOutlined,
  MenuFoldOutlined,
  PlusOutlined,
  RobotOutlined,
  SettingOutlined,
  UserOutlined,
} from '@ant-design/icons'
import {
  closeMobileOverlay,
  logoutUser,
  setUILeftPanelCollapsed,
  Stores,
} from '../../store'
import { RecentConversations } from '../Chat/RecentConversations.tsx'
import { DownloadIndicator } from './DownloadIndicator'

export function LeftPanel() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const location = useLocation()
  const { user, isDesktop } = Stores.Auth
  const { isMobile } = Stores.UI.Layout
  const { token } = theme.useToken()

  const handleItemClick = () => {
    if (isMobile) {
      closeMobileOverlay()
    }
  }

  const handleNewChat = () => {
    // Navigate to chat without a conversation ID to start a new conversation
    navigate('/')
    handleItemClick()
  }

  const getSelectedKeys = () => {
    if (location.pathname === '/chat-history') return ['chat-history']
    if (location.pathname === '/projects') return ['projects']
    if (location.pathname === '/artifacts') return ['artifacts']
    if (location.pathname === '/hub') return ['hub']
    if (location.pathname === '/assistants') return ['assistants']
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
              if (isMobile) {
                closeMobileOverlay()
              } else {
                setUILeftPanelCollapsed(true)
              }
            }}
          />
        </Tooltip>
      </div>

      {/* New Chat Button */}
      <Button
        type="primary"
        onClick={handleNewChat}
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
            label: <Link to="/chat-history">{t('navigation.chats')}</Link>,
            onClick: () => {
              navigate('/chat-history')
              handleItemClick()
            },
          },
          {
            key: 'projects',
            icon: <FolderOutlined />,
            label: <Link to="/projects">{t('navigation.projects')}</Link>,
            onClick: () => {
              navigate('/projects')
              handleItemClick()
            },
          },
          {
            key: 'artifacts',
            icon: <BlockOutlined />,
            label: <Link to="/artifacts">{t('navigation.artifacts')}</Link>,
            onClick: () => {
              navigate('/artifacts')
              handleItemClick()
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
      <RecentConversations />

      <Divider size={'small'} />

      {/* Bottom Navigation */}
      <Menu
        selectedKeys={getSelectedKeys()}
        items={[
          {
            key: 'hub',
            icon: <AppstoreOutlined />,
            label: <Link to="/hub">{t('navigation.hub')}</Link>,
            onClick: () => {
              navigate('/hub')
              handleItemClick()
            },
          },
          {
            key: 'assistants',
            icon: <RobotOutlined />,
            label: <Link to="/assistants">{t('navigation.assistants')}</Link>,
            onClick: () => {
              navigate('/assistants')
              handleItemClick()
            },
          },
          {
            key: 'settings',
            icon: <SettingOutlined />,
            label: <Link to="/settings">{t('navigation.settings')}</Link>,
            onClick: () => {
              navigate('/settings')
              handleItemClick()
            },
          },
        ]}
        style={{ border: 'none' }}
      />

      {/* Download Indicator */}
      <DownloadIndicator />

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
                  label: t('navigation.profile'),
                  onClick: () => {
                    // Navigate to profile page or open profile modal
                    console.log('Profile clicked')
                  },
                },
                {
                  key: 'logout',
                  icon: <LogoutOutlined />,
                  label: t('navigation.logout'),
                  onClick: async () => {
                    await logoutUser()
                    handleItemClick()
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
