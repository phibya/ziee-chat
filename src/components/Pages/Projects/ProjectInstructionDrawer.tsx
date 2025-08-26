import React, { useEffect, useState } from 'react'
import { App, Button, Form, Input, Typography } from 'antd'
import { useParams } from 'react-router-dom'
import { Drawer } from '../../common/Drawer.tsx'
import { useProjectStore } from '../../../store'

const { TextArea } = Input

interface ProjectInstructionFormData {
  instruction: string
}

interface ProjectInstructionDrawerProps {
  open: boolean
  onClose: () => void
}

export const ProjectInstructionDrawer: React.FC<
  ProjectInstructionDrawerProps
> = ({ open, onClose }) => {
  const { message } = App.useApp()
  const { projectId } = useParams<{ projectId: string }>()
  const [form] = Form.useForm<ProjectInstructionFormData>()
  const [loading, setLoading] = useState(false)
  
  // Get project store
  const { project, updateProject } = useProjectStore(projectId)

  const handleSubmit = async (values: ProjectInstructionFormData) => {
    if (!project) return

    setLoading(true)
    try {
      await updateProject({ instruction: values.instruction })
      message.success('Project instructions updated successfully')
      onClose()
    } catch (error) {
      console.error('Failed to update instruction:', error)
      message.error('Failed to update project instructions')
    } finally {
      setLoading(false)
    }
  }

  // Initialize form when drawer opens
  useEffect(() => {
    if (open) {
      form.setFieldsValue({
        instruction: project?.instruction || '',
      })
    } else {
      // Reset when drawer closes
      form.resetFields()
    }
  }, [open, project?.instruction, form])

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
      <div className="flex flex-col gap-2">
        <Typography.Text type={'secondary'}>
          Enter instructions to guide AI conversations for this project. These
          instructions will be used to provide context and help the AI
          understand the project better.
        </Typography.Text>

        <Form form={form} onFinish={handleSubmit} layout="vertical">
          <Form.Item name="instruction" label="Instructions" noStyle>
            <TextArea
              placeholder="Enter project instructions to guide AI conversations..."
              rows={12}
              showCount
            />
          </Form.Item>
        </Form>
      </div>
    </Drawer>
  )
}
