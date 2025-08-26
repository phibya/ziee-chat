import { App, Card, Flex, Form, Switch, Typography } from 'antd'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

const { Text } = Typography

interface AdvancedCardProps {
  isAdmin?: boolean
}

export function AdvancedCard({
  isAdmin = false,
}: AdvancedCardProps) {
  const { t } = useTranslation()
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [experimentalFeatures, setExperimentalFeatures] = useState(false)

  const handleFormChange = async (changedValues: any) => {
    try {
      if ('experimentalFeatures' in changedValues) {
        setExperimentalFeatures(changedValues.experimentalFeatures)
        message.success(
          changedValues.experimentalFeatures
            ? t('admin.experimentalEnabled')
            : t('admin.experimentalDisabled'),
        )
      }
    } catch (error: any) {
      message.error(error?.message || t('common.failedToUpdate'))
      form.setFieldsValue({
        experimentalFeatures,
      })
    }
  }

  return (
    <Card title={isAdmin ? t('admin.advanced') : t('general.advanced')}>
      <Form
        form={form}
        onValuesChange={handleFormChange}
        initialValues={{
          experimentalFeatures,
        }}
      >
        <Flex justify="space-between" align="center">
          <div>
            <Text strong>
              {isAdmin
                ? t('admin.experimentalFeatures')
                : t('labels.experimentalFeatures')}
            </Text>
            <div>
              <Text type="secondary">
                {isAdmin
                  ? t('admin.experimentalFeaturesDesc')
                  : t('general.experimentalFeaturesDescription')}
              </Text>
            </div>
          </div>
          <Form.Item
            name="experimentalFeatures"
            valuePropName="checked"
            style={{ margin: 0 }}
          >
            <Switch />
          </Form.Item>
        </Flex>
      </Form>
    </Card>
  )
}
