import React, { useEffect } from 'react'
import { Button, Form, Input } from 'antd'
import { Drawer } from '../../Common/Drawer.tsx'
import {
  closeProjectDrawer,
  createNewProject,
  createProjectStore,
  setProjectDrawerLoading,
  Stores,
} from '../../../store'

const { TextArea } = Input

interface ProjectFormData {
  name: string
  description?: string
  instruction?: string
}

export const ProjectFormDrawer: React.FC = () => {
  const [form] = Form.useForm<ProjectFormData>()

  // Store usage
  const { open, loading, editingProject } = Stores.UI.ProjectDrawer

  const handleSubmit = async (values: ProjectFormData) => {
    const finalValues = {
      ...values,
      description: values.description || '',
      instruction: values.instruction || '',
    }

    setProjectDrawerLoading(true)
    try {
      if (editingProject) {
        const projectStore = createProjectStore(editingProject.id)
        await projectStore.__state.updateProject(finalValues)
      } else {
        await createNewProject(finalValues)
      }
      closeProjectDrawer()
    } catch (error) {
      console.error('Failed to save project:', error)
    } finally {
      setProjectDrawerLoading(false)
    }
  }

  // Initialize form when drawer opens or editing project changes
  useEffect(() => {
    if (open) {
      if (editingProject) {
        // Editing existing project
        form.setFieldsValue({
          name: editingProject.name,
          description: editingProject.description,
        })
      } else {
        // Creating new project - no default values needed
      }
    } else {
      // Reset when drawer closes
      form.resetFields()
    }
  }, [open, editingProject, form])

  const getTitle = () => {
    return editingProject ? 'Edit Project' : 'Create Project'
  }

  return (
    <Drawer
      title={getTitle()}
      open={open}
      onClose={closeProjectDrawer}
      footer={[
        <Button key="cancel" onClick={closeProjectDrawer} disabled={loading}>
          Cancel
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={() => form.submit()}
        >
          {editingProject ? 'Update' : 'Create'}
        </Button>,
      ]}
      width={400}
      maskClosable={false}
    >
      <Form form={form} onFinish={handleSubmit} layout="vertical">
        <Form.Item
          name="name"
          label="Project Name"
          rules={[{ required: true, message: 'Please enter a project name' }]}
        >
          <Input placeholder="Enter project name" />
        </Form.Item>

        <Form.Item name="description" label="Description">
          <TextArea placeholder="Enter project description" rows={4} />
        </Form.Item>
      </Form>
    </Drawer>
  )
}
