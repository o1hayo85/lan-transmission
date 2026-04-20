import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'antd'
import { useTransferStore } from '../../stores/transferStore'

interface TransferAcceptedEvent {
  transfer_id: string
  receiver_ip: string
  receiver_port: number
  save_path: string
}

interface UploadResult {
  success: boolean
  message: string
  transferred_size: number
}

interface UploadStatusResult {
  success: boolean
  exists: boolean
  received_size: number
}

const MAX_RETRIES = 3
const RETRY_DELAY = 1000

function UploadHandler() {
  const { transfers, updateTransferStatus, updateTransferProgress } = useTransferStore()

  useEffect(() => {
    const unlisten = listen<TransferAcceptedEvent>('transfer-accepted', async (event) => {
      const { transfer_id, receiver_ip, receiver_port } = event.payload

      // 查找对应的传输记录
      const transfer = transfers.find(t => t.id === transfer_id)
      if (!transfer) {
        message.error('找不到传输记录')
        return
      }

      // 更新状态为上传中
      updateTransferStatus(transfer_id, 'in_progress')

      // 获取文件列表
      const fileList = transfer.files || []
      let totalTransferred = 0

      // 逐个上传文件
      for (const file of fileList) {
        let retries = 0
        let success = false

        while (retries < MAX_RETRIES && !success) {
          try {
            const filePath = (file as any).file_path || (file as any).path
            const fileId = file.id || (file as any).file_id
            const relativePath = (file as any).relative_path || file.relativePath || null

            // 查询已接收大小（断点续传）
            let offset = 0
            try {
              const statusResult = await invoke<UploadStatusResult>('query_upload_status', {
                receiverIp: receiver_ip,
                receiverPort: receiver_port,
                transferId: transfer_id,
                fileName: file.name,
                relativePath: relativePath || ''
              })
              if (statusResult.exists) {
                offset = statusResult.received_size
                console.log(`断点续传: ${file.name} 已接收 ${offset} 字节`)
              }
            } catch (e) {
              // 查询失败则从头开始
              offset = 0
            }

            // 使用 Tauri 命令上传文件（支持断点续传）
            const result = await invoke<UploadResult>('upload_file_to_receiver', {
              filePath: filePath,
              transferId: transfer_id,
              fileId: fileId,
              fileName: file.name,
              relativePath: relativePath,
              receiverIp: receiver_ip,
              receiverPort: receiver_port,
              offset: offset
            })

            if (result.success) {
              success = true
              totalTransferred += result.transferred_size
              updateTransferProgress(transfer_id, totalTransferred)
            } else {
              retries++
              if (retries < MAX_RETRIES) {
                message.warning(`上传文件 ${file.name} 失败，正在重试 (${retries}/${MAX_RETRIES})`)
                await new Promise(resolve => setTimeout(resolve, RETRY_DELAY))
              } else {
                throw new Error(result.message)
              }
            }

          } catch (error: any) {
            retries++
            if (retries >= MAX_RETRIES) {
              message.error(`上传文件 ${file.name} 失败: ${error.message}`)
              updateTransferStatus(transfer_id, 'failed')
              return
            }
            await new Promise(resolve => setTimeout(resolve, RETRY_DELAY))
          }
        }
      }

      // 所有文件上传完成
      updateTransferStatus(transfer_id, 'completed')
      updateTransferProgress(transfer_id, transfer.totalSize)
      message.success('所有文件已成功发送')

      // 保存历史记录到数据库
      try {
        await invoke('save_transfer_record', {
          id: transfer_id,
          direction: 'send',
          peerDeviceId: transfer.peerDeviceId,
          peerDeviceName: transfer.peerDeviceName,
          totalSize: transfer.totalSize
        })
        await invoke('update_transfer_status', {
          id: transfer_id,
          status: 'completed',
          transferredSize: transfer.totalSize
        })
      } catch (e) {
        console.error('保存历史记录失败:', e)
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [transfers, updateTransferStatus, updateTransferProgress])

  return null
}

export default UploadHandler