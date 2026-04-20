import { useState, useEffect } from 'react'
import { Button, List, Space, message, Spin } from 'antd'
import { InboxOutlined, FolderOpenOutlined, FileAddOutlined, DeleteOutlined } from '@ant-design/icons'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'

interface FileEntry {
  id: string
  name: string
  path: string
  size: number
  relative_path: string
  is_dir: boolean
}

export interface SelectedItem {
  id: string
  name: string
  path: string
  size: number
  relativePath?: string
  isFolder?: boolean
}

interface FileDropZoneProps {
  fileList: SelectedItem[]
  onFileListChange: (files: SelectedItem[]) => void
}

function FileDropZone({ fileList, onFileListChange }: FileDropZoneProps) {
  const [loading, setLoading] = useState(false)
  const [isDragging, setIsDragging] = useState(false)

  // 监听Tauri 2.x拖拽事件
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        setIsDragging(true)
      } else if (event.payload.type === 'drop') {
        setIsDragging(false)
        const paths = event.payload.paths
        if (paths.length > 0) {
          setLoading(true)
          const newFiles: SelectedItem[] = []

          for (const path of paths) {
            const name = path.split(/[\\/]/).pop() || 'unknown'
            invoke<number>('get_file_size', { filePath: path }).then(size => {
              newFiles.push({
                id: `file_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
                name,
                path,
                size,
              })
              // 最后一个文件处理完后更新列表
              if (newFiles.length === paths.length) {
                onFileListChange([...fileList, ...newFiles])
                setLoading(false)
              }
            }).catch(() => {
              // 获取文件大小失败时使用0
              newFiles.push({
                id: `file_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
                name,
                path,
                size: 0,
              })
              if (newFiles.length === paths.length) {
                onFileListChange([...fileList, ...newFiles])
                setLoading(false)
              }
            })
          }
        }
      } else {
        // 'cancel' type
        setIsDragging(false)
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [fileList, onFileListChange])

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

        onFileListChange([...fileList, ...newFiles])
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
          onFileListChange([...fileList, {
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

  // 清空文件列表
  const handleClear = () => {
    onFileListChange([])
  }

  // 显示列表（合并文件夹项和文件项，最多显示5个）
  const displayList = fileList.slice(0, 5)
  const actualFileCount = fileList.filter(f => !f.isFolder).length

  return (
    <div>
      <div
        style={{
          padding: '40px 0',
          textAlign: 'center',
          border: isDragging ? '2px dashed #1890ff' : '1px dashed #d9d9d9',
          borderRadius: 8,
          background: isDragging ? '#e6f7ff' : '#fafafa',
          transition: 'all 0.2s ease'
        }}
      >
        <InboxOutlined style={{ fontSize: 48, color: isDragging ? '#1890ff' : '#999' }} />
        <p style={{ marginTop: 16, color: '#666' }}>
          {isDragging ? '放开即可添加文件' : '拖拽文件到这里或点击下方按钮选择'}
        </p>
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
            header={
              <Space style={{ width: '100%', justifyContent: 'space-between' }}>
                <span>已选择 {actualFileCount} 个文件，总大小: {formatSize(totalSize)}</span>
                <Button icon={<DeleteOutlined />} size="small" onClick={handleClear}>
                  清空
                </Button>
              </Space>
            }
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
        </div>
      )}
    </div>
  )
}

export default FileDropZone