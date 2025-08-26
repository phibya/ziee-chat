import { Card, Typography } from 'antd'
import React from 'react'

const { Text } = Typography

export const RagQueryCard: React.FC = () => {
  return (
    <Card title="Query Interface">
      <div className="text-center">
        <div className="text-lg font-medium mb-2">
          RAG Query Interface
        </div>
        <Text type="secondary">
          Query interface will be implemented in Phase 4
        </Text>
      </div>
    </Card>
  )
}