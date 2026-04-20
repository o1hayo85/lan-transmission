import { useState } from 'react'
import { Button, List, Space, message, Select, Spin } from 'antd'
import { InboxOutlined, SendOutlined, DeleteOutlined, FolderOpenOutlined, FileAddOutlined } from '@ant-design/icons'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useDeviceStore } from '../../stores/deviceStore'
import { useTransferStore } from '../../stores/transferStore'
import { sendTransferRequest } from '../../services/transfer'

interface FileEntry {
  id: string
  name: string
  path: string
  size: number
  relative_path: string
  is_dir: boolean
}

interface SelectedItem {
  id: string
  name: string
  path: string
  size: number
  relativePath?: string
  isFolder?: boolean
}

function FileDropZone() {
  const { devices } = useDeviceStore()
  const { addTransfer } = useTransferStore()
  const [fileList, setFileList] = useState<SelectedItem[]>([])
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null)
  const [sending, setSending] = useState(false)
  const [loading, setLoading] = useState(false)

  // 计算总大小
  const totalSize = fileList.reduce((sum, f) => sum + f.size, 0)

  // 格式化文件大小
  const formatSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
    if (bytes < 1024 * 1024 * 1024) return (bytes / 1024 / 1024).toFixed(2) + ' MB'
    return (bytes / 1024 / 1024 / 1024).toFixed(2) + ' GB'
  }

  // 选择单个或多个文件
  const handleSelectFiles = async () => {
    try {
      const selected = await open({
        multiple: true,
        title: '选择文件'
      })
      if (selected) {
        setLoading(true)
        const paths = Array.isArray(selected) ? selected : [selected]
        const newFiles: SelectedItem[] = []

        for (const path of paths) {
          const name = path.split(/[\\/]/).pop() || 'unknown'
          const size = await invoke<number>('get_file_size', { filePath: path })
          newFiles.push({
            id: `file_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            name,
            path,
            size,
          })
        }

        setFileList(prev => [...prev, ...newFiles])
        setLoading(false)
      }
    } catch (error) {
      setLoading(false)
      message.error('选择文件失败')
    }
  }

  // 选择文件夹（遍历获取所有文件）
  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择文件夹'
      })
      if (selected) {
        setLoading(true)
        const folderPath = selected as string
        const folderName = folderPath.split(/[\\/]/).pop() || 'unknown'

        // 遍历文件夹获取所有文件
        const files = await invoke<FileEntry[]>('list_folder_files', { folderPath })

        const selectedFiles: SelectedItem[] = files.map(f => ({
          id: f.id,
          name: f.name,
          path: f.path,
          size: f.size,
          relativePath: f.relative_path,
          isFolder: false
        }))

        if (selectedFiles.length > 0) {
          // 添加一个文件夹标记项
          setFileList(prev => [...prev, {
            id: `folder_${Date.now()}`,
            name: `${folderName} (${selectedFiles.length} 个文件)`,
            path: folderPath,
            size: selectedFiles.reduce((sum, f) => sum + f.size, 0),
            isFolder: true,
            relativePath: folderName
          }, ...selectedFiles])
        } else {
          message.warning('文件夹内没有文件')
        }
        setLoading(false)
      }
    } catch (error) {
      setLoading(false)
      message.error('选择文件夹失败')
    }
  }

  // 发送文件
  const handleSend = async () => {
    if (!selectedDevice) {
      message.warning('请先选择目标设备')
      return
    }
    if (fileList.length === 0) {
      message.warning('请先选择要发送的文件')
      return
    }

    const device = devices.find(d => d.id === selectedDevice)
    if (!device) {
      message.error('目标设备不存在')
      return
    }

    setSending(true)

    try {
      // 获取本机IP
      const myIp = await invoke<string>('get_local_ip')

      // 过滤掉文件夹标记项，只发送实际文件
      const actualFiles = fileList.filter(f => !f.isFolder)

      // 构建文件信息列表
      const files = actualFiles.map(f => ({
        file_id: f.id,
        name: f.name,
        size: f.size,
        file_type: 'application/octet-stream',
        relative_path: f.relativePath,
        file_path: f.path,
      }))

      const actualTotalSize = actualFiles.reduce((sum, f) => sum + f.size, 0)

      // 发送传输请求
      const result = await sendTransferRequest(device, files, actualTotalSize, myIp)

      if (result.success && result.transferId) {
        message.success('传输请求已发送，等待对方确认')

        // 创建传输记录
        addTransfer({
          id: result.transferId,
          direction: 'send',
          status: 'waiting_accept',
          peerDeviceId: device.id,
          peerDeviceName: device.name,
          totalSize: actualTotalSize,
          transferredSize: 0,
          files: files as any,
          createdAt: Date.now()
        })

        // 清空文件列表
        setFileList([])
      } else {
        message.error('发送传输请求失败: ' + (result.error || '未知错误'))
      }
    } catch (error: any) {
      message.error('获取本机IP失败: ' + error.message)
    }

    setSending(false)
  }

  // 清空文件列表
  const handleClear = () => {
    setFileList([])
  }

  // 显示列表（合并文件夹项和文件项，最多显示5个）
  const displayList = fileList.slice(0, 5)

  return (
    <div>
      <div style={{ padding: '40px 0', textAlign: 'center', border: '1px dashed #d9d9d9', borderRadius: 8, background: '#fafafa' }}>
        <InboxOutlined style={{ fontSize: 48, color: '#1890ff' }} />
        <p style={{ marginTop: 16, color: '#666' }}>点击下方按钮选择文件</p>
      </div>

      <div style={{ marginTop: 16, textAlign: 'center' }}>
        <Button icon={<FileAddOutlined />} onClick={handleSelectFiles} style={{ marginRight: 8 }} disabled={loading}>
          选择文件
        </Button>
        <Button icon={<FolderOpenOutlined />} onClick={handleSelectFolder} disabled={loading}>
          选择文件夹
        </Button>
        {loading && <Spin style={{ marginLeft: 16 }} />}
      </div>

      {fileList.length > 0 && (
        <div style={{ marginTop: 16 }}>
          <List
            header={`已选择 ${fileList.filter(f => !f.isFolder).length} 个文件，总大小: ${formatSize(totalSize)}`}
            dataSource={displayList}
            renderItem={(file) => (
              <List.Item>
                <Space>
                  <span style={{ color: file.isFolder ? '#1890ff' : 'inherit', fontWeight: file.isFolder ? 'bold' : 'normal' }}>
                    {file.name}
                  </span>
                  <span style={{ color: '#999' }}>{formatSize(file.size)}</span>
                </Space>
              </List.Item>
            )}
            footer={
              fileList.length > 5 && (
                <span style={{ color: '#999' }}>还有 {fileList.length - 5} 个项目...</span>
              )
            }
          />

          <Space style={{ marginTop: 16, width: '100%', justifyContent: 'space-between' }}>
            <Select
              placeholder="选择目标设备"
              style={{ width: 200 }}
              value={selectedDevice}
              onChange={setSelectedDevice}
              options={devices
                .filter(d => d.isOnline)
                .map(d => ({ value: d.id, label: `${d.name} (${d.ip})` }))
              }
            />
            <Button
              type="primary"
              icon={<SendOutlined />}
              loading={sending}
              onClick={handleSend}
            >
              发送文件
            </Button>
            <Button
              icon={<DeleteOutlined />}
              onClick={handleClear}
            >
              清空
            </Button>
          </Space>
        </div>
      )}
    </div>
  )
}

export default FileDropZone