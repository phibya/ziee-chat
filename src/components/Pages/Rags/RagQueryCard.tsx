import { Card, Typography } from 'antd'
import React from 'react'

const { Text } = Typography

export const RagQueryCard: React.FC = () => {
  return (
    <Card title="Query Interface">
      <div className="text-center">
        <Typography.Title level={5} className={'!m-0 !pt-[2px]'}>
          RAG Query Interface
        </Typography.Title>
        <Text type="secondary">
          Query interface will be implemented in Phase 4
        </Text>
      </div>
    </Card>
  )
}
