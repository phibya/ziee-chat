import { Form, Input, InputNumber, Select } from 'antd'

const { TextArea } = Input

export interface ParameterFieldConfig {
  name: string | string[]
  label: string
  help?: string
  placeholder?: string
  type: 'number' | 'text' | 'password' | 'textarea' | 'select'
  min?: number
  max?: number
  step?: number
  required?: boolean
  options?: Array<{ value: string | number; label: string }>
  rules?: any[]
}

type ModelParameterFieldProps = ParameterFieldConfig

export function ModelParameterField({
  name,
  label,
  help,
  placeholder,
  type,
  min,
  max,
  step,
  required,
  options,
  rules = [],
}: ModelParameterFieldProps) {
  const fieldRules = [
    ...(required ? [{ required: true, message: `${label} is required` }] : []),
    ...rules,
  ]

  const renderInput = () => {
    const commonStyle = { width: '100%' }

    switch (type) {
      case 'number':
        return (
          <InputNumber
            placeholder={placeholder}
            style={commonStyle}
            min={min}
            max={max}
            step={step}
          />
        )
      case 'password':
        return <Input.Password placeholder={placeholder} style={commonStyle} />
      case 'textarea':
        return <TextArea placeholder={placeholder} rows={3} />
      case 'select':
        return (
          <Select
            placeholder={placeholder}
            style={commonStyle}
            options={options}
          />
        )
      case 'text':
      default:
        return <Input placeholder={placeholder} style={commonStyle} />
    }
  }

  return (
    <Form.Item name={name} label={label} help={help} rules={fieldRules}>
      {renderInput()}
    </Form.Item>
  )
}
