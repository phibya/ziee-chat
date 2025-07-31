import { CopyOutlined, PlusOutlined, RobotOutlined } from '@ant-design/icons'
import { App, Button, Card, Col, Flex, Row, Typography } from 'antd'
import React, { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  clearAssistantsStoreError,
  loadUserAssistants,
  openAssistantDrawer,
  Stores,
} from '../../../store'
import { PageContainer } from '../../common/PageContainer.tsx'
import { AssistantFormDrawer } from '../../common/AssistantFormDrawer.tsx'
import { isDesktopApp } from '../../../api/core.ts'
import { AssistantCard } from './index.ts'
import { TemplateAssistantDrawer } from './TemplateAssistantDrawer.tsx'

const { Title, Text } = Typography

export const AssistantsPage: React.FC = () => {
  const { t } = useTranslation()
  const { message } = App.useApp()

  // Assistants store
  const { assistants: allAssistants, loading, error } = Stores.Assistants

  const assistants = allAssistants.filter(a => !a.is_template)
  const templateAssistants = allAssistants.filter(a => a.is_template)

  const [templateModalVisible, setTemplateModalVisible] = useState(false)

  useEffect(() => {
    loadUserAssistants()
  }, [])

  // Show errors
  useEffect(() => {
    if (error) {
      message.error(error)
      clearAssistantsStoreError()
    }
  }, [error, message])

  const handleCreate = () => {
    openAssistantDrawer()
  }

  const handleCloneFromTemplate = () => {
    setTemplateModalVisible(true)
  }

  return (
    <PageContainer>
      <Row gutter={[24, 24]}>
        <Col span={24}>
          <div className="flex justify-between items-center mb-6">
            <div>
              <Title level={2}>{t('assistants.title')}</Title>
              <Text type="secondary">{t('assistants.subtitle')}</Text>
            </div>
            <Flex className="gap-2">
              {!isDesktopApp && (
                <Button
                  type="default"
                  icon={<CopyOutlined />}
                  onClick={handleCloneFromTemplate}
                >
                  Clone from Template
                </Button>
              )}
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={handleCreate}
              >
                Create New
              </Button>
            </Flex>
          </div>

          {loading ? (
            <div className="flex justify-center items-center py-12">
              <div>Loading assistants...</div>
            </div>
          ) : assistants.length === 0 ? (
            <Card>
              <div className="text-center py-12">
                <RobotOutlined className="text-4xl mb-4" />
                <Title level={4} type="secondary">
                  No assistants yet
                </Title>
                <Text type="secondary">
                  Create your first assistant to get started
                </Text>
              </div>
            </Card>
          ) : (
            <Row gutter={[16, 16]}>
              {assistants.map(assistant => (
                <Col xs={24} sm={12} md={8} lg={6} key={assistant.id}>
                  <AssistantCard assistant={assistant} />
                </Col>
              ))}
            </Row>
          )}
        </Col>
      </Row>

      <AssistantFormDrawer />

      <TemplateAssistantDrawer
        open={templateModalVisible}
        onClose={() => setTemplateModalVisible(false)}
        templateAssistants={templateAssistants}
      />
    </PageContainer>
  )
}
