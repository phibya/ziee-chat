import { Form, Select } from 'antd'
import { useTranslation } from 'react-i18next'
import { LOCAL_FILE_TYPE_OPTIONS } from '../../../../../constants/localModelTypes'
import { LOCAL_MODEL_FIELDS } from '../shared/constants'
import { ModelParametersSection } from './ModelParametersSection'

interface LocalModelCommonFieldsProps {
  onFileFormatChange?: (value: string) => void
}

export function LocalModelCommonFields({
  onFileFormatChange,
}: LocalModelCommonFieldsProps) {
  const { t } = useTranslation()

  return (
    <>
      <ModelParametersSection parameters={LOCAL_MODEL_FIELDS} />

      <Form.Item
        name="file_format"
        label={t('providers.fileFormat')}
        rules={[
          {
            required: true,
            message: t('providers.fileFormatRequired'),
          },
        ]}
      >
        <Select
          placeholder={t('providers.selectFileFormat')}
          onChange={onFileFormatChange}
          options={LOCAL_FILE_TYPE_OPTIONS.map(option => ({
            value: option.value,
            label: option.label,
            description: option.description,
          }))}
        />
      </Form.Item>
    </>
  )
}
