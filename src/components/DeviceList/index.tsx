import { useState } from 'react'
import { List, Avatar, Tag, Button, message, Space } from 'antd'
import { DesktopOutlined, FileOutlined, FileTextOutlined } from '@ant-design/icons'
import { useDeviceStore } from '../../stores/deviceStore'
import { SelectedItem } from '../FileDropZone'
import { invoke } from '@tauri-apps/api/core'
import { useTransferStore } from '../../stores/transferStore'
import './index.css'

interface DeviceListProps {
  fileList: SelectedItem[]
  textContent: string
  onSendComplete: () => void
}

interface SendTransferResult {
  success: boolean
  message: string
}

interface SendTextResult {
  success: boolean
  message: string
}

function DeviceList({ fileList, textContent, onSendComplete }: DeviceListProps) {
  const { devices } = useDeviceStore()
  const { addTransfer } = useTransferStore()
  const [sending, setSending] = useState(false)

  const sendFiles = async (device: { id: string; name: string; ip: string; port: number }) => {
    const transferId = `transfer_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const myIp = await invoke<string>('get_local_ip')
    const httpPort = await invoke<number>('get_http_port')

    const files = fileList.map(f => ({
      file_id: f.id,
      name: f.name,
      size: f.size,
      file_type: 'file',
      relative_path: f.relativePath || null
    }))

    const totalSize = fileList.reduce((sum, f) => sum + f.size, 0)

    addTransfer({
      id: transferId,
      direction: 'send',
      status: 'waiting_accept',
      peerDeviceId: device.id,
      peerDeviceName: device.name,
      totalSize: totalSize,
      transferredSize: 0,
      files: fileList.map(f => ({
        id: f.id,
        name: f.name,
        size: f.size,
        type: 'file',
        relativePath: f.relativePath,
        filePath: f.path
      })),
      createdAt: Date.now()
    })

    const result = await invoke<SendTransferResult>('send_transfer_request', {
      deviceIp: device.ip,
      devicePort: device.port,
      transferId: transferId,
      senderName: '发送方',
      senderIp: myIp,
      senderPort: httpPort,
      files: files,
      totalSize: totalSize
    })

    return result
  }

  const sendText = async (device: { id: string; name: string; ip: string; port: number }) => {
    const transferId = `text_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const myIp = await invoke<string>('get_local_ip')
    const httpPort = await invoke<number>('get_http_port')

    const result = await invoke<SendTextResult>('send_text_request', {
      deviceIp: device.ip,
      devicePort: device.port,
      transferId: transferId,
      senderName: '本机',
      senderIp: myIp,
      senderPort: httpPort,
      textContent: textContent
    })

    return result
  }

  const doSendFiles = async (device: { id: string; name: string; ip: string; port: number }) => {
    if (fileList.length === 0) {
      message.warning('请先选择要发送的文件')
      return
    }
    setSending(true)
    try {
      const result = await sendFiles(device)
      if (result.success) {
        message.success(`已向 ${device.name} 发送传输请求`)
        onSendComplete()
      } else {
        message.error(`发送请求失败: ${result.message}`)
      }
    } catch {
      message.error(`发送请求失败: ${device.name} 可能已离线`)
    }
    setSending(false)
  }

  const doSendText = async (device: { id: string; name: string; ip: string; port: number }) => {
    if (textContent.trim().length === 0) {
      message.warning('请先输入要发送的文本内容')
      return
    }
    setSending(true)
    try {
      const result = await sendText(device)
      if (result.success) {
        message.success(`文本已发送到 ${device.name}`)
        onSendComplete()
      } else {
        message.error(`发送失败: ${result.message}`)
      }
    } catch {
      message.error(`发送失败: ${device.name} 可能已离线`)
    }
    setSending(false)
  }

  return (
    <List
      dataSource={devices.filter(d => d.isOnline)}
      renderItem={(device) => (
        <List.Item className="device-item">
          <div className="device-content">
            <Avatar icon={<DesktopOutlined />} />
            <div className="device-info">
              <span className="device-name" title={device.name}>{device.name}</span>
              <span className="device-ip">{device.ip}:{device.port}</span>
            </div>
          </div>
          <div className="device-actions">
            <Tag color="green">在线</Tag>
            <Space>
              <Button
                type="primary"
                size="small"
                icon={<FileOutlined />}
                onClick={() => doSendFiles(device)}
                loading={sending}
                disabled={fileList.length === 0}
              >
                发送文件
              </Button>
              <Button
                size="small"
                icon={<FileTextOutlined />}
                onClick={() => doSendText(device)}
                loading={sending}
                disabled={textContent.trim().length === 0}
              >
                发送文本
              </Button>
            </Space>
          </div>
        </List.Item>
      )}
    />
  )
}

export default DeviceList