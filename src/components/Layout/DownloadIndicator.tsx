import { CloseOutlined, DownloadOutlined } from '@ant-design/icons'
import {
  App,
  Badge,
  Button,
  Flex,
  List,
  Popover,
  Progress,
  theme,
  Typography,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router-dom'
import { deleteModelDownload, Stores } from '../../store'
import type { DownloadInstance } from '../../types/api/modelDownloads'

const { Text } = Typography

export function DownloadIndicator() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const { token } = theme.useToken()

  // Get active downloads from store
  const downloads = Object.values(Stores.ModelDownload.downloads)

  // Don't render if no active downloads
  if (downloads.length === 0) {
    return null
  }

  const getProgressPercent = (download: DownloadInstance): number => {
    if (!download.progress_data?.current || !download.progress_data?.total) {
      return 0
    }
    return Math.round(
      (download.progress_data.current / download.progress_data.total) * 100,
    )
  }

  const formatSpeed = (speedBps?: number): string => {
    if (!speedBps) return ''

    const speedMBps = speedBps / (1024 * 1024)
    if (speedMBps >= 1) {
      return `${speedMBps.toFixed(1)} MB/s`
    }

    const speedKBps = speedBps / 1024
    return `${speedKBps.toFixed(1)} KB/s`
  }

  const formatETA = (etaSeconds?: number): string => {
    if (!etaSeconds) return ''

    const hours = Math.floor(etaSeconds / 3600)
    const minutes = Math.floor((etaSeconds % 3600) / 60)
    const seconds = Math.floor(etaSeconds % 60)

    if (hours > 0) {
      return `${hours}h ${minutes}m`
    } else if (minutes > 0) {
      return `${minutes}m ${seconds}s`
    } else {
      return `${seconds}s`
    }
  }

  // Get the first download for the button display
  const firstDownload = downloads[0]
  const firstDownloadPercent = getProgressPercent(firstDownload)

  // Create the popover content
  const popoverContent = (
    <div className="w-60">
      <Text strong className="block mb-2">
        {t('downloads.activeDownloads', {
          count: downloads.length,
          defaultValue: `${downloads.length} Active Downloads`,
        })}
      </Text>
      <List
        size="small"
        dataSource={downloads}
        className={'!p-0 !m-0'}
        renderItem={(download: DownloadInstance) => {
          const percent = getProgressPercent(download)
          const speed = formatSpeed(download.progress_data?.download_speed)
          const eta = formatETA(download.progress_data?.eta_seconds)

          const handleCloseDownload = async (e: React.MouseEvent) => {
            e.preventDefault()
            e.stopPropagation()
            try {
              await deleteModelDownload(download.id)
              message.success('Download removed successfully')
            } catch (error: any) {
              console.error('Failed to delete download:', error)
              message.error(`Failed to remove download: ${error.message}`)
            }
          }

          return (
            <List.Item className="flex flex-col !px-0 py-2">
              <Link
                to={`/settings/providers/${download.provider_id}`}
                className="w-full cursor-pointer"
              >
                <div className="w-full">
                  <Flex justify="space-between" align="center">
                    <Text
                      ellipsis
                      className={`text-sm font-medium`}
                      style={{ maxWidth: '60%' }}
                    >
                      {download.request_data.alias}
                    </Text>
                    <Flex align="center" gap={8}>
                      <Text className="text-sm font-medium">{percent}%</Text>
                      {download.status === 'completed' && (
                        <Button
                          type="text"
                          size="small"
                          icon={<CloseOutlined />}
                          onClick={handleCloseDownload}
                          title="Remove from list"
                          className="!p-0 !min-w-0 !w-4 !h-4"
                        />
                      )}
                    </Flex>
                  </Flex>

                  <Progress
                    percent={percent}
                    size="small"
                    status="active"
                    strokeColor={token.colorPrimary}
                    showInfo={false}
                    className={'mt-1 !leading-0'}
                  />

                  {(speed || eta) && (
                    <Flex
                      justify="space-between"
                      align="center"
                      className="mt-1"
                    >
                      <Text type="secondary" className="text-xs">
                        {speed}
                      </Text>
                      <Text type="secondary" className="text-xs">
                        {eta && `ETA: ${eta}`}
                      </Text>
                    </Flex>
                  )}
                </div>
              </Link>
            </List.Item>
          )
        }}
      />
    </div>
  )

  return (
    <Popover
      content={popoverContent}
      title={null}
      trigger="click"
      placement="rightTop"
    >
      <Button
        type="text"
        className={'w-full flex items-center justify-between px-3 !py-5'}
        style={{
          border: `1px solid ${token.colorBorder}`,
        }}
      >
        <Flex vertical className="w-full" gap={4}>
          <Flex align="center" justify="space-between">
            <Flex align="center" gap={8}>
              <Badge
                count={downloads.length}
                size="small"
                style={{ backgroundColor: token.colorPrimary }}
              >
                <DownloadOutlined />
              </Badge>
              <Text className="text-xs" type="secondary">
                Downloading
              </Text>
            </Flex>
            <Text className="text-xs" type="secondary">
              {firstDownloadPercent}%
            </Text>
          </Flex>

          <Progress
            percent={firstDownloadPercent}
            size="small"
            status="active"
            strokeColor={token.colorPrimary}
            showInfo={false}
            strokeWidth={3}
            className={'!leading-0'}
          />
        </Flex>
      </Button>
    </Popover>
  )
}
