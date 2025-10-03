import { Button, Form, Select } from 'antd'
import { RobotOutlined, SettingOutlined } from '@ant-design/icons'
import { IoIosArrowDown } from 'react-icons/io'
import type { Assistant } from '../../../../types'

interface SelectorsProps {
  isBreaking: boolean
  isDisabled: boolean
  availableAssistants: Assistant[]
  availableModels: Array<{
    label: string
    options: Array<{ label: string; value: string; description?: string }>
  }>
}

export const Selectors = ({
  isBreaking,
  isDisabled,
  availableAssistants,
  availableModels,
}: SelectorsProps) => {
  return (
    <div className={'flex items-center gap-[6px]'}>
      <Form.Item name="assistant" noStyle>
        <Select
          popupMatchSelectWidth={false}
          placeholder="Assistant"
          options={availableAssistants.map((assistant: Assistant) => ({
            label: assistant.name,
            value: assistant.id,
          }))}
          style={{
            width: isBreaking ? 40 : 120,
            paddingLeft: isBreaking ? 0 : 6,
          }}
          labelRender={isBreaking ? () => '' : undefined}
          variant={isBreaking ? 'borderless' : undefined}
          prefix={
            isBreaking && (
              <Button>
                <RobotOutlined />
              </Button>
            )
          }
          suffixIcon={<IoIosArrowDown />}
        />
      </Form.Item>

      <Form.Item name="model" noStyle>
        <Select
          popupMatchSelectWidth={false}
          placeholder="Model"
          disabled={isDisabled}
          options={availableModels}
          style={{ width: isBreaking ? 40 : 120 }}
          variant={isBreaking ? 'borderless' : undefined}
          labelRender={isBreaking ? () => '' : undefined}
          prefix={
            isBreaking && (
              <Button>
                <SettingOutlined />
              </Button>
            )
          }
          suffixIcon={<IoIosArrowDown />}
        />
      </Form.Item>
    </div>
  )
}
