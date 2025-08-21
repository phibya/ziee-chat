import { useTranslation } from 'react-i18next'
import { Alert, App, Button, Card, Flex, Typography } from 'antd'
import {
  PlayCircleOutlined,
  ReloadOutlined,
  StopOutlined,
} from '@ant-design/icons'
import { Stores } from '../../../../store'
import { isTauriView } from '../../../../api/core.ts'
import {
  startApiProxyServer,
  stopApiProxyServer,
} from '../../../../store/admin/apiProxyServer.ts'
import { ApiClient } from '../../../../api/client'
import { MdOutlineMonitorHeart } from 'react-icons/md'

const { Text } = Typography

export function ServerControlCard() {
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Store data
  const { config, status, loadingStatus, loadingModels, loadingHosts, models } =
    Stores.AdminApiProxyServer

  // Check if server is properly configured
  const isConfigured = Boolean(
    config?.address && config?.port && config?.prefix,
  )

  const handleStart = async () => {
    try {
      await startApiProxyServer()
      message.success(t('apiProxyServer.serverStarted'))
    } catch (_error) {
      message.error(t('apiProxyServer.serverStartError'))
    }
  }

  const handleStop = async () => {
    try {
      await stopApiProxyServer()
      message.success(t('apiProxyServer.serverStopped'))
    } catch (_error) {
      message.error(t('apiProxyServer.serverStopError'))
    }
  }

  const handleReloadModels = async () => {
    try {
      await ApiClient.Admin.reloadApiProxyServerModels()
      message.success(t('apiProxyServer.modelsReloaded'))
    } catch (_error) {
      message.error(t('apiProxyServer.modelsReloadError'))
    }
  }

  const handleReloadTrustedHosts = async () => {
    try {
      await ApiClient.Admin.reloadApiProxyServerTrustedHosts()
      message.success(t('apiProxyServer.trustedHostsReloaded'))
    } catch (_error) {
      message.error(t('apiProxyServer.trustedHostsReloadError'))
    }
  }

  const handleOpenLogMonitor = async () => {
    if (isTauriView) {
      try {
        const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')
        const existingWindow = await WebviewWindow.getByLabel('log-monitor')
        if (existingWindow) {
          await existingWindow.setFocus()
        } else {
          new WebviewWindow('log-monitor', {
            url: '/api-proxy-log-monitor',
            title: t('apiProxyServer.logMonitor'),
            width: 800,
            height: 600,
          })
        }
      } catch (error) {
        console.error('Failed to open log monitor window:', error)
        message.error(t('apiProxyServer.logMonitorError'))
      }
    } else {
      // Use browser popup for web app
      const popup = window.open(
        window.location.origin + '/api-proxy-log-monitor',
        'api-proxy-log-monitor', // Using same name will focus existing popup
        'width=800,height=600,scrollbars=yes,resizable=yes,menubar=no,toolbar=no',
      )
      if (popup) {
        popup.focus()
      } else {
        message.error('Please allow popups for this website')
      }
    }
  }

  // Modified title with Log Monitor button (following HardwareSettings pattern)
  const titleWithButton = (
    <div className="flex items-center justify-between w-full">
      <span>{t('apiProxyServer.serverControl')}</span>
      <Button icon={<MdOutlineMonitorHeart />} onClick={handleOpenLogMonitor}>
        {t('apiProxyServer.logMonitor')}
      </Button>
    </div>
  )

  return (
    <Card title={titleWithButton}>
      <div className="flex flex-col gap-3 flex-wrap w-full">
        {/* Status Display */}
        <Flex justify="space-between" align="center">
          <Text strong>{t('apiProxyServer.status')}:</Text>
          <Text type={status?.running ? 'success' : 'secondary'}>
            {status?.running
              ? t('apiProxyServer.running')
              : t('apiProxyServer.stopped')}
          </Text>
        </Flex>

        {/* Server URL */}
        {status?.running && config && (
          <Flex justify="space-between" align="center">
            <Text strong>{t('apiProxyServer.serverUrl')}:</Text>
            <div className="flex gap-3 flex-wrap">
              <Text code copyable>
                http://{config.address}:{config.port}
                {config.prefix}
              </Text>
            </div>
          </Flex>
        )}

        {/* Active Models Count */}
        {status?.running && (
          <Flex justify="space-between" align="center">
            <Text strong>{t('apiProxyServer.activeModels')}:</Text>
            <Text>{status.active_models || 0}</Text>
          </Flex>
        )}

        {/* Control Buttons */}
        <div className="flex gap-3 flex-wrap">
          {!status?.running ? (
            <Button
              type="primary"
              icon={<PlayCircleOutlined />}
              onClick={handleStart}
              loading={loadingStatus}
              disabled={!isConfigured}
            >
              {t('apiProxyServer.startServer')}
            </Button>
          ) : (
            <Button
              danger
              icon={<StopOutlined />}
              onClick={handleStop}
              loading={loadingStatus}
            >
              {t('apiProxyServer.stopServer')}
            </Button>
          )}

          {/* Reload Buttons - Only show when server is running */}
          {status?.running && (
            <>
              <Button
                icon={<ReloadOutlined />}
                onClick={handleReloadModels}
                loading={loadingModels}
              >
                {t('apiProxyServer.reloadModels')}
              </Button>
              <Button
                icon={<ReloadOutlined />}
                onClick={handleReloadTrustedHosts}
                loading={loadingHosts}
              >
                {t('apiProxyServer.reloadTrustedHosts')}
              </Button>
            </>
          )}
        </div>

        {/* Configuration Warnings */}
        {!isConfigured && (
          <Alert
            message={t('apiProxyServer.configurationIncomplete')}
            description={t('apiProxyServer.configurationIncompleteDesc')}
            type="warning"
            showIcon
          />
        )}

        {/* No Models Warning when server is running */}
        {status?.running && models.length === 0 && (
          <Alert
            message={t('apiProxyServer.noModelsRunning')}
            description={t('apiProxyServer.noModelsRunningDesc')}
            type="info"
            showIcon
          />
        )}
      </div>
    </Card>
  )
}
