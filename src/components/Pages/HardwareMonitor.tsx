import { Alert, App, Button, Card, Progress, Spin, Tag, Typography } from 'antd'
import { useEffect } from 'react'
import {
  disconnectHardwareUsage,
  Stores,
  subscribeToHardwareUsage,
} from '../../store'
import { formatBytes } from '../../utils/formatBytes'
import { useSetBackgroundColor } from '../hooks/useSetBackgroundColor.ts'

const { Text } = Typography

export function HardwareMonitor() {
  const { message } = App.useApp()
  useSetBackgroundColor()

  // Hardware store state
  const {
    hardwareInfo,
    hardwareLoading,
    hardwareError,
    currentUsage,
    usageLoading,
    usageError,
    sseConnected,
    sseError,
  } = Stores.AdminHardware

  // Initialize hardware monitoring on component mount
  useEffect(() => {
    // Load hardware info first, then start monitoring
    subscribeToHardwareUsage().catch(console.error)

    // Cleanup on component unmount
    return () => {
      disconnectHardwareUsage()
    }
  }, [])

  // Show errors
  useEffect(() => {
    if (hardwareError) {
      message.error(`Hardware Error: ${hardwareError}`)
    }
    if (usageError) {
      message.error(`Usage Monitoring Error: ${usageError}`)
    }
    if (sseError) {
      message.error(`Connection Error: ${sseError}`)
    }
  }, [hardwareError, usageError, sseError, message])

  const handleManualConnect = async () => {
    try {
      await subscribeToHardwareUsage()
      message.success('Connecting to hardware monitoring...')
    } catch (_error) {
      message.error('Failed to connect to hardware monitoring')
    }
  }

  const renderConnectionStatus = () => (
    <Card
      style={{
        display: sseConnected ? 'none' : 'block',
      }}
    >
      <div className={'flex flex-wrap justify-between gap-3'}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Text strong>Real-time Monitoring:</Text>
            <Tag color={sseConnected ? 'green' : 'red'}>
              {sseConnected ? 'Connected' : 'Disconnected'}
            </Tag>
            {usageLoading && (
              <div className="flex items-center gap-2">
                <Spin />
                <Text type="secondary">Connecting...</Text>
              </div>
            )}
          </div>
          {!sseConnected && !usageLoading && (
            <Button type="primary" onClick={handleManualConnect}>
              Connect
            </Button>
          )}
        </div>
        {currentUsage && (
          <Text type="secondary" style={{ fontSize: '11px' }}>
            Last update: {new Date(currentUsage.timestamp).toLocaleTimeString()}
          </Text>
        )}
      </div>
    </Card>
  )

  const renderCPUUsage = () => {
    if (!currentUsage) return null

    return (
      <Card title="CPU Usage">
        <Progress
          percent={currentUsage.cpu.usage_percentage}
          status={
            currentUsage.cpu.usage_percentage > 90 ? 'exception' : 'active'
          }
          format={percent => `${percent != null ? percent.toFixed(1) : '0.0'}%`}
        />
        <div className="flex gap-3 mt-2">
          {currentUsage.cpu.temperature && (
            <Text type="secondary" style={{ fontSize: '12px' }}>
              Temperature: {currentUsage.cpu.temperature}°C
            </Text>
          )}
          {currentUsage.cpu.frequency && (
            <Text type="secondary" style={{ fontSize: '12px' }}>
              Frequency: {currentUsage.cpu.frequency} MHz
            </Text>
          )}
        </div>
      </Card>
    )
  }

  const renderMemoryUsage = () => {
    if (!currentUsage) return null

    return (
      <Card title="Memory Usage">
        <Progress
          percent={currentUsage.memory.usage_percentage}
          status={
            currentUsage.memory.usage_percentage > 90 ? 'exception' : 'active'
          }
          format={percent => `${percent != null ? percent.toFixed(1) : '0.0'}%`}
        />
        <div className="flex gap-3 mt-2">
          <Text type="secondary" style={{ fontSize: '12px' }}>
            Used: {formatBytes(currentUsage.memory.used_ram)}
          </Text>
          <Text type="secondary" style={{ fontSize: '12px' }}>
            Available: {formatBytes(currentUsage.memory.available_ram)}
          </Text>
        </div>
      </Card>
    )
  }

  if (hardwareLoading) {
    return (
      <div className="p-3">
        <div style={{ textAlign: 'center', padding: '50px' }}>
          <Spin size="large" />
          <Text
            type="secondary"
            style={{ display: 'block', marginTop: '16px' }}
          >
            Loading hardware monitor...
          </Text>
        </div>
      </div>
    )
  }

  if (hardwareError && !hardwareInfo) {
    return (
      <div className="p-3">
        <Alert
          message="Hardware Monitor Unavailable"
          description={hardwareError}
          type="error"
          showIcon
        />
      </div>
    )
  }

  return (
    <div className="p-3 max-w-4xl mx-auto">
      <div className="flex flex-col gap-3">
        {renderConnectionStatus()}

        {currentUsage ? (
          <>
            {/* CPU and Memory Usage - First Row */}
            <div className="flex gap-3 flex-wrap">
              <div className="flex-1 min-w-80">{renderCPUUsage()}</div>
              <div className="flex-1 min-w-80">{renderMemoryUsage()}</div>
            </div>

            {/* GPU Usage Cards - Arranged with wrapping support */}
            <div className="flex gap-3 flex-wrap">
              {!currentUsage?.gpu_devices ||
              currentUsage.gpu_devices.length === 0 ? (
                <div className="flex-1 min-w-80">
                  <Card title="GPU Usage">
                    <Text type="secondary">No GPU usage data available</Text>
                  </Card>
                </div>
              ) : (
                currentUsage.gpu_devices.map((gpuUsage, index) => {
                  // Find corresponding GPU info
                  const gpuInfo = hardwareInfo?.gpu_devices.find(
                    gpu => gpu.device_id === gpuUsage.device_id,
                  )

                  const gpuName =
                    gpuInfo?.name || gpuUsage.device_name || `GPU ${index + 1}`

                  return (
                    <div key={index} className="flex-1 min-w-80">
                      <Card title={`${gpuName} Usage`}>
                        <div className="space-y-3">
                          {gpuUsage.utilization_percentage !== undefined && (
                            <div>
                              <Text strong>GPU Utilization</Text>
                              <Progress
                                percent={gpuUsage.utilization_percentage}
                                status={
                                  gpuUsage.utilization_percentage > 90
                                    ? 'exception'
                                    : 'active'
                                }
                                format={percent =>
                                  `${percent != null ? percent.toFixed(1) : '0.0'}%`
                                }
                              />
                            </div>
                          )}

                          {(gpuUsage.memory_usage_percentage !== undefined ||
                            (gpuUsage.memory_used !== undefined &&
                              gpuUsage.memory_total !== undefined)) && (
                            <div>
                              <Text strong>
                                {gpuInfo?.vendor?.includes('Apple')
                                  ? 'System Memory Usage'
                                  : 'GPU Memory'}
                              </Text>
                              {gpuUsage.memory_usage_percentage !==
                              undefined ? (
                                <Progress
                                  percent={gpuUsage.memory_usage_percentage}
                                  status={
                                    gpuUsage.memory_usage_percentage > 90
                                      ? 'exception'
                                      : 'active'
                                  }
                                  format={percent =>
                                    `${percent != null ? percent.toFixed(1) : '0.0'}%`
                                  }
                                />
                              ) : (
                                gpuUsage.memory_used !== undefined &&
                                gpuUsage.memory_total !== undefined && (
                                  <Progress
                                    percent={
                                      (gpuUsage.memory_used /
                                        gpuUsage.memory_total) *
                                      100
                                    }
                                    status={
                                      (gpuUsage.memory_used /
                                        gpuUsage.memory_total) *
                                        100 >
                                      90
                                        ? 'exception'
                                        : 'active'
                                    }
                                    format={percent =>
                                      `${percent != null ? percent.toFixed(1) : '0.0'}%`
                                    }
                                  />
                                )
                              )}

                              {gpuUsage.memory_used !== undefined &&
                                gpuUsage.memory_total !== undefined && (
                                  <div className="mt-1">
                                    <Text
                                      type="secondary"
                                      style={{ fontSize: '12px' }}
                                    >
                                      {gpuInfo?.vendor?.includes('Apple')
                                        ? 'GPU Memory Used: '
                                        : 'Used: '}
                                      {formatBytes(gpuUsage.memory_used)}
                                      {gpuInfo?.vendor?.includes('Apple')
                                        ? ` of ${formatBytes(gpuUsage.memory_total)} system memory`
                                        : ` / ${formatBytes(gpuUsage.memory_total)}`}
                                    </Text>
                                  </div>
                                )}
                            </div>
                          )}

                          <div className="flex gap-3">
                            {gpuUsage.temperature !== undefined && (
                              <Text
                                type="secondary"
                                style={{ fontSize: '12px' }}
                              >
                                Temperature: {gpuUsage.temperature}°C
                              </Text>
                            )}
                            {gpuUsage.power_usage !== undefined && (
                              <Text
                                type="secondary"
                                style={{ fontSize: '12px' }}
                              >
                                Power: {gpuUsage.power_usage}W
                              </Text>
                            )}
                          </div>
                        </div>
                      </Card>
                    </div>
                  )
                })
              )}
            </div>
          </>
        ) : (
          <Card>
            <div className="text-center py-8">
              <Text type="secondary">
                {sseConnected
                  ? 'Waiting for usage data...'
                  : 'Connect to hardware monitoring to view real-time usage data'}
              </Text>
            </div>
          </Card>
        )}
      </div>
    </div>
  )
}
