import { Card, Flex, Typography } from 'antd'
import React from 'react'

const { Text } = Typography

// Mock data for recent conversations
const mockConversations = [
  {
    id: '1',
    title: 'Academic Manuscript Cover Letter Revision',
    lastMessage: '9 hours ago',
  },
  {
    id: '2',
    title: 'Academic Paper Cover Letter LaTeX',
    lastMessage: '9 hours ago',
  },
  {
    id: '3',
    title: 'Reviewer Feedback: Extensibility Response',
    lastMessage: '1 day ago',
  },
  {
    id: '4',
    title: 'Scalability Analysis for Single-Cell Datasets',
    lastMessage: '4 days ago',
  },
  {
    id: '5',
    title: 'CytoAnalyst: Single-Cell Data Platform',
    lastMessage: '5 days ago',
  },
]

export const RecentConversationsSection: React.FC = () => {
  return (
    <Flex className={'flex-col gap-3'}>
      <Typography.Title level={5}>Recent Conversations</Typography.Title>
      <Flex vertical className={'gap-3 flex-1'}>
        {mockConversations.map(conv => (
          <Card key={conv.id} size="small" hoverable>
            <Text strong>{conv.title}</Text>
            <br />
            <Text type="secondary">Last message {conv.lastMessage}</Text>
          </Card>
        ))}
      </Flex>
    </Flex>
  )
}
