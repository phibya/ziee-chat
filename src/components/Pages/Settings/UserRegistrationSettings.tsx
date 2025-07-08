import { useEffect, useState } from 'react'
import { App, Card, Form, Switch, Typography } from 'antd'
import { ApiClient } from '../../../api/client'
import { Permission, usePermissions } from '../../../permissions'

const { Text } = Typography

export function UserRegistrationSettings() {
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)
  const [registrationEnabled, setRegistrationEnabled] = useState(true)
  const { hasPermission } = usePermissions()

  const canRead = hasPermission(Permission.config.userRegistration.read)
  const canEdit = hasPermission(Permission.config.userRegistration.edit)

  useEffect(() => {
    fetchRegistrationStatus()
  }, [])

  const fetchRegistrationStatus = async () => {
    try {
      const { enabled } = await ApiClient.Admin.getUserRegistrationStatus()
      setRegistrationEnabled(enabled)
      form.setFieldsValue({ enabled })
    } catch (error) {
      message.error(
        error instanceof Error
          ? error.message
          : 'Failed to fetch registration status',
      )
    }
  }

  const handleFormChange = async (changedValues: any) => {
    if (!canEdit) {
      message.error('You do not have permission to edit this setting')
      return
    }
    if ('enabled' in changedValues) {
      const newValue = changedValues.enabled

      // Optimistic update - assume success
      setRegistrationEnabled(newValue)
      setLoading(true)

      try {
        await ApiClient.Admin.updateUserRegistrationStatus({
          enabled: newValue,
        })
        message.success(
          `User registration ${newValue ? 'enabled' : 'disabled'} successfully`,
        )
      } catch (error) {
        // Revert on error
        setRegistrationEnabled(!newValue)
        form.setFieldsValue({ enabled: !newValue })

        message.error(
          error instanceof Error
            ? error.message
            : 'Failed to update registration status',
        )
      } finally {
        setLoading(false)
      }
    }
  }

  if (!canRead) {
    return null
  }

  return (
    <Card title="User Registration" className="mb-6">
      <Form
        form={form}
        onValuesChange={handleFormChange}
        initialValues={{ enabled: registrationEnabled }}
      >
        <div className="flex justify-between items-center">
          <div>
            <Text strong>Enable User Registration</Text>
            <div>
              <Text type="secondary">
                Allow new users to register for accounts
              </Text>
            </div>
          </div>
          <Form.Item name="enabled" valuePropName="checked" className="mb-0">
            <Switch loading={loading} size="default" />
          </Form.Item>
        </div>
      </Form>
    </Card>
  )
}
