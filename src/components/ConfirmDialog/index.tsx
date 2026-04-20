import { useState, useEffect } from 'react'
import { Modal, Button, List, Typography, Space, message, Input } from 'antd'
import { FolderOpenOutlined } from '@ant-design/icons'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import axios from 'axios'
import { useTransferStore } from '../../stores/transferStore'
import { FileInfo } from '../../types'

const { Text } = Typography

interface IncomingRequest {
  transfer_id: string
  sender_id: string
  sender_name: string
  sender_ip: string
  sender_port: number
  files: FileInfo[]
  total_size: number
}

function ConfirmDialog() {
  const [currentRequest, setCurrentRequest] = useState<IncomingRequest | null>(null)
  const [savePath, setSavePath] = useState<string>('')
  const { addTransfer } = useTransferStore()

  // 监听传输请求事件
  useEffect(() => {
    const unlisten = listen<IncomingRequest>('transfer-request', (event) => {
      setCurrentRequest(event.payload)
      // 默认保存路径为空，用户需要选择
      setSavePath('')
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
    return (bytes / 1024 / 1024).toFixed(2) + ' MB'
  }

  // 选择保存文件夹
  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择保存位置'
      })
      if (selected) {
        setSavePath(selected as string)
      }
    } catch (error) {
      message.error('选择文件夹失败')
    }
  }

  const handleAccept = async () => {
    if (!currentRequest) return

    if (!savePath) {
      message.warning('请先选择保存文件夹')
      return
    }

    try {
      // 获取本机IP作为接收方IP
      const myIp = await invoke<string>('get_local_ip')

      // 发送接受请求到发送方，包含接收方地址信息
      await axios.post(`http://${currentRequest.sender_ip}:${currentRequest.sender_port}/api/transfer/accept`, {
        transfer_id: currentRequest.transfer_id,
        save_path: savePath,
        receiver_ip: myIp,
        receiver_port: 8080
      })

      // 创建传输记录
      addTransfer({
        id: currentRequest.transfer_id,
        direction: 'receive',
        status: 'in_progress',
        peerDeviceId: currentRequest.sender_id,
        peerDeviceName: currentRequest.sender_name,
        totalSize: currentRequest.total_size,
        transferredSize: 0,
        files: currentRequest.files,
        createdAt: Date.now()
      })

      message.success('已接受传输请求，文件将保存到: ' + savePath)
      setCurrentRequest(null)
    } catch (error) {
      message.error('接受传输请求失败')
    }
  }

  const handleReject = async () => {
    if (!currentRequest) return

    try {
      await axios.post(`http://${currentRequest.sender_ip}:${currentRequest.sender_port}/api/transfer/reject`, {
        transfer_id: currentRequest.transfer_id
      })
    } catch (error) {
      // 忽略拒绝失败
    }

    message.info('已拒绝传输请求')
    setCurrentRequest(null)
  }

  if (!currentRequest) return null

  return (
    <Modal
      open={true}
      title="接收文件请求"
      onCancel={handleReject}
      footer={
        <Space>
          <Button onClick={handleReject}>拒绝</Button>
          <Button type="primary" onClick={handleAccept} disabled={!savePath}>
            接收
          </Button>
        </Space>
      }
    >
      <Text>{currentRequest.sender_name} 想要发送以下文件：</Text>

      <List
        style={{ maxHeight: 200, overflow: 'auto', marginBottom: 16 }}
        dataSource={currentRequest.files.slice(0, 10)}
        renderItem={(file) => (
          <List.Item>
            <Space>
              <Text>{file.name}</Text>
              <Text type="secondary">{formatSize(file.size)}</Text>
            </Space>
          </List.Item>
        )}
        footer={
          currentRequest.files.length > 10 && (
            <span style={{ color: '#999' }}>还有 {currentRequest.files.length - 10} 个文件...</span>
          )
        }
      />

      <Text strong>总大小: {formatSize(currentRequest.total_size)}</Text>

      <div style={{ marginTop: 16 }}>
        <Text>保存位置:</Text>
        <Space.Compact style={{ width: '100%', marginTop: 8 }}>
          <Input
            value={savePath}
            placeholder="请选择保存文件夹"
            readOnly
          />
          <Button icon={<FolderOpenOutlined />} onClick={handleSelectFolder}>
            选择文件夹
          </Button>
        </Space.Compact>
      </div>
    </Modal>
  )
}

export default ConfirmDialog