import { useEffect, useRef } from 'react'
import { notification, Progress, Tag, Space } from 'antd'
import { ArrowUpOutlined, ArrowDownOutlined } from '@ant-design/icons'
import { useTransferStore } from '../../stores/transferStore'
import type { TransferRecord } from '../../types'

const COMPLETED_DURATION = 5 // 完成后停留秒数

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

// 格式化文件大小 - 提取到组件外部
const formatSize = (bytes: number) => {
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / 1024 / 1024).toFixed(2) + ' MB'
}

// 获取当前正在传输的文件名 - 提取到组件外部
const getCurrentFileName = (transfer: TransferRecord): string | undefined => {
  if (!transfer.files || transfer.files.length === 0) return undefined

  let accumulatedSize = 0
  for (const file of transfer.files) {
    accumulatedSize += file.size
    if (accumulatedSize > transfer.transferredSize) {
      return file.name
    }
  }

  return transfer.files[transfer.files.length - 1]?.name
}

function TransferProgress() {
  const { transfers } = useTransferStore()

  // 记录已打开的通知ID
  const openedNotifications = useRef<Set<string>>(new Set())

  useEffect(() => {
    // 过滤需要显示通知的传输
    const activeTransfers = transfers.filter(t =>
      t.status === 'in_progress' ||
      t.status === 'waiting_accept' ||
      t.status === 'pending' ||
      t.status === 'completed' ||
      t.status === 'failed' ||
      t.status === 'cancelled' ||
      t.status === 'rejected'
    )

    activeTransfers.forEach(transfer => {
      const { id, status, direction, peerDeviceName, files, totalSize, transferredSize } = transfer

      // 计算进度百分比
      const percent = totalSize > 0
        ? Math.round((transferredSize / totalSize) * 100)
        : 0

      // 判断是否完成或出错
      const isCompleted = status === 'completed'
      const isActive = status === 'in_progress'
      const isError = ['failed', 'cancelled', 'rejected'].includes(status)

      // 获取当前文件名
      const currentFileName = isActive ? getCurrentFileName(transfer) : undefined

      // 通知内容
      const description = (
        <div style={{ minWidth: 280 }}>
          {/* 第一行：方向 + 设备名 + 文件数量 */}
          <Space style={{ marginBottom: 8 }}>
            {direction === 'send' ? (
              <ArrowUpOutlined style={{ color: '#1890ff' }} />
            ) : (
              <ArrowDownOutlined style={{ color: '#52c41a' }} />
            )}
            <span>{peerDeviceName}</span>
            <span style={{ color: '#999' }}>
              {files?.length || 0} 个文件
            </span>
          </Space>

          {/* 第二行：当前文件名（仅传输中显示） */}
          {currentFileName && (
            <div style={{ marginBottom: 8, color: '#666', fontSize: 13, maxWidth: 280, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              正在传输: {currentFileName}
            </div>
          )}

          {/* 进度条 */}
          <Progress
            percent={percent}
            size="small"
            status={isError ? 'exception' : (isCompleted ? 'success' : 'active')}
          />

          {/* 第四行：大小 + 状态 */}
          <Space style={{ marginTop: 4, width: '100%', justifyContent: 'space-between' }}>
            <span style={{ color: '#999', fontSize: 12 }}>
              {formatSize(transferredSize)} / {formatSize(totalSize)}
            </span>
            <Tag color={statusColors[status]}>{statusTexts[status]}</Tag>
          </Space>
        </div>
      )

      // 通知标题
      const message = (
        <Space>
          <Tag color={direction === 'send' ? 'blue' : 'green'}>
            {direction === 'send' ? '发送' : '接收'}
          </Tag>
        </Space>
      )

      // 显示时长：完成/失败后5秒消失，进行中不自动关闭
      const duration = (isCompleted || isError) ? COMPLETED_DURATION : 0

      // 打开或更新通知
      notification.open({
        key: id,
        message,
        description,
        placement: 'topRight',
        duration,
        closable: true,
        onClose: () => {
          // 仅关闭通知，不取消传输
          openedNotifications.current.delete(id)
        },
        style: {
          width: 320,
        }
      })

      // 记录已打开的通知
      openedNotifications.current.add(id)
    })

    // 清理不再活跃的传输的通知
    openedNotifications.current.forEach(key => {
      const stillActive = activeTransfers.some(t => t.id === key)
      if (!stillActive) {
        notification.destroy(key)
        openedNotifications.current.delete(key)
      }
    })
  }, [transfers])

  // 组件卸载时关闭所有通知
  useEffect(() => {
    return () => {
      openedNotifications.current.forEach(key => {
        notification.destroy(key)
      })
      openedNotifications.current.clear()
    }
  }, [])

  return null // 不渲染任何内容，通知由antd渲染
}

export default TransferProgress