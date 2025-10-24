import { CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons'
import { Alert, Button, Modal, Result, Spin } from 'antd'
import type { AuthProvider, TestConnectionResult } from '../../../../types'

interface TestConnectionModalProps {
  open: boolean
  onClose: () => void
  provider: AuthProvider | null
  testResult: TestConnectionResult | null
  testing: boolean
}

export function TestConnectionModal({
  open,
  onClose,
  provider,
  testResult,
  testing,
}: TestConnectionModalProps) {
  return (
    <Modal
      title={`Test Connection: ${provider?.name || ''}`}
      open={open}
      onCancel={onClose}
      footer={
        <Button type="primary" onClick={onClose}>
          Close
        </Button>
      }
      width={500}
    >
      {testing ? (
        <div className="flex justify-center py-8">
          <Spin size="large" tip="Testing connection..." />
        </div>
      ) : testResult ? (
        <div>
          <Result
            status={testResult.success ? 'success' : 'error'}
            title={
              testResult.success ? 'Connection Successful' : 'Connection Failed'
            }
            subTitle={testResult.message || ''}
            icon={
              testResult.success ? (
                <CheckCircleOutlined style={{ color: '#52c41a' }} />
              ) : (
                <CloseCircleOutlined style={{ color: '#ff4d4f' }} />
              )
            }
          />
          {testResult.details && (
            <Alert
              message="Details"
              description={
                <pre className="text-xs whitespace-pre-wrap">
                  {JSON.stringify(testResult.details, null, 2)}
                </pre>
              }
              type="info"
              showIcon
              className="mt-4"
            />
          )}
        </div>
      ) : (
        <div className="text-center py-4">
          <p>Click "Test" to test the provider connection</p>
        </div>
      )}
    </Modal>
  )
}
