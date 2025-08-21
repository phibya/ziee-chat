import { Card, Flex, Form, Switch, Typography } from 'antd'
import { useTranslation } from 'react-i18next'

const { Text } = Typography

interface AdvancedCardProps {
  form: any
  experimentalFeatures: boolean
  onFormChange: (changedValues: any) => void
  isAdmin?: boolean
}

export function AdvancedCard({
  form,
  experimentalFeatures,
  onFormChange,
  isAdmin = false,
}: AdvancedCardProps) {
  const { t } = useTranslation()

  return (
    <Card title={isAdmin ? t('admin.advanced') : t('general.advanced')}>
      <Form
        form={form}
        onValuesChange={onFormChange}
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
