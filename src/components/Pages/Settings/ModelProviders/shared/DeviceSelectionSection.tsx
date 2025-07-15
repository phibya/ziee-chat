import React, { useEffect, useState } from 'react'
import { Alert, Card, Form, Select, Spin, Typography } from 'antd'
import { ApiClient } from '../../../../../api/client'
import {
  AvailableDevicesResponse,
  DeviceInfo,
} from '../../../../../types/api/modelProvider'
import { useUpdate } from 'react-use'

const { Title, Text } = Typography

interface DeviceSelectionSectionProps {
  value?: {
    device_type?: string
    device_ids?: string[]
  }
  onChange?: (value: { device_type?: string; device_ids?: string[] }) => void
  disabled?: boolean
}

export const DeviceSelectionSection: React.FC<DeviceSelectionSectionProps> = ({
  onChange,
}) => {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [devicesByType, setDevicesByType] = useState<
    Record<string, DeviceInfo[]>
  >({})
  const update = useUpdate()
  const form = Form.useFormInstance()
  const selectedDeviceType = form.getFieldValue('device_type') || 'cpu'
  const selectedDeviceIds = form.getFieldValue('device_ids') || []

  // Fetch available devices
  useEffect(() => {
    const fetchDevices = async () => {
      try {
        setLoading(true)
        setError(null)
        console.log('Fetching available devices...')

        const response: AvailableDevicesResponse =
          await ApiClient.Admin.getAvailableDevices()

        console.log('Device response:', response)

        // Group devices by type
        const grouped = response.devices.reduce(
          (acc, device) => {
            if (!acc[device.device_type]) {
              acc[device.device_type] = []
            }
            acc[device.device_type].push(device)
            return acc
          },
          {} as Record<string, DeviceInfo[]>,
        )

        console.log('Grouped devices:', grouped)
        setDevicesByType(grouped)
      } catch (err) {
        console.error('Failed to fetch available devices:', err)
        setError(
          `Failed to load available devices: ${err instanceof Error ? err.message : 'Unknown error'}`,
        )
      } finally {
        setLoading(false)
      }
    }

    fetchDevices()
  }, [])

  const availableDeviceTypes = Object.keys(devicesByType)
  const availableDevicesForType = selectedDeviceType
    ? devicesByType[selectedDeviceType] || []
    : []

  const handleDeviceTypeChange = (deviceType: string) => {
    const newValue = {
      device_type: deviceType,
      device_ids: selectedDeviceIds,
    }
    onChange?.(newValue)
    form.setFieldsValue({
      device_ids: [devicesByType[deviceType]?.[0]?.id || undefined].filter(
        Boolean,
      ), // Set first device ID if available
    }) // Update form state
    update()
  }

  const handleDeviceIdsChange = (deviceIds: string[]) => {
    const newValue = {
      device_type: selectedDeviceType,
      device_ids: deviceIds,
    }
    onChange?.(newValue)
  }

  const formatMemorySize = (bytes?: number) => {
    if (!bytes) return 'Unknown'
    const gb = bytes / (1024 * 1024 * 1024)
    return `${gb.toFixed(1)} GB`
  }

  const getDeviceTypeLabel = (deviceType: string) => {
    switch (deviceType) {
      case 'cpu':
        return 'CPU'
      case 'cuda':
        return 'NVIDIA CUDA'
      case 'metal':
        return 'Apple Metal'
      default:
        return deviceType.toUpperCase()
    }
  }

  if (loading) {
    return (
      <Card>
        <div className="text-center">
          <Spin size="large" />
          <div className="mt-2">Loading available devices...</div>
        </div>
      </Card>
    )
  }

  if (error) {
    return (
      <Alert
        message="Device Detection Error"
        description={error}
        type="error"
        showIcon
      />
    )
  }

  return (
    <div className="space-y-4 pb-3">
      <div>
        <Title level={5}>Device Configuration</Title>
        <Text type="secondary">
          Select the compute device type and specific devices for model
          execution
        </Text>
      </div>

      <Form.Item
        label="Device Type"
        name={['device_type']}
        tooltip="The type of compute device to use for model inference"
        initialValue={selectedDeviceType}
      >
        <Select
          placeholder="Select device type"
          onChange={handleDeviceTypeChange}
          options={availableDeviceTypes.map(deviceType => ({
            value: deviceType,
            label: getDeviceTypeLabel(deviceType),
            deviceType: deviceType,
            deviceCount: devicesByType[deviceType].length,
          }))}
          optionRender={option => (
            <div>
              <strong>{option.label}</strong>
              <div className="text-xs text-gray-500">
                {option.data.deviceCount} device(s) available
              </div>
            </div>
          )}
        />
      </Form.Item>

      {selectedDeviceType && (
        <Form.Item
          label="Specific Devices"
          name={['device_ids']}
          tooltip="Select specific devices to use. Leave empty to use the default device."
          initialValue={selectedDeviceIds}
        >
          <Select
            mode="multiple"
            placeholder="Select specific devices (optional)"
            onChange={handleDeviceIdsChange}
            allowClear
            options={availableDevicesForType.map(device => ({
              value: device.id,
              label: device.name,
              device: device,
            }))}
            optionRender={option => (
              <div>
                <strong>{option.label}</strong>
                {option.data.device.memory_total && (
                  <div className="text-xs text-gray-500">
                    Memory: {formatMemorySize(option.data.device.memory_total)}
                    {option.data.device.memory_free &&
                      ` (${formatMemorySize(option.data.device.memory_free)} free)`}
                  </div>
                )}
                <div className="text-xs text-gray-400">
                  ID: {option.data.device.id}
                </div>
              </div>
            )}
          />
        </Form.Item>
      )}

      {selectedDeviceType === 'cpu' && (
        <Alert
          message="CPU Selected"
          description="Model will run on CPU. This may be slower than GPU execution but works on all systems."
          type="info"
          showIcon
        />
      )}

      {selectedDeviceType === 'cuda' &&
        availableDevicesForType.length === 0 && (
          <Alert
            message="No CUDA Devices Found"
            description="No NVIDIA CUDA devices were detected. Make sure NVIDIA drivers and CUDA are properly installed."
            type="warning"
            showIcon
          />
        )}

      {selectedDeviceType === 'metal' &&
        availableDevicesForType.length === 0 && (
          <Alert
            message="No Metal Devices Found"
            description="No Apple Metal devices were detected. Metal is only available on macOS with compatible hardware."
            type="warning"
            showIcon
          />
        )}
    </div>
  )
}
