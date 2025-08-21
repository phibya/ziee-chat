import { App, Card, Divider, Flex, Form, Select, Typography } from 'antd'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Stores,
  useUserAppearanceLanguage,
  useUserAppearanceTheme,
} from '../../../../store'
import {
  setUserAppearanceLanguage,
  setUserAppearanceTheme,
} from '../../../../store/settings.ts'
import { LANGUAGE_OPTIONS } from '../../../../types'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'

const { Text } = Typography

export function AppearanceSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()

  const theme = useUserAppearanceTheme()
  const language = useUserAppearanceLanguage()
  const { loading } = Stores.Settings

  useEffect(() => {
    form.setFieldsValue({
      theme,
      language,
    })
  }, [theme, language]) // Removed form from dependencies to prevent infinite rerenders

  const handleFormChange = async (changedValues: any) => {
    try {
      if ('theme' in changedValues) {
        await setUserAppearanceTheme(changedValues.theme)
        message.success(t('appearance.themeUpdated'))
      }
      if ('language' in changedValues) {
        await setUserAppearanceLanguage(changedValues.language)
        message.success(t('appearance.languageUpdated'))
      }
    } catch (error: any) {
      message.error(error?.message || 'Failed to update settings')
      form.setFieldsValue({
        theme,
        language,
      })
    }
  }

  return (
    <SettingsPageContainer title={t('pages.appearance')}>
      <Card title={t('appearance.themeAndDisplay')}>
        <Form
          form={form}
          onValuesChange={handleFormChange}
          initialValues={{
            theme,
            language,
          }}
        >
          <Flex vertical className="gap-2 w-full">
            <Flex
              justify="space-between"
              align="flex-start"
              wrap
              gap="small"
              className="min-w-0"
            >
              <div className="flex-1 min-w-80">
                <Text strong>{t('labels.theme')}</Text>
                <div>
                  <Text type="secondary">
                    Choose your preferred theme or match the OS theme.
                  </Text>
                </div>
              </div>
              <div className="flex-shrink-0">
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
              </div>
            </Flex>
            <Divider style={{ margin: 0 }} />
            <Flex
              justify="space-between"
              align="flex-start"
              wrap
              gap="small"
              className="min-w-0"
            >
              <div className="flex-1 min-w-80">
                <Text strong>{t('labels.language')}</Text>
                <div>
                  <Text type="secondary">
                    Choose your preferred language for the interface.
                  </Text>
                </div>
              </div>
              <div className="flex-shrink-0">
                <Form.Item name="language" style={{ margin: 0 }}>
                  <Select
                    loading={loading}
                    style={{ minWidth: 120 }}
                    options={LANGUAGE_OPTIONS}
                  />
                </Form.Item>
              </div>
            </Flex>
          </Flex>
        </Form>
      </Card>
    </SettingsPageContainer>
  )
}
