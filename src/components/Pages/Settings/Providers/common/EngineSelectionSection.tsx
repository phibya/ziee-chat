import React, { useEffect } from 'react'
import { Alert, Card, Form, Select, Spin, Typography } from 'antd'
import { initializeEngines, Stores } from '../../../../../store'

const { Text } = Typography

const ENGINE_DESCRIPTIONS = {
  mistralrs:
    'High-performance inference engine optimized for Mistral and Llama models',
  llamacpp: 'GPU-optimized inference engine with broad model format support',
  default: 'Local model execution engine',
} as const

export const EngineSelectionSection: React.FC = () => {
  const { engines, loading, error, initialized } = Stores.AdminEngines
  const form = Form.useFormInstance()
  const selectedEngineType = form.getFieldValue('engine_type') || 'mistralrs'

  const { modelId } = Stores.UI.EditLocalModelDrawer
  const { providers } = Stores.AdminProviders

  // Find the current model from all providers
  const currentModel = modelId
    ? providers.flatMap(p => p.models).find(m => m.id === modelId)
    : null

  // Initialize engines on mount
  useEffect(() => {
    if (!initialized) {
      initializeEngines().catch(console.error)
    }
  }, [initialized])

  const getEngineDisplayName = (engine: any) => {
    return engine.name || engine.engine_type
  }

  if (loading) {
    return (
      <Card title="Engine Configuration" size="small">
        <div className="text-center py-4">
          <Spin size="large" />
          <div className="mt-2">Loading available engines...</div>
        </div>
      </Card>
    )
  }

  if (error) {
    return (
      <Alert
        message="Engine Loading Error"
        description={error}
        type="error"
        showIcon
      />
    )
  }

  return (
    <Card title="Engine Configuration">
      <div className="space-y-4">
        <div>
          <Text type="secondary">
            Select the inference engine to use for running this model locally
          </Text>
        </div>

        <Form.Item
          label="Inference Engine"
          name="engine_type"
          tooltip="The engine that will be used to run this model locally"
          initialValue={selectedEngineType}
          rules={[
            {
              required: true,
              message: 'Please select an inference engine',
            },
          ]}
        >
          <Select
            placeholder="Select inference engine"
            options={engines?.map(engine => {
              // Disable LlamaCpp if model file format is not "gguf"
              const isLlamaCppDisabled =
                engine.engine_type === 'llamacpp' &&
                currentModel &&
                currentModel.file_format !== 'gguf'

              return {
                value: engine.engine_type,
                label: getEngineDisplayName(engine),
                disabled: Boolean(isLlamaCppDisabled),
                engine: engine,
              }
            })}
            optionRender={option => (
              <div className={'flex flex-col gap-1 py-1'}>
                <Text strong>
                  {option.label}
                  {option.data.disabled && ' (Not compatible)'}
                </Text>
                {option.data.engine.version && (
                  <Text className={`!text-xs`}>
                    Version: {option.data.engine.version}
                  </Text>
                )}
                <Text className={`!text-xs`}>
                  {option.data.disabled &&
                  option.data.engine.engine_type === 'llamacpp'
                    ? `Requires GGUF format (current: ${currentModel?.file_format || 'unknown'})`
                    : ENGINE_DESCRIPTIONS[
                        option.data.engine
                          .engine_type as keyof typeof ENGINE_DESCRIPTIONS
                      ] || ENGINE_DESCRIPTIONS.default}
                </Text>
              </div>
            )}
          />
        </Form.Item>

        {engines?.length === 0 && (
          <Alert
            message="No Engines Available"
            description="No local inference engines are currently available. Please ensure engines are properly installed."
            type="warning"
            showIcon
          />
        )}
      </div>
    </Card>
  )
}
