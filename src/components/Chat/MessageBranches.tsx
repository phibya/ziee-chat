import { memo } from 'react'
import { Button, Spin } from 'antd'
import { LeftOutlined, RightOutlined } from '@ant-design/icons'
import { Message } from '../../types/api/chat'

interface MessageBranchesProps {
  branchInfo: {
    branches: Message[]
    currentIndex: number
    hasBranches: boolean
    isLoading: boolean
  }
  onLoadBranches: () => void
  onSwitchBranch: (messageId: string) => void
}

export const MessageBranches = memo(function MessageBranches({
  branchInfo,
  onLoadBranches,
  onSwitchBranch,
}: MessageBranchesProps) {
  if (!branchInfo.hasBranches && !branchInfo.isLoading) {
    return (
      <Button size="small" type="text" onClick={onLoadBranches}>
        <LeftOutlined />
      </Button>
    )
  }

  if (branchInfo.isLoading) {
    return <Spin size="small" />
  }

  if (branchInfo.hasBranches) {
    return (
      <>
        <Button
          size="small"
          type="text"
          icon={<LeftOutlined />}
          disabled={branchInfo.currentIndex <= 0}
          onClick={() => {
            const prevBranch = branchInfo.branches[branchInfo.currentIndex - 1]
            if (prevBranch) onSwitchBranch(prevBranch.id)
          }}
        />
        <div className="flex items-center text-sm text-gray-500">
          {branchInfo.currentIndex + 1} / {branchInfo.branches.length}
        </div>
        <Button
          size="small"
          type="text"
          icon={<RightOutlined />}
          disabled={branchInfo.currentIndex >= branchInfo.branches.length - 1}
          onClick={() => {
            const nextBranch = branchInfo.branches[branchInfo.currentIndex + 1]
            if (nextBranch) onSwitchBranch(nextBranch.id)
          }}
        />
      </>
    )
  }

  return null
})