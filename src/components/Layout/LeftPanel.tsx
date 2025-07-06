import {useNavigate, useLocation} from 'react-router-dom'
import {useTranslation} from 'react-i18next'
import {Button, Typography} from 'antd'
import {
    PlusOutlined,
    MessageOutlined,
    FolderOutlined,
    BlockOutlined,
    SettingOutlined,
    AppstoreOutlined,
    DatabaseOutlined
} from '@ant-design/icons'
import {useAppStore} from '../../store'

const {Text} = Typography

interface LeftPanelProps {
    onItemClick?: () => void
}

export function LeftPanel({onItemClick}: LeftPanelProps) {
    const {t} = useTranslation()
    const navigate = useNavigate()
    const location = useLocation()
    const {
        threads,
        currentThreadId,
        setCurrentThreadId,
        createThread
    } = useAppStore()

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
            icon: <PlusOutlined/>,
            label: t('navigation.newChat'),
            onClick: handleNewChat,
            type: 'primary'
        },
        {
            key: 'chats',
            icon: <MessageOutlined/>,
            label: 'Chats',
            onClick: () => navigate('/'),
            active: location.pathname === '/'
        },
        {
            key: 'projects',
            icon: <FolderOutlined/>,
            label: 'Projects',
            onClick: () => navigate('/projects'),
            active: location.pathname === '/projects'
        },
        {
            key: 'artifacts',
            icon: <BlockOutlined/>,
            label: 'Artifacts',
            onClick: () => navigate('/artifacts'),
            active: location.pathname === '/artifacts'
        }
    ]

    const bottomNavigationItems = [
        {
            key: 'hub',
            icon: <AppstoreOutlined/>,
            label: t('navigation.hub'),
            onClick: () => navigate('/hub'),
            active: location.pathname === '/hub'
        },
        {
            key: 'models',
            icon: <DatabaseOutlined/>,
            label: 'Models',
            onClick: () => navigate('/models'),
            active: location.pathname === '/models'
        },
        {
            key: 'settings',
            icon: <SettingOutlined/>,
            label: t('navigation.settings'),
            onClick: () => navigate('/settings'),
            active: location.pathname === '/settings'
        }
    ]

    return (
        <div style={{
            height: '100vh',
            display: 'flex',
            flexDirection: 'column',
            padding: '12px',
            backgroundColor: 'rgb(20, 20, 20)',
            color: 'white'
        }}>
            {/* Navigation Items */}
            <div style={{marginBottom: '16px'}}>
                {navigationItems.map((item) => (
                    <Button
                        key={item.key}
                        type={item.type === 'primary' ? 'primary' : 'text'}
                        icon={item.icon}
                        block
                        onClick={() => {
                            item.onClick()
                            onItemClick?.()
                        }}
                        style={{
                            marginBottom: '4px',
                            justifyContent: 'flex-start',
                            height: '36px',
                            color: item.active ? '#ff8c00' : (item.type === 'primary' ? undefined : 'rgba(255,255,255,0.8)'),
                            backgroundColor: item.type === 'primary' ? '#ff8c00' : (item.active ? 'rgba(255,140,0,0.1)' : 'transparent'),
                            border: 'none',
                            borderRadius: '8px'
                        }}
                    >
                        <span style={{marginLeft: '8px', fontSize: '14px'}}>{item.label}</span>
                    </Button>
                ))}
            </div>

            {/* Recents Section */}
            <div style={{marginBottom: '16px'}}>
                <Text style={{
                    fontSize: '12px',
                    fontWeight: 600,
                    color: 'rgba(255,255,255,0.6)',
                    textTransform: 'uppercase',
                    letterSpacing: '0.5px',
                    marginBottom: '8px',
                    display: 'block'
                }}>
                    Recents
                </Text>
            </div>

            {/* Recent Conversations */}
            <div style={{flex: 1, overflow: 'auto'}}>
                {threads.length === 0 ? (
                    <div style={{
                        padding: '32px 16px',
                        textAlign: 'center',
                        color: 'rgba(255,255,255,0.5)'
                    }}>
                        <MessageOutlined style={{fontSize: '24px', marginBottom: '8px'}}/>
                        <div style={{fontSize: '14px'}}>No conversations yet</div>
                    </div>
                ) : (
                    threads.slice(0, 20).map((thread) => (
                        <div
                            key={thread.id}
                            onClick={() => handleThreadClick(thread.id)}
                            style={{
                                padding: '8px 12px',
                                marginBottom: '2px',
                                borderRadius: '8px',
                                cursor: 'pointer',
                                backgroundColor: currentThreadId === thread.id ? 'rgba(255,140,0,0.1)' : 'transparent',
                                color: currentThreadId === thread.id ? '#ff8c00' : 'rgba(255,255,255,0.8)',
                                fontSize: '14px',
                                overflow: 'hidden',
                                textOverflow: 'ellipsis',
                                whiteSpace: 'nowrap',
                                transition: 'all 0.2s ease',
                                border: currentThreadId === thread.id ? '1px solid rgba(255,140,0,0.3)' : '1px solid transparent'
                            }}
                            onMouseEnter={(e) => {
                                if (currentThreadId !== thread.id) {
                                    e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.05)'
                                }
                            }}
                            onMouseLeave={(e) => {
                                if (currentThreadId !== thread.id) {
                                    e.currentTarget.style.backgroundColor = 'transparent'
                                }
                            }}
                        >
                            {thread.title}
                        </div>
                    ))
                )}
            </div>

            {/* Bottom Navigation */}
            <div style={{
                borderTop: '1px solid rgba(255,255,255,0.1)',
                paddingTop: '12px',
                marginTop: '12px'
            }}>
                {bottomNavigationItems.map((item) => (
                    <Button
                        key={item.key}
                        type="text"
                        icon={item.icon}
                        block
                        onClick={() => {
                            item.onClick()
                            onItemClick?.()
                        }}
                        style={{
                            marginBottom: '4px',
                            justifyContent: 'flex-start',
                            height: '36px',
                            color: item.active ? '#ff8c00' : 'rgba(255,255,255,0.8)',
                            backgroundColor: item.active ? 'rgba(255,140,0,0.1)' : 'transparent',
                            border: 'none',
                            borderRadius: '8px'
                        }}
                    >
                        <span style={{marginLeft: '8px', fontSize: '14px'}}>{item.label}</span>
                    </Button>
                ))}
            </div>
        </div>
    )
}