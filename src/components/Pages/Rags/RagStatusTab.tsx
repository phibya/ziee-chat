import React from 'react'
import { RagInstanceStatus } from './RagInstanceStatus.tsx'

export const RagStatusTab: React.FC = () => {
  return (
    <div className="flex flex-col gap-3">
      <RagInstanceStatus />
    </div>
  )
}