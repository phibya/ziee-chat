import {
  Card,
  Divider,
  Flex,
  Form,
  message,
  Select,
  Space,
  Typography,
} from 'antd'
import { useEffect, useState } from 'react'
import { useAppearanceSettings } from '../../../store'

const { Title, Text } = Typography

export function AppearanceSettings() {
  const [form] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)
  const {
    theme,
    componentSize,
    language,
    setTheme,
    setComponentSize,
    setLanguage,
    loading,
  } = useAppearanceSettings()

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768)
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)

    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  useEffect(() => {
    form.setFieldsValue({
      theme,
      componentSize,
      language,
    })
  }, [theme, componentSize, language, form])

  const handleFormChange = async (changedValues: any) => {
    try {
      if ('theme' in changedValues) {
        await setTheme(changedValues.theme)
        message.success('Theme updated successfully')
      }
      if ('componentSize' in changedValues) {
        await setComponentSize(changedValues.componentSize)
        message.success('Component size updated successfully')
      }
      if ('language' in changedValues) {
        await setLanguage(changedValues.language)
        message.success('Language updated successfully')
      }
    } catch {
      message.error('Failed to update settings')
      form.setFieldsValue({
        theme,
        componentSize,
        language,
      })
    }
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>Appearance</Title>

      <Card title="Theme & Display">
        <Form
          form={form}
          onValuesChange={handleFormChange}
          initialValues={{
            theme,
            componentSize,
            language,
          }}
        >
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>Theme</Text>
                <div>
                  <Text type="secondary">
                    Choose your preferred theme or match the OS theme.
                  </Text>
                </div>
              </div>
              <Form.Item name="theme" style={{ margin: 0 }}>
                <Select
                  loading={loading}
                  style={{ minWidth: 120 }}
                  options={[
                    { value: 'light', label: 'Light' },
                    { value: 'dark', label: 'Dark' },
                    { value: 'system', label: 'System' },
                  ]}
                />
              </Form.Item>
            </Flex>
            <Divider style={{ margin: 0 }} />
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>Component Size</Text>
                <div>
                  <Text type="secondary">
                    Adjust the size of UI components throughout the app.
                  </Text>
                </div>
              </div>
              <Form.Item name="componentSize" style={{ margin: 0 }}>
                <Select
                  loading={loading}
                  style={{ minWidth: 120 }}
                  options={[
                    { value: 'small', label: 'Small' },
                    { value: 'medium', label: 'Medium' },
                    { value: 'large', label: 'Large' },
                  ]}
                />
              </Form.Item>
            </Flex>
            <Divider style={{ margin: 0 }} />
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>Language</Text>
                <div>
                  <Text type="secondary">
                    Choose your preferred language for the interface.
                  </Text>
                </div>
              </div>
              <Form.Item name="language" style={{ margin: 0 }}>
                <Select
                  loading={loading}
                  style={{ minWidth: 120 }}
                  options={[
                    { value: 'en', label: 'English' },
                    { value: 'vi', label: 'Tiếng Việt' },
                  ]}
                />
              </Form.Item>
            </Flex>
          </Space>
        </Form>
      </Card>
    </Space>
  )
}
