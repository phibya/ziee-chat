import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { Card, Typography, Button, Tag, Space, Alert, App, Spin } from 'antd'
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  ClearOutlined,
  DownloadOutlined,
  FileTextOutlined,
  ArrowDownOutlined,
} from '@ant-design/icons'
import { Stores } from '../../store'
import {
  connectToApiProxyLogs,
  disconnectFromApiProxyLogs,
  clearLogBuffer,
  downloadLogs,
  setAutoScroll,
} from '../../store/admin/apiProxyLogMonitor'

const { Text, Title } = Typography

export function ApiProxyLogMonitor() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const logContainerRef = useRef<HTMLDivElement>(null)

  // Store state
  const {
    logs,
    connected,
    connecting,
    error,
    logCount,
    lastUpdate,
    autoScroll,
  } = Stores.AdminApiProxyLogMonitor

  // Connect to log stream on component mount
  useEffect(() => {
    connectToApiProxyLogs().catch(console.error)

    // Cleanup on unmount
    return () => {
      disconnectFromApiProxyLogs()
    }
  }, [])

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (autoScroll && logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight
    }
  }, [logs, autoScroll])

  // Handle connection errors
  useEffect(() => {
    if (error) {
      message.error(`Log Monitor Error: ${error}`)
    }
  }, [error, message])

  const handleToggleConnection = async () => {
    try {
      if (connected) {
        disconnectFromApiProxyLogs()
        message.info('Disconnected from API proxy logs')
      } else {
        await connectToApiProxyLogs()
        message.success('Connected to API proxy logs')
      }
    } catch (error) {
      message.error('Failed to toggle log connection')
    }
  }

  const handleClearLogs = () => {
    clearLogBuffer()
    message.success('Log buffer cleared')
  }

  const handleDownloadLogs = async () => {
    try {
      await downloadLogs()
      message.success('Logs downloaded successfully')
    } catch (error) {
      message.error('Failed to download logs')
    }
  }

  const handleScrollToBottom = () => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight
      setAutoScroll(true)
    }
  }

  const handleScroll = () => {
    if (logContainerRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = logContainerRef.current
      const isAtBottom = scrollTop + clientHeight >= scrollHeight - 10
      if (autoScroll !== isAtBottom) {
        setAutoScroll(isAtBottom)
      }
    }
  }

  // Parse log line for better formatting
  const parseLogLine = (line: string) => {
    // Try to parse structured log format: timestamp level target message
    const timestampMatch = line.match(
      /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z?)/,
    )
    const levelMatch = line.match(/\s+(ERROR|WARN|INFO|DEBUG|TRACE)\s+/)
    const targetMatch = line.match(/api_proxy_server/)

    let timestamp = ''
    let level = ''
    let message = line
    let levelColor = 'default'

    if (timestampMatch) {
      timestamp = timestampMatch[1]
      message = line.substring(timestampMatch[0].length).trim()
    }

    if (levelMatch) {
      level = levelMatch[1]
      message = message.replace(levelMatch[0], ' ').trim()

      // Set level colors
      switch (level) {
        case 'ERROR':
          levelColor = 'red'
          break
        case 'WARN':
          levelColor = 'orange'
          break
        case 'INFO':
          levelColor = 'blue'
          break
        case 'DEBUG':
          levelColor = 'green'
          break
        case 'TRACE':
          levelColor = 'purple'
          break
        default:
          levelColor = 'default'
      }
    }

    return { timestamp, level, message, levelColor, isApiProxy: !!targetMatch }
  }

  return (
    <div className="p-4 h-screen flex flex-col">
      <div className="mb-4">
        <Title level={3} className="mb-2">
          <FileTextOutlined className="mr-2" />
          {t('apiProxyServer.logMonitor')}
        </Title>

        {/* Status Bar */}
        <Card size="small" className="mb-4">
          <div className="flex justify-between items-center flex-wrap gap-3">
            <Space>
              <Text strong>{t('apiProxyServer.connectionStatus')}:</Text>
              <Tag color={connected ? 'green' : 'red'}>
                {connected
                  ? t('apiProxyServer.connected')
                  : t('apiProxyServer.disconnected')}
              </Tag>
              {connecting && <Spin size="small" />}
            </Space>

            <Space>
              <Text type="secondary">
                {t('apiProxyServer.logCount')}: {logCount}
              </Text>
              {lastUpdate && (
                <Text type="secondary">
                  {t('apiProxyServer.lastUpdate')}:{' '}
                  {new Date(lastUpdate).toLocaleTimeString()}
                </Text>
              )}
              <Tag color={autoScroll ? 'blue' : 'default'}>
                Auto-scroll: {autoScroll ? 'ON' : 'OFF'}
              </Tag>
            </Space>
          </div>
        </Card>

        {/* Control Buttons */}
        <Space className="mb-4" wrap>
          <Button
            type="primary"
            icon={connected ? <PauseCircleOutlined /> : <PlayCircleOutlined />}
            onClick={handleToggleConnection}
            loading={connecting}
          >
            {connected
              ? t('apiProxyServer.disconnect')
              : t('apiProxyServer.connect')}
          </Button>

          <Button
            icon={<ClearOutlined />}
            onClick={handleClearLogs}
            disabled={logs.length === 0}
          >
            {t('apiProxyServer.clearLogs')}
          </Button>

          <Button
            icon={<DownloadOutlined />}
            onClick={handleDownloadLogs}
            disabled={logs.length === 0}
          >
            {t('apiProxyServer.downloadLogs')}
          </Button>

          <Button
            icon={<ArrowDownOutlined />}
            onClick={handleScrollToBottom}
            disabled={autoScroll}
          >
            {t('apiProxyServer.scrollToBottom')}
          </Button>
        </Space>

        {/* Connection Error */}
        {error && (
          <Alert
            message={t('apiProxyServer.connectionError')}
            description={error}
            type="error"
            showIcon
            closable
            className="mb-4"
            onClose={() => {
              // Clear error when user closes alert
              // Note: this would need to be implemented in the store
            }}
          />
        )}
      </div>

      {/* Log Display */}
      <Card
        className="flex-1 overflow-hidden"
        bodyStyle={{ height: '100%', padding: 0 }}
      >
        <div
          ref={logContainerRef}
          className="h-full p-4 overflow-y-auto font-mono text-sm"
          onScroll={handleScroll}
          style={{
            backgroundColor: '#1e1e1e',
            color: '#d4d4d4',
            lineHeight: '1.4',
          }}
        >
          {logs.length === 0 ? (
            <div className="text-center text-gray-500 mt-8">
              {connected ? (
                <div>
                  <FileTextOutlined className="text-2xl mb-2 block" />
                  {t('apiProxyServer.waitingForLogs')}
                </div>
              ) : (
                <div>
                  <PlayCircleOutlined className="text-2xl mb-2 block" />
                  {t('apiProxyServer.notConnected')}
                </div>
              )}
            </div>
          ) : (
            logs.map((line, index) => {
              const { timestamp, level, message, levelColor, isApiProxy } =
                parseLogLine(line)
              return (
                <div
                  key={index}
                  className={`mb-1 leading-tight ${isApiProxy ? 'opacity-100' : 'opacity-60'}`}
                >
                  {timestamp && (
                    <span className="text-gray-500 mr-3 text-xs">
                      {new Date(timestamp).toLocaleTimeString()}
                    </span>
                  )}
                  {level && (
                    <span className="mr-3">
                      <Tag
                        color={levelColor}
                        className="font-mono text-xs min-w-[50px] text-center"
                      >
                        {level}
                      </Tag>
                    </span>
                  )}
                  <span className="break-all">{message}</span>
                </div>
              )
            })
          )}
        </div>
      </Card>
    </div>
  )
}
