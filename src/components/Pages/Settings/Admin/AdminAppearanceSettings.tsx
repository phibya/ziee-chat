import { App, Card, Flex, Form, Select, Typography } from 'antd'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { isTauriView } from '../../../../api/core'
import {
  loadGlobalDefaultLanguage,
  Stores,
  updateSystemDefaultLanguage,
} from '../../../../store'
import { LANGUAGE_OPTIONS } from '../../../../types'
import { SettingsPageContainer } from '../common/SettingsPageContainer.tsx'

const { Text } = Typography

export function AdminAppearanceSettings() {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [isMobile, setIsMobile] = useState(false)
  const { globalDefaultLanguage } = Stores.Settings
  const { updating } = Stores.Admin

  // Check permissions - using a general config permission for appearance settings

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
      language: globalDefaultLanguage,
    })
  }, [globalDefaultLanguage]) // Removed form from dependencies to prevent infinite rerenders

  const handleFormChange = async (changedValues: any) => {
    if ('language' in changedValues) {
      try {
        // Update global default language via admin store
        await updateSystemDefaultLanguage(changedValues.language)

        // Update the store's global language
        await loadGlobalDefaultLanguage()

        message.success('Default language updated successfully')
      } catch {
        console.error('Failed to update default language')
        // Error is handled by the store
        form.setFieldsValue({ language: globalDefaultLanguage })
      }
    }
  }

  if (isTauriView) {
    return (
      <SettingsPageContainer title="Admin Appearance Settings">
        <Card>
          <div className="text-center">
            <Text type="secondary">
              Admin appearance settings are disabled in desktop mode
            </Text>
          </div>
        </Card>
      </SettingsPageContainer>
    )
  }

  return (
    <SettingsPageContainer title={t('admin.appearanceSettings')}>
      <Card title={t('admin.defaultSystemSettings')}>
        <Form
          form={form}
          onValuesChange={handleFormChange}
          initialValues={{
            language: globalDefaultLanguage,
          }}
        >
          <Flex vertical className="gap-2 w-full">
            <Flex
              justify="space-between"
              align={isMobile ? 'flex-start' : 'center'}
              vertical={isMobile}
              gap={isMobile ? 'small' : 0}
            >
              <div>
                <Text strong>Default Language</Text>
                <div>
                  <Text type="secondary">
                    Set the default language for new users and the system
                    interface.
                  </Text>
                </div>
              </div>
              <Form.Item name="language" style={{ margin: 0 }}>
                <Select
                  loading={updating}
                  style={{ minWidth: 120 }}
                  options={LANGUAGE_OPTIONS}
                />
              </Form.Item>
            </Flex>
          </Flex>
        </Form>
      </Card>
    </SettingsPageContainer>
  )
}
