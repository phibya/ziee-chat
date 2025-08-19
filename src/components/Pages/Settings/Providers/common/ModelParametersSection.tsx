import { Typography } from 'antd'
import {
  ModelParameterField,
  ParameterFieldConfig,
} from '../../../../common/ModelParameterField'

const { Title } = Typography

interface ModelParametersSectionProps {
  title?: string
  parameters: ParameterFieldConfig[]
}

export function ModelParametersSection({
  title,
  parameters,
}: ModelParametersSectionProps) {
  return (
    <>
      {title && <Title level={5}>{title}</Title>}
      {parameters.map((param, index) => (
        <ModelParameterField key={index} {...param} />
      ))}
    </>
  )
}
