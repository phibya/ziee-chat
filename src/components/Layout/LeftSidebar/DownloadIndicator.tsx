import { DownloadOutlined } from '@ant-design/icons'
import {
  Badge,
  Button,
  Divider,
  Flex,
  Popover,
  Progress,
  theme,
  Typography,
} from 'antd'
import { useTranslation } from 'react-i18next'
import { Stores, toggleSidebar } from '../../../store'
import type { DownloadInstance } from '../../../types/api/modelDownloads'
import { DownloadItem } from '../../Common/DownloadItem'
import { useWindowMinSize } from '../../hooks/useWindowMinSize.ts'
import { useState } from 'react'

const { Text } = Typography

export function DownloadIndicator() {
  const { t } = useTranslation()
  const { token } = theme.useToken()
  const windowMinSize = useWindowMinSize()
  const [isPopoverOpen, setIsPopoverOpen] = useState(false)

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

  // Find downloads with errors first, otherwise use the first download
  const errorDownload = downloads.find(d => d.status === 'failed')
  const displayDownload = errorDownload || downloads[0]
  const displayPercent =
    displayDownload.status === 'failed'
      ? 100
      : getProgressPercent(displayDownload)
  const hasError = displayDownload.status === 'failed'

  // Create the popover content
  const popoverContent = (
    <div className="w-60">
      <Text className="block mb-1">
        {t('downloads.activeDownloads', {
          count: downloads.length,
          defaultValue: `${downloads.length} Active Downloads`,
        })}
      </Text>
      <Flex vertical>
        {downloads.map((download, i) => (
          <>
            <DownloadItem
              key={download.id}
              download={download}
              mode="minimal"
              onClick={() => {
                console.log('click download item')
                if (windowMinSize.xs) {
                  toggleSidebar()
                }
                setIsPopoverOpen(false)
              }}
            />
            {i < downloads.length - 1 && (
              <Divider className={'!m-0 !leading-0'} />
            )}
          </>
        ))}
      </Flex>
    </div>
  )

  return (
    <Popover
      content={popoverContent}
      title={null}
      trigger="click"
      placement={windowMinSize.xs ? 'top' : 'rightTop'}
      open={isPopoverOpen}
      onOpenChange={open => setIsPopoverOpen(open)}
    >
      <Button
        type="text"
        className={'w-full flex items-center justify-between px-3 !py-5'}
        style={{
          border: `1px solid ${token.colorBorder}`,
        }}
        onClick={() => setIsPopoverOpen(!isPopoverOpen)}
      >
        <Flex vertical className="w-full" gap={4}>
          <Flex align="center" justify="space-between">
            <Flex align="center" gap={8}>
              <Badge
                count={downloads.length}
                size="small"
                style={{
                  backgroundColor: hasError
                    ? token.colorError
                    : token.colorPrimary,
                }}
              >
                <DownloadOutlined />
              </Badge>
              <Text className="text-xs" type="secondary">
                {hasError ? 'Error' : 'Downloading'}
              </Text>
            </Flex>
            <Text className="text-xs" type="secondary">
              {hasError ? 'Failed' : `${displayPercent}%`}
            </Text>
          </Flex>

          <Progress
            percent={displayPercent}
            size="small"
            status={hasError ? 'exception' : 'active'}
            strokeColor={hasError ? token.colorError : token.colorPrimary}
            showInfo={false}
            strokeWidth={3}
            className={'!leading-0'}
          />
        </Flex>
      </Button>
    </Popover>
  )
}
