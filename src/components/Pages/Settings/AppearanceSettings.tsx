import { App, Card, Divider, Flex, Form, Select, Space, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useAppearanceSettings } from '../../../store'

const { Title, Text } = Typography

export function AppearanceSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
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
        message.success(t('appearance.themeUpdated'))
      }
      if ('componentSize' in changedValues) {
        await setComponentSize(changedValues.componentSize)
        message.success(t('appearance.componentSizeUpdated'))
      }
      if ('language' in changedValues) {
        await setLanguage(changedValues.language)
        message.success(t('appearance.languageUpdated'))
      }
    } catch (error: any) {
      message.error(error?.message || 'Failed to update settings')
      form.setFieldsValue({
        theme,
        componentSize,
        language,
      })
    }
  }

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Title level={3}>{t('pages.appearance')}</Title>

      <Card title={t('appearance.themeAndDisplay')}>
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
                <Text strong>{t('labels.theme')}</Text>
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
                    { value: 'light', label: t('appearance.light') },
                    { value: 'dark', label: t('appearance.dark') },
                    { value: 'system', label: t('appearance.system') },
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
                <Text strong>{t('labels.componentSize')}</Text>
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
                    { value: 'small', label: t('appearance.small') },
                    { value: 'medium', label: t('appearance.medium') },
                    { value: 'large', label: t('appearance.large') },
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
                <Text strong>{t('labels.language')}</Text>
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
                    { value: 'en', label: t('appearance.english') },
                    { value: 'vi', label: t('appearance.vietnamese') },
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
