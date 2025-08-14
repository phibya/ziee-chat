import { Form, Select } from 'antd'
import { useTranslation } from 'react-i18next'
import { LOCAL_FILE_TYPE_OPTIONS } from '../../../../../constants/localModelTypes'
import { ModelParametersSection } from './ModelParametersSection'
import { LOCAL_MODEL_FIELDS } from '../../../../../constants/modelParameters.ts'

const ENGINE_OPTIONS = [
  {
    value: 'mistralrs',
    label: 'MistralRs',
    description: 'High-performance inference engine with advanced features'
  },
  {
    value: 'llamacpp',
    label: 'LlamaCpp',
    description: 'Coming soon - GGML-based inference engine'
  }
]

interface LocalModelCommonFieldsProps {
  onFileFormatChange?: (value: string) => void
  onEngineChange?: (value: string) => void
}

export function LocalModelCommonFields({
  onFileFormatChange,
  onEngineChange,
}: LocalModelCommonFieldsProps) {
  const { t } = useTranslation()

  return (
    <>
      <ModelParametersSection parameters={LOCAL_MODEL_FIELDS} />

      <Form.Item
        name="engine_type"
        label="Engine"
        rules={[
          {
            required: true,
            message: 'Please select an engine',
          },
        ]}
        initialValue="mistralrs"
      >
        <Select
          placeholder="Select Engine"
          onChange={onEngineChange}
          options={ENGINE_OPTIONS.map(option => ({
            value: option.value,
            label: option.label,
            disabled: option.value === 'llamacpp', // Disable LlamaCpp for now
          }))}
        />
      </Form.Item>

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
