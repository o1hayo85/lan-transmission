import { List, Progress, Tag, Space, Button } from 'antd'
import { CloseOutlined } from '@ant-design/icons'
import { useTransferStore } from '../../stores/transferStore'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'antd'

const statusTexts: Record<string, string> = {
  pending: '等待中',
  waiting_accept: '等待确认',
  in_progress: '传输中',
  completed: '已完成',
  cancelled: '已取消',
  rejected: '已拒绝',
  failed: '失败',
}

const statusColors: Record<string, string> = {
  pending: 'default',
  waiting_accept: 'default',
  in_progress: 'processing',
  completed: 'success',
  cancelled: 'warning',
  rejected: 'error',
  failed: 'error',
}

function TransferProgress() {
  const { transfers, updateTransferStatus } = useTransferStore()
  const activeTransfers = transfers.filter(t =>
    t.status === 'in_progress' || t.status === 'waiting_accept' || t.status === 'pending'
  )

  if (activeTransfers.length === 0) {
    return <div style={{ color: '#999', textAlign: 'center', padding: 20 }}>暂无正在进行的传输</div>
  }

  const formatSize = (bytes: number) => {
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
    return (bytes / 1024 / 1024).toFixed(2) + ' MB'
  }

  const handleCancel = async (transferId: string) => {
    try {
      // 更新状态为已取消
      updateTransferStatus(transferId, 'cancelled')

      // 保存历史记录
      await invoke('update_transfer_status', {
        id: transferId,
        status: 'cancelled',
        transferredSize: 0
      })

      message.info('传输已取消')
    } catch (error) {
      message.error('取消传输失败')
    }
  }

  return (
    <List
      dataSource={activeTransfers}
      renderItem={(transfer) => (
        <List.Item>
          <div style={{ width: '100%' }}>
            <Space style={{ marginBottom: 8, width: '100%', justifyContent: 'space-between' }}>
              <Space>
                <Tag color={transfer.direction === 'send' ? 'blue' : 'green'}>
                  {transfer.direction === 'send' ? '发送' : '接收'}
                </Tag>
                <span>{transfer.peerDeviceName}</span>
                <span style={{ color: '#999' }}>
                  {(transfer.files?.length || 0)} 个文件
                </span>
              </Space>
              <Button
                size="small"
                icon={<CloseOutlined />}
                onClick={() => handleCancel(transfer.id)}
                danger
              >
                取消
              </Button>
            </Space>
            <Progress
              percent={transfer.totalSize > 0
                ? Math.round((transfer.transferredSize / transfer.totalSize) * 100)
                : 0
              }
              status={statusColors[transfer.status] === 'success' ? 'success' : 'active'}
              strokeColor={statusColors[transfer.status] === 'error' ? '#ff4d4f' : '#1890ff'}
            />
            <Space style={{ marginTop: 4, color: '#666' }}>
              <span>{formatSize(transfer.transferredSize)} / {formatSize(transfer.totalSize)}</span>
              <Tag color={statusColors[transfer.status]}>{statusTexts[transfer.status]}</Tag>
            </Space>
          </div>
        </List.Item>
      )}
    />
  )
}

export default TransferProgress