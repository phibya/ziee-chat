import { Drawer, Empty, Typography } from 'antd'

interface AddRAGDatabaseDownloadDrawerProps {
  open?: boolean
  onClose?: () => void
  providerId?: string
}

export function AddRAGDatabaseDownloadDrawer({ 
  open = false, 
  onClose
}: AddRAGDatabaseDownloadDrawerProps) {
  const handleClose = () => {
    onClose?.()
  }

  return (
    <Drawer
      title="Download RAG Database from Repository"
      width={600}
      open={open}
      onClose={handleClose}
    >
      <div style={{ marginBottom: 16 }}>
        <Typography.Text type="secondary">
          Select a RAG database from available repositories to download and add to your provider.
        </Typography.Text>
      </div>

      {/* TODO: Load and display available databases from repositories */}
      <Empty 
        description="No RAG databases available for download"
        image={Empty.PRESENTED_IMAGE_SIMPLE}
      />
    </Drawer>
  )
}