import axios from 'axios'
import { Device } from '../types'

const getBaseUrl = (device: Device) => `http://${device.ip}:${device.port}/api`

// 发送传输请求
export async function sendTransferRequest(
  device: Device,
  files: any[],
  totalSize: number,
  myIp: string,
  myPort: number = 8080
): Promise<{ success: boolean; transferId?: string; error?: string }> {
  const transferId = generateTransferId()

  const request = {
    transfer_id: transferId,
    sender_id: localStorage.getItem('deviceId') || 'unknown',
    sender_name: localStorage.getItem('deviceName') || '本机设备',
    sender_ip: myIp,
    sender_port: myPort,
    files,
    total_size: totalSize
  }

  console.log('发送请求到:', `${getBaseUrl(device)}/transfer/request`)
  console.log('请求内容:', request)
  console.log('设备信息:', device)

  try {
    const response = await axios.post(`${getBaseUrl(device)}/transfer/request`, request, {
      timeout: 5000
    })
    console.log('响应:', response.data)
    return { success: true, transferId }
  } catch (error: any) {
    console.error('发送传输请求失败:', error)
    console.error('错误详情:', error.message, error.response?.data)
    return { success: false, error: error.message }
  }
}

// 上传文件
export async function uploadFile(
  device: Device,
  transferId: string,
  file: File,
  onProgress?: (progress: number) => void
): Promise<boolean> {
  const formData = new FormData()
  formData.append('transfer_id', transferId)
  formData.append('file_id', generateFileId())
  formData.append('file_name', file.name)
  formData.append('file', file)

  try {
    await axios.post(`${getBaseUrl(device)}/upload`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
      onUploadProgress: (progressEvent) => {
        if (onProgress && progressEvent.total) {
          const progress = Math.round((progressEvent.loaded / progressEvent.total) * 100)
          onProgress(progress)
        }
      }
    })
    return true
  } catch (error) {
    console.error('上传文件失败:', error)
    return false
  }
}

// 生成唯一ID
function generateTransferId(): string {
  return `transfer_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
}

function generateFileId(): string {
  return `file_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
}