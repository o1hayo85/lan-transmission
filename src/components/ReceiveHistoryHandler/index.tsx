import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'antd'

interface UploadProgressEvent {
  transfer_id: string
  file_id: string
  received_size: number
}

interface TransferRequestEvent {
  transfer_id: string
  sender_id: string
  sender_name: string
  sender_ip: string
  sender_port: number
  files: Array<{
    file_id: string
    name: string
    size: number
    file_type: string
    relative_path?: string
  }>
  total_size: number
}

// 存储当前接收的传输信息
const pendingReceives: Map<string, TransferRequestEvent> = new Map()

function ReceiveHistoryHandler() {
  useEffect(() => {
    // 监听传输请求事件，存储接收信息
    const unlistenRequest = listen<TransferRequestEvent>('transfer-request', (event) => {
      pendingReceives.set(event.payload.transfer_id, event.payload)
    })

    // 监听上传进度，检测接收完成
    const unlistenProgress = listen<UploadProgressEvent>('upload-progress', async (event) => {
      const { transfer_id, received_size } = event.payload
      const request = pendingReceives.get(transfer_id)

      if (request) {
        // 检查是否接收完成
        if (received_size >= request.total_size) {
          // 保存历史记录
          try {
            await invoke('save_transfer_record', {
              id: transfer_id,
              direction: 'receive',
              peerDeviceId: request.sender_id,
              peerDeviceName: request.sender_name,
              totalSize: request.total_size
            })
            await invoke('update_transfer_status', {
              id: transfer_id,
              status: 'completed',
              transferredSize: received_size
            })
            message.success('文件接收完成')
            pendingReceives.delete(transfer_id)
          } catch (e) {
            console.error('保存接收历史记录失败:', e)
          }
        }
      }
    })

    return () => {
      unlistenRequest.then((fn) => fn())
      unlistenProgress.then((fn) => fn())
    }
  }, [])

  return null
}

export default ReceiveHistoryHandler