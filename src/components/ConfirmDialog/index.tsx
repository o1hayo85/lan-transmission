import { useState, useEffect, useRef } from 'react'
import { Modal, Button, List, Typography, Space, message, Input } from 'antd'
import { FolderOpenOutlined } from '@ant-design/icons'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useTransferStore } from '../../stores/transferStore'
import { useSettingsStore } from '../../stores/settingsStore'

const { Text } = Typography

// 后端发送的文件信息格式
interface BackendFileInfo {
  file_id: string
  name: string
  size: number
  file_type: string
  relative_path?: string
}

interface IncomingRequest {
  transfer_id: string
  sender_id: string
  sender_name: string
  sender_ip: string
  sender_port: number
  files: BackendFileInfo[]
  total_size: number
}

interface SendTransferResult {
  success: boolean
  message: string
}

function ConfirmDialog() {
  const [currentRequest, setCurrentRequest] = useState<IncomingRequest | null>(null)
  const [savePath, setSavePath] = useState<string>('')
  const { addTransfer } = useTransferStore()
  const { defaultSavePath, loadSettings, validatePath } = useSettingsStore()

  // 使用 ref 跟踪最新的 defaultSavePath，避免竞态条件
  const defaultSavePathRef = useRef(defaultSavePath)

  // 更新 ref 值
  useEffect(() => {
    defaultSavePathRef.current = defaultSavePath
  }, [defaultSavePath])

  // 加载默认保存路径设置
  useEffect(() => {
    loadSettings()
  }, [loadSettings])

  // 监听传输请求事件（只设置一次，使用 ref 获取最新值）
  useEffect(() => {
    const unlisten = listen<IncomingRequest>('transfer-request', (event) => {
      console.log('收到传输请求:', event.payload)
      setCurrentRequest(event.payload)
      // 使用 ref 获取最新的默认保存路径
      setSavePath(defaultSavePathRef.current)
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

    // 验证保存路径有效性
    const validation = await validatePath(savePath)
    if (!validation.valid) {
      message.error(`保存路径无效: ${validation.error}`)
      return
    }

    try {
      // 首先在本地存储save_path（接收方的HTTP服务器需要这个路径来保存文件）
      const localResult = await invoke<SendTransferResult>('save_transfer_path_locally', {
        transferId: currentRequest.transfer_id,
        savePath: savePath
      })

      if (!localResult.success) {
        message.error(`本地存储路径失败: ${localResult.message}`)
        return
      }

      // 保存传输记录到数据库（必须先保存，否则上传时文件记录的外键约束会失败）
      await invoke('save_transfer_record', {
        id: currentRequest.transfer_id,
        direction: 'receive',
        peerDeviceId: currentRequest.sender_id,
        peerDeviceName: currentRequest.sender_name,
        peerIp: currentRequest.sender_ip,
        totalSize: currentRequest.total_size
      })

      // 获取本机IP作为接收方IP
      const myIp = await invoke<string>('get_local_ip')

      // 获取动态HTTP端口
      const httpPort = await invoke<number>('get_http_port')

      // 使用后端命令发送接受请求到发送方
      const result = await invoke<SendTransferResult>('send_accept_request', {
        senderIp: currentRequest.sender_ip,
        senderPort: currentRequest.sender_port,
        transferId: currentRequest.transfer_id,
        savePath: savePath,
        receiverIp: myIp,
        receiverPort: httpPort
      })

      if (!result.success) {
        message.error(`接受请求失败: ${result.message}`)
        return
      }

      // 创建传输记录
      addTransfer({
        id: currentRequest.transfer_id,
        direction: 'receive',
        status: 'in_progress',
        peerDeviceId: currentRequest.sender_id,
        peerDeviceName: currentRequest.sender_name,
        totalSize: currentRequest.total_size,
        transferredSize: 0,
        files: currentRequest.files.map(f => ({
          id: f.file_id,
          name: f.name,
          size: f.size,
          type: f.file_type
        })),
        createdAt: Date.now()
      })

      message.success('已接受传输请求，文件将保存到: ' + savePath)
      setCurrentRequest(null)
    } catch (error) {
      console.error('接受传输请求失败:', error)
      message.error('接受传输请求失败')
    }
  }

  const handleReject = async () => {
    if (!currentRequest) return

    try {
      // 使用后端命令发送拒绝请求
      await invoke<SendTransferResult>('send_reject_request', {
        senderIp: currentRequest.sender_ip,
        senderPort: currentRequest.sender_port,
        transferId: currentRequest.transfer_id
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