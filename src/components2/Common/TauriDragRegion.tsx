import React from 'react'

interface TauriDragRegionProps extends React.HTMLAttributes<HTMLDivElement> {}

export const TauriDragRegion: React.FC<TauriDragRegionProps> = ({
  ...props
}) => {
  return <div data-tauri-drag-region={''} {...props} />
}
