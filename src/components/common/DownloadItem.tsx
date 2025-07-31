import { CloseOutlined } from '@ant-design/icons'
import { App, Button, Flex, Progress, theme, Typography } from 'antd'
import {
  cancelModelDownload,
  deleteModelDownload,
  openViewDownloadModal,
} from '../../store'
import type { DownloadInstance } from '../../types'
import {
  formatBytes,
  formatETA,
  formatSpeed,
} from '../../utils/downloadUtils.ts'
import { Link } from 'react-router-dom'

const { Text } = Typography

interface DownloadItemProps {
  download: DownloadInstance
  mode?: 'full' | 'compact' | 'minimal'
}

export function DownloadItem({ download, mode = 'full' }: DownloadItemProps) {
  const { message } = App.useApp()
  const { token } = theme.useToken()

  const percent = download.progress_data
    ? Math.round(
        (download.progress_data.current / download.progress_data.total) * 100,
      )
    : 0

  const speed = formatSpeed(download.progress_data?.download_speed)
  const eta = formatETA(download.progress_data?.eta_seconds)

  const handleCloseDownload = async (downloadId: string) => {
    try {
      await deleteModelDownload(downloadId)
      message.success('Download removed successfully')
    } catch (error: any) {
      console.error('Failed to delete download:', error)
      message.error(`Failed to remove download: ${error.message}`)
    }
  }

  const handleCancelDownload = async () => {
    try {
      await cancelModelDownload(download.id)
      message.success('Download cancelled successfully')
    } catch (error: any) {
      console.error('Failed to cancel download:', error)
      message.error(`Failed to cancel download: ${error.message}`)
    }
  }

  // Minimal mode for DownloadIndicator (sidebar popover)
  if (mode === 'minimal') {
    return (
      <div className="py-2">
        <Flex justify="space-between" align="center" className="mb-1">
          <Link
            to={`/settings/providers/${download.provider_id}`}
            className="text-xs truncate flex-1 pr-2"
            onClick={() => {
              openViewDownloadModal(download.id)
            }}
          >
            {download.request_data.alias}
          </Link>
          <Text type="secondary" className="text-xs">
            {percent}%
          </Text>
        </Flex>
        <Progress
          percent={percent}
          size="small"
          status="active"
          strokeColor={token.colorPrimary}
          showInfo={false}
          strokeWidth={4}
        />
        {(speed || eta) && (
          <Flex justify="space-between" align="center" className="mt-1">
            <Text type="secondary" className="text-xs">
              {speed || ''}
            </Text>
            <Text type="secondary" className="text-xs">
              {eta ? `ETA: ${eta}` : ''}
            </Text>
          </Flex>
        )}
      </div>
    )
  }

  // Compact mode for ModelCard
  if (mode === 'compact') {
    return (
      <div className="rounded-lg">
        <Flex justify="space-between" align="center" className="mb-2">
          <Link
            to={`/settings/providers/${download.provider_id}`}
            className="text-xs truncate flex-1 pr-2"
            onClick={() => {
              openViewDownloadModal(download.id)
            }}
          >
            {download.request_data.alias}
          </Link>
          <Flex className="gap-1">
            {!['completed', 'failed', 'cancelled'].includes(
              download.status,
            ) && (
              <Button
                type="text"
                size="small"
                danger
                onClick={handleCancelDownload}
              >
                Cancel
              </Button>
            )}
          </Flex>
        </Flex>
        <Progress
          percent={percent}
          status="active"
          strokeColor={token.colorPrimary}
          size="small"
          className="mb-2"
        />
        <Flex justify="space-between" align="center">
          <Text type="secondary" className="text-xs">
            {download.progress_data
              ? `${formatBytes(download.progress_data.current)} / ${formatBytes(download.progress_data.total)}`
              : '0 B / 0 B'}
          </Text>
          <Flex className="gap-2">
            {speed && (
              <Text type="secondary" className="text-xs">
                {speed}
              </Text>
            )}
            {eta && (
              <Text type="secondary" className="text-xs">
                {eta}
              </Text>
            )}
          </Flex>
        </Flex>
      </div>
    )
  }

  // Full mode for LocalProviderSettings (default)
  return (
    <div className="rounded-lg">
      <Flex justify="space-between" align="flex-start" className="mb-3">
        <div className="flex-1">
          <div className="text-base font-medium mb-1">
            {download.request_data.alias}
          </div>
          <Text type="secondary" className="text-xs">
            {download.progress_data?.message || 'Preparing download...'}
          </Text>
        </div>
        <Flex className="gap-2">
          <Button
            type="text"
            size="small"
            onClick={() => openViewDownloadModal(download.id)}
          >
            View Details
          </Button>
          {['completed', 'failed', 'cancelled'].includes(download.status) ? (
            <Button
              type="text"
              size="small"
              icon={<CloseOutlined />}
              onClick={() => handleCloseDownload(download.id)}
              title="Remove from list"
            >
              Close
            </Button>
          ) : (
            <Button
              type="text"
              danger
              size="small"
              onClick={handleCancelDownload}
            >
              Cancel
            </Button>
          )}
        </Flex>
      </Flex>
      <Progress
        percent={percent}
        status="active"
        strokeColor={token.colorPrimary}
        size="small"
        className="mb-2"
      />
      <Flex justify="space-between" align="center">
        <Text type="secondary" className="text-xs">
          {download.progress_data
            ? `${formatBytes(download.progress_data.current)} / ${formatBytes(download.progress_data.total)}`
            : '0 B / 0 B'}
        </Text>
        <Flex className={'gap-2'}>
          {speed && (
            <Text type="secondary" className="text-xs">
              Speed: {speed}
            </Text>
          )}
          {eta && (
            <Text type="secondary" className="text-xs">
              ETA: {eta}
            </Text>
          )}
        </Flex>
      </Flex>
    </div>
  )
}
