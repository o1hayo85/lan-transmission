import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'antd'
import { useTransferStore } from '../../stores/transferStore'

interface UploadProgressEvent {
  transfer_id: string
  file_id: string
  received_size: number
}

function ReceiveHandler() {
  const { transfers, updateTransferProgress, updateTransferStatus } = useTransferStore()

  useEffect(() => {
    // 监听上传进度，更新接收进度
    const unlistenProgress = listen<UploadProgressEvent>('upload-progress', async (event) => {
      const { transfer_id, received_size } = event.payload

      // 更新进度
      updateTransferProgress(transfer_id, received_size)

      // 查找传输记录检查是否完成
      const transfer = transfers.find(t => t.id === transfer_id)
      if (transfer && received_size >= transfer.totalSize) {
        updateTransferStatus(transfer_id, 'completed')
        message.success('文件接收完成')

        // 保存历史记录
        try {
          await invoke('save_transfer_record', {
            id: transfer_id,
            direction: 'receive',
            peerDeviceId: transfer.peerDeviceId,
            peerDeviceName: transfer.peerDeviceName,
            totalSize: transfer.totalSize
          })
          await invoke('update_transfer_status', {
            id: transfer_id,
            status: 'completed',
            transferredSize: received_size
          })
        } catch (e) {
          console.error('保存接收历史记录失败:', e)
        }
      }
    })

    return () => {
      unlistenProgress.then((fn) => fn())
    }
  }, [transfers, updateTransferProgress, updateTransferStatus])

  return null
}

export default ReceiveHandler