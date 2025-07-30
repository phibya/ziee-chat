import {
  App,
  Button,
  Card,
  Divider,
  Flex,
  Form,
  Input,
  Radio,
  Select,
  Typography,
  Alert,
} from 'antd'
import { useEffect, useState } from 'react'
import { Permission, usePermissions } from '../../../../permissions'
import { SettingsPageContainer } from '../SettingsPageContainer'
import {
  initializeDocumentExtraction,
  setExtractionMethod,
  setOcrSettings,
  setLlmSettings,
  validateLlmSettings,
  validateOcrSettings,
  Stores,
  loadAllModelProviders,
} from '../../../../store'
import type {
  ExtractionFileType,
  OcrExtractionSettings,
  LlmExtractionSettings,
  DocumentModelParameters,
} from '../../../../types'
import { OCR_LANGUAGES } from '../../../../types'
import { ModelParametersSection } from '../Providers/shared'
import { MODEL_PARAMETERS } from '../../../../constants/modelParameters.ts'

const { Text, Title } = Typography
const { TextArea } = Input

interface ParserCardProps {
  fileType: ExtractionFileType
  title: string
  description: string
  availableMethods: Array<'simple' | 'ocr' | 'llm'>
}

function ParserCard({
  fileType,
  title,
  description,
  availableMethods,
}: ParserCardProps) {
  const { message } = App.useApp()
  const [form] = Form.useForm()
  const [loading, setLoading] = useState(false)
  const { hasPermission } = usePermissions()

  const { pdfSettings, imageSettings, initialized } = Stores.DocumentExtraction
  const { modelsByProvider } = Stores.AdminProviders
  const settings = fileType === 'pdf' ? pdfSettings : imageSettings

  // Watch for method changes to show/hide sections
  const selectedMethod = Form.useWatch('method', form)

  // Get all vision-capable models for LLM extraction
  const getVisionCapableModels = () => {
    const allModels = Object.values(modelsByProvider).flat()
    return allModels.filter(
      model => model.enabled && model.capabilities?.vision === true,
    )
  }

  const visionModels = getVisionCapableModels()

  useEffect(() => {
    if (initialized) {
      form.setFieldsValue({
        method: settings.method,
        ocrLanguage: settings.ocr.language,
        ocrEngine: settings.ocr.engine,
        llmModelId: settings.llm.model_id,
        llmSystemPrompt: settings.llm.system_prompt,
        temperature: settings.llm.parameters.temperature || 0.2,
        topP: settings.llm.parameters.top_p || 0.9,
        topK: settings.llm.parameters.top_k || 20,
        maxTokens: settings.llm.parameters.max_tokens || 4096,
      })
    }
  }, [settings, initialized, form])

  const handleMethodChange = (method: 'simple' | 'ocr' | 'llm') => {
    // Just update the form value, don't auto-save
    form.setFieldValue('method', method)
  }

  const handleFormSubmit = async (values: any) => {
    if (!hasPermission(Permission.config.documentExtraction.edit)) {
      message.error('No permission to configure document extraction')
      return
    }

    setLoading(true)
    try {
      // Update extraction method
      await setExtractionMethod(fileType, values.method)

      // Update settings based on method
      if (values.method === 'ocr') {
        const ocrSettings: OcrExtractionSettings = {
          language: values.ocrLanguage,
          engine: values.ocrEngine,
        }
        const validationErrors = validateOcrSettings(ocrSettings)
        if (validationErrors.length > 0) {
          message.error(validationErrors.join(', '))
          return
        }
        await setOcrSettings(fileType, ocrSettings)
      } else if (values.method === 'llm') {
        const modelParameters: DocumentModelParameters = {
          temperature: values.temperature,
          top_p: values.topP,
          top_k: values.topK,
          max_tokens: values.maxTokens,
          // Include other existing parameters
          ...settings.llm.parameters,
        }

        const llmSettings: LlmExtractionSettings = {
          model_id: values.llmModelId,
          system_prompt: values.llmSystemPrompt,
          parameters: modelParameters,
        }
        const validationErrors = validateLlmSettings(llmSettings)
        if (validationErrors.length > 0) {
          message.error(validationErrors.join(', '))
          return
        }
        await setLlmSettings(fileType, llmSettings)
      }

      message.success(`${title} settings updated successfully`)
    } catch (error) {
      console.error('Failed to update settings:', error)
      message.error('Failed to update settings')
    } finally {
      setLoading(false)
    }
  }

  const renderMethodDescription = (method: string) => {
    switch (method) {
      case 'simple':
        return (
          <Text type="secondary">
            {fileType === 'pdf'
              ? 'Extract text directly from PDF using built-in text content'
              : 'Not available for images'}
          </Text>
        )
      case 'ocr':
        return (
          <Text type="secondary">
            Convert document to image and use OCR to extract text
          </Text>
        )
      case 'llm':
        return (
          <Text type="secondary">
            Use vision-capable AI model to analyze and extract text from
            document images
          </Text>
        )
      default:
        return null
    }
  }

  return (
    <Card title={title}>
      <Text type="secondary" style={{ marginBottom: 16, display: 'block' }}>
        {description}
      </Text>

      <Form form={form} layout="vertical" disabled={loading}>
        {/* Method Selection */}
        <Form.Item label="Extraction Method" name="method">
          <Radio.Group
            onChange={e => handleMethodChange(e.target.value)}
            disabled={!hasPermission(Permission.config.documentExtraction.edit)}
          >
            {availableMethods.map(method => (
              <Radio.Button key={method} value={method}>
                {method === 'simple' && 'Simple Text'}
                {method === 'ocr' && 'OCR'}
                {method === 'llm' && 'AI Vision'}
              </Radio.Button>
            ))}
          </Radio.Group>
        </Form.Item>

        {renderMethodDescription(selectedMethod || settings.method)}

        {/* OCR Settings */}
        {(selectedMethod || settings.method) === 'ocr' && (
          <>
            <Divider />
            <Title level={5}>OCR Settings</Title>

            <Form.Item label="Language" name="ocrLanguage">
              <Select
                placeholder="Select language"
                disabled={
                  !hasPermission(Permission.config.documentExtraction.edit)
                }
                options={OCR_LANGUAGES.map(lang => ({
                  key: lang.value,
                  value: lang.value,
                  label: lang.label,
                }))}
              />
            </Form.Item>

            <Form.Item label="Engine" name="ocrEngine">
              <Select
                disabled={
                  !hasPermission(Permission.config.documentExtraction.edit)
                }
                options={[
                  {
                    key: 'tesseract',
                    value: 'tesseract',
                    label: 'Tesseract',
                  },
                ]}
              />
            </Form.Item>
          </>
        )}

        {/* LLM Settings */}
        {(selectedMethod || settings.method) === 'llm' && (
          <>
            <Divider />
            <Title level={5}>AI Vision Settings</Title>

            <Form.Item
              label="Vision Model"
              name="llmModelId"
              rules={[
                {
                  required: true,
                  message:
                    'Please select a vision model when using AI Vision method',
                },
              ]}
            >
              <Select
                placeholder="Select a vision-capable model"
                disabled={
                  !hasPermission(Permission.config.documentExtraction.edit)
                }
                allowClear={false}
                showSearch
                filterOption={(input, option) =>
                  (option?.label ?? '')
                    .toString()
                    .toLowerCase()
                    .includes(input.toLowerCase())
                }
                notFoundContent={
                  visionModels.length === 0
                    ? 'No vision-capable models available'
                    : 'No results found'
                }
                options={visionModels.map(model => ({
                  key: model.id,
                  value: model.id,
                  label: model.alias,
                  model: model,
                }))}
                optionRender={option => (
                  <div>
                    <div style={{ fontWeight: 500 }}>
                      {option.data.model.alias}
                    </div>
                    <div style={{ fontSize: '12px', color: '#666' }}>
                      {option.data.model.name}
                    </div>
                  </div>
                )}
              />
            </Form.Item>

            <Form.Item
              label="System Prompt"
              name="llmSystemPrompt"
              rules={[{ required: true, message: 'System prompt is required' }]}
            >
              <TextArea
                rows={3}
                placeholder="Extract all text from this document image. Maintain formatting and structure."
                disabled={
                  !hasPermission(Permission.config.documentExtraction.edit)
                }
              />
            </Form.Item>

            <Title level={5}>Model Parameters</Title>

            <Flex gap="middle" className={'flex-col !mb-2'}>
              <ModelParametersSection parameters={MODEL_PARAMETERS} />
            </Flex>
          </>
        )}

        {/* Save Button */}
        <Form.Item>
          <Button
            type="primary"
            onClick={() => form.validateFields().then(handleFormSubmit)}
            loading={loading}
            disabled={!hasPermission(Permission.config.documentExtraction.edit)}
          >
            Save
          </Button>
        </Form.Item>
      </Form>
    </Card>
  )
}

export function DocumentExtractionSettings() {
  const { message } = App.useApp()
  const { hasPermission } = usePermissions()

  const { initialized, error } = Stores.DocumentExtraction

  useEffect(() => {
    const loadSettings = async () => {
      if (!initialized) {
        try {
          await Promise.all([
            initializeDocumentExtraction(),
            loadAllModelProviders(), // Load providers and models for model selection
          ])
        } catch (error) {
          console.error('Failed to initialize document extraction:', error)
          message.error('Failed to initialize document extraction')
        }
      }
    }

    loadSettings()
  }, [initialized, message])

  if (!hasPermission(Permission.config.documentExtraction.read)) {
    return (
      <SettingsPageContainer title="Document Parser">
        <Alert
          message="No permission to view document extraction settings"
          type="error"
          showIcon
        />
      </SettingsPageContainer>
    )
  }

  return (
    <SettingsPageContainer title="Document Parser">
      {error && (
        <Alert
          message="Failed to load settings"
          description={error}
          type="error"
          showIcon
          style={{ marginBottom: 16 }}
        />
      )}

      <Flex vertical gap="large">
        <ParserCard
          fileType="pdf"
          title="PDF Parser"
          description="Extract text content from PDF documents"
          availableMethods={['simple', 'ocr', 'llm']}
        />

        <ParserCard
          fileType="image"
          title="Image Parser"
          description="Extract text content from image files"
          availableMethods={['ocr', 'llm']}
        />
      </Flex>
    </SettingsPageContainer>
  )
}
