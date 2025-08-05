import React, { useEffect } from 'react'
import { Button, Form, Input } from 'antd'
import { Drawer } from '../../common/Drawer.tsx'

const { TextArea } = Input

interface ProjectInstructionFormData {
  instruction: string
}

interface ProjectInstructionDrawerProps {
  open: boolean
  onClose: () => void
  onSave: (instruction: string) => Promise<void>
  currentInstruction?: string
  loading?: boolean
}

export const ProjectInstructionDrawer: React.FC<ProjectInstructionDrawerProps> = ({
  open,
  onClose,
  onSave,
  currentInstruction,
  loading = false,
}) => {
  const [form] = Form.useForm<ProjectInstructionFormData>()

  const handleSubmit = async (values: ProjectInstructionFormData) => {
    try {
      await onSave(values.instruction)
      onClose()
    } catch (error) {
      console.error('Failed to save instruction:', error)
    }
  }

  // Initialize form when drawer opens
  useEffect(() => {
    if (open) {
      form.setFieldsValue({
        instruction: currentInstruction || '',
      })
    } else {
      // Reset when drawer closes
      form.resetFields()
    }
  }, [open, currentInstruction, form])

  return (
    <Drawer
      title="Edit Project Instructions"
      open={open}
      onClose={onClose}
      footer={[
        <Button key="cancel" onClick={onClose} disabled={loading}>
          Cancel
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={() => form.submit()}
        >
          Save
        </Button>,
      ]}
      width={500}
      maskClosable={false}
    >
      <Form form={form} onFinish={handleSubmit} layout="vertical">
        <Form.Item
          name="instruction"
          label="Instructions"
          tooltip="Provide detailed instructions to guide AI conversations within this project"
        >
          <TextArea
            placeholder="Enter project instructions to guide AI conversations..."
            rows={12}
            showCount
            maxLength={2000}
          />
        </Form.Item>
      </Form>
    </Drawer>
  )
}