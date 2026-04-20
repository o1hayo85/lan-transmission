import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useTransferStore } from '../stores/transferStore'
import { useDeviceStore } from '../stores/deviceStore'
import { message } from 'antd'

export function useTransferListener() {
  const { devices } = useDeviceStore()
  const { transfers, updateTransferStatus, updateTransferProgress, removeTransfer } = useTransferStore()

  useEffect(() => {
    // 监听传输开始事件（接收方接受了请求）
    const unlistenStarted = listen<string>('transfer-started', async (event) => {
      const transferId = event.payload

      // 找到对应的传输记录
      const transfer = transfers.find(t => t.id === transferId)
      if (!transfer || transfer.direction !== 'send') return

      // 找到目标设备
      const device = devices.find(d => d.id === transfer.peerDeviceId)
      if (!device) {
        message.error('目标设备不存在')
        return
      }

      // 开始上传文件
      updateTransferStatus(transferId, 'in_progress')

      let totalUploaded = 0
      for (const fileInfo of transfer.files) {
        // 获取文件对象（这里需要从之前的文件列表获取）
        // 暂时模拟上传进度
        message.info(`正在上传: ${fileInfo.name}`)
        totalUploaded += fileInfo.size
        updateTransferProgress(transferId, totalUploaded)
      }

      updateTransferStatus(transferId, 'completed')
      message.success('文件传输完成')
    })

    // 监听传输被拒绝事件
    const unlistenRejected = listen<string>('transfer-rejected', (event) => {
      const transferId = event.payload
      updateTransferStatus(transferId, 'rejected')
      message.info('对方拒绝了传输请求')
    })

    // 监听传输取消事件
    const unlistenCancelled = listen<string>('transfer-cancelled', (event) => {
      const transferId = event.payload
      updateTransferStatus(transferId, 'cancelled')
      message.info('传输已取消')
    })

    // 监听上传进度事件
    const unlistenProgress = listen<{ transfer_id: string; received_size: number }>('upload-progress', (event) => {
      const { transfer_id, received_size } = event.payload
      updateTransferProgress(transfer_id, received_size)
    })

    return () => {
      unlistenStarted.then((fn) => fn())
      unlistenRejected.then((fn) => fn())
      unlistenCancelled.then((fn) => fn())
      unlistenProgress.then((fn) => fn())
    }
  }, [transfers, devices, updateTransferStatus, updateTransferProgress, removeTransfer])
}