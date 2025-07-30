import React, { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Divider,
  Form,
  Input,
  Switch,
  Typography,
} from 'antd'
import { Drawer } from '../common/Drawer.tsx'
import { Assistant } from '../../types/api/assistant'
import {
  closeAssistantDrawer,
  createSystemAdminAssistant,
  createUserAssistant,
  setAssistantDrawerLoading,
  Stores,
  updateSystemAdminAssistant,
  updateUserAssistant,
} from '../../store'
import { ModelParameterField } from './ModelParameterField'
import { MODEL_PARAMETERS } from '../../constants/modelParameters'

const { Text } = Typography
const { TextArea } = Input

interface AssistantFormData {
  name: string
  description?: string
  instructions?: string
  parameters?: any
  is_default?: boolean
  is_active?: boolean
}

export const AssistantFormDrawer: React.FC = () => {
  const { t } = useTranslation()
  const [form] = Form.useForm<AssistantFormData>()

  // Store usage
  const { open, loading, editingAssistant, isAdmin, isCloning } =
    Stores.UI.AssistantDrawer

  // TODO: Handle clone source through store if needed
  const cloneSource: Assistant | null = null

  const handleSubmit = async (values: AssistantFormData) => {
    const finalValues = {
      ...values,
      is_template: isAdmin, // Set is_template based on admin status
    }

    setAssistantDrawerLoading(true)
    try {
      if (editingAssistant && !isCloning) {
        if (isAdmin) {
          await updateSystemAdminAssistant(editingAssistant.id, finalValues)
        } else {
          await updateUserAssistant(editingAssistant.id, finalValues)
        }
      } else {
        if (isAdmin) {
          await createSystemAdminAssistant(finalValues)
        } else {
          await createUserAssistant(finalValues)
        }
      }
      closeAssistantDrawer()
    } catch (error) {
      console.error('Failed to save assistant:', error)
    } finally {
      setAssistantDrawerLoading(false)
    }
  }

  // Initialize form when modal opens or editing assistant changes
  useEffect(() => {
    if (open) {
      if (editingAssistant) {
        // Editing existing assistant - set parameters as nested object
        form.setFieldsValue({
          name: editingAssistant.name,
          description: editingAssistant.description,
          instructions: editingAssistant.instructions,
          parameters: editingAssistant.parameters || {},
          is_active: editingAssistant.is_active,
          is_default: editingAssistant.is_default,
        })
      } else {
        // Creating new assistant with default values
        form.setFieldsValue({
          is_active: true,
          is_default: false,
          parameters: {},
        })
      }
    } else {
      // Reset when modal closes
      form.resetFields()
    }
  }, [open, editingAssistant, cloneSource, form])

  const getTitle = () => {
    if (editingAssistant && !isCloning) {
      return isAdmin ? 'Edit Template Assistant' : 'Edit Assistant'
    }
    return isAdmin ? 'Create Template Assistant' : 'Create Assistant'
  }

  return (
    <Drawer
      title={getTitle()}
      open={open}
      onClose={closeAssistantDrawer}
      footer={[
        <Button key="cancel" onClick={closeAssistantDrawer} disabled={loading}>
          Cancel
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={loading}
          onClick={() => form.submit()}
        >
          {editingAssistant && !isCloning ? 'Update' : 'Create'}
        </Button>,
      ]}
      width={500}
      maskClosable={false}
    >
      <Form form={form} onFinish={handleSubmit} layout="vertical">
        <Form.Item
          name="name"
          label={t('labels.name')}
          rules={[{ required: true, message: 'Please enter a name' }]}
        >
          <Input placeholder={t('forms.enterAssistantName')} />
        </Form.Item>

        <Form.Item name="description" label={t('labels.description')}>
          <Input.TextArea
            placeholder={t('forms.enterAssistantDescription')}
            rows={2}
          />
        </Form.Item>

        <Form.Item name="instructions" label={t('labels.instructions')}>
          <TextArea
            placeholder={t('forms.enterAssistantInstructions')}
            rows={6}
          />
        </Form.Item>

        <Divider orientation="left" plain>
          <Text strong>{t('labels.parameters')}</Text>
        </Divider>
        
        {MODEL_PARAMETERS.map((param, index) => (
          <ModelParameterField key={index} {...param} />
        ))}

        <Form.Item
          name="is_active"
          label={t('labels.active')}
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        <Form.Item
          name="is_default"
          label={t('labels.default')}
          valuePropName="checked"
          tooltip={
            isAdmin
              ? t('assistants.defaultTemplateTooltip')
              : t('assistants.defaultUserTooltip')
          }
        >
          <Switch />
        </Form.Item>
      </Form>
    </Drawer>
  )
}
