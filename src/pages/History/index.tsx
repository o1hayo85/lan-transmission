import { useEffect } from 'react'
import { Card, Table, Tag, Space, Button, Empty, Spin, message, Modal, Descriptions } from 'antd'
import { ReloadOutlined, EyeOutlined } from '@ant-design/icons'
import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'
import dayjs from 'dayjs'

interface HistoryRecord {
  id: string
  direction: string
  status: string
  peer_device_id: string
  peer_device_name: string
  peer_ip: string | null
  total_size: number
  transferred_size: number
  created_at: number
  completed_at: number | null
}

interface FileRecord {
  id: string
  transfer_id: string
  name: string
  path: string | null
  size: number
  md5: string | null
  status: string
  created_at: number
}

const statusColors: Record<string, string> = {
  completed: 'success',
  in_progress: 'processing',
  pending: 'default',
  waiting_accept: 'default',
  cancelled: 'warning',
  rejected: 'error',
  failed: 'error',
}

const statusText: Record<string, string> = {
  completed: '已完成',
  in_progress: '传输中',
  pending: '等待',
  waiting_accept: '等待确认',
  cancelled: '已取消',
  rejected: '已拒绝',
  failed: '失败',
}

function History() {
  const [history, setHistory] = useState<HistoryRecord[]>([])
  const [loading, setLoading] = useState(true)
  const [detailVisible, setDetailVisible] = useState(false)
  const [selectedRecord, setSelectedRecord] = useState<HistoryRecord | null>(null)
  const [files, setFiles] = useState<FileRecord[]>([])
  const [filesLoading, setFilesLoading] = useState(false)

  const loadHistory = async () => {
    setLoading(true)
    try {
      const records = await invoke<HistoryRecord[]>('get_transfer_history')
      setHistory(records)
    } catch (error) {
      message.error('加载历史记录失败')
    }
    setLoading(false)
  }

  const loadFiles = async (transferId: string) => {
    setFilesLoading(true)
    try {
      const fileRecords = await invoke<FileRecord[]>('get_transfer_files', { transferId })
      setFiles(fileRecords)
    } catch (error) {
      message.error('加载文件列表失败')
      setFiles([])
    }
    setFilesLoading(false)
  }

  const showDetail = async (record: HistoryRecord) => {
    setSelectedRecord(record)
    setDetailVisible(true)
    await loadFiles(record.id)
  }

  useEffect(() => {
    loadHistory()
  }, [])

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
    if (bytes < 1024 * 1024 * 1024) return (bytes / 1024 / 1024).toFixed(2) + ' MB'
    return (bytes / 1024 / 1024 / 1024).toFixed(2) + ' GB'
  }

  const columns = [
    {
      title: '时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: number) => dayjs(time * 1000).format('YYYY-MM-DD HH:mm:ss'),
      sorter: (a: HistoryRecord, b: HistoryRecord) => a.created_at - b.created_at,
      defaultSortOrder: 'descend' as const,
    },
    {
      title: '方向',
      dataIndex: 'direction',
      key: 'direction',
      render: (dir: string) => (
        <Tag color={dir === 'send' ? 'blue' : 'green'}>
          {dir === 'send' ? '发送' : '接收'}
        </Tag>
      ),
    },
    {
      title: '设备',
      dataIndex: 'peer_device_name',
      key: 'peer_device_name',
    },
    {
      title: 'IP',
      dataIndex: 'peer_ip',
      key: 'peer_ip',
      render: (ip: string | null) => ip || '-',
    },
    {
      title: '大小',
      dataIndex: 'total_size',
      key: 'total_size',
      render: (size: number) => formatSize(size),
    },
    {
      title: '进度',
      key: 'progress',
      render: (_: any, record: HistoryRecord) => {
        const percent = record.total_size > 0
          ? Math.round((record.transferred_size / record.total_size) * 100)
          : 0
        return `${percent}%`
      },
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={statusColors[status] || 'default'}>
          {statusText[status] || status}
        </Tag>
      ),
    },
    {
      title: '操作',
      key: 'action',
      render: (_: any, record: HistoryRecord) => (
        <Button
          type="link"
          size="small"
          icon={<EyeOutlined />}
          onClick={() => showDetail(record)}
        >
          详情
        </Button>
      ),
    },
  ]

  return (
    <>
      <Card
        title="传输历史"
        extra={
          <Space>
            <Button icon={<ReloadOutlined />} onClick={loadHistory}>
              刷新
            </Button>
          </Space>
        }
      >
        {loading ? (
          <div style={{ textAlign: 'center', padding: 40 }}>
            <Spin />
          </div>
        ) : history.length === 0 ? (
          <Empty description="暂无传输记录" />
        ) : (
          <Table
            dataSource={history}
            columns={columns}
            rowKey="id"
            pagination={{ pageSize: 20 }}
          />
        )}
      </Card>

      {/* 详情弹窗 */}
      <Modal
        open={detailVisible}
        title="传输详情"
        onCancel={() => setDetailVisible(false)}
        footer={<Button onClick={() => setDetailVisible(false)}>关闭</Button>}
        width={600}
      >
        {selectedRecord && (
          <>
            <Descriptions column={2} bordered size="small">
              <Descriptions.Item label="传输ID">{selectedRecord.id}</Descriptions.Item>
              <Descriptions.Item label="方向">
                <Tag color={selectedRecord.direction === 'send' ? 'blue' : 'green'}>
                  {selectedRecord.direction === 'send' ? '发送' : '接收'}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="设备名称">{selectedRecord.peer_device_name}</Descriptions.Item>
              <Descriptions.Item label="设备IP">{selectedRecord.peer_ip || '-'}</Descriptions.Item>
              <Descriptions.Item label="总大小">{formatSize(selectedRecord.total_size)}</Descriptions.Item>
              <Descriptions.Item label="已传输">{formatSize(selectedRecord.transferred_size)}</Descriptions.Item>
              <Descriptions.Item label="状态">
                <Tag color={statusColors[selectedRecord.status] || 'default'}>
                  {statusText[selectedRecord.status] || selectedRecord.status}
                </Tag>
              </Descriptions.Item>
              <Descriptions.Item label="创建时间">
                {dayjs(selectedRecord.created_at * 1000).format('YYYY-MM-DD HH:mm:ss')}
              </Descriptions.Item>
            </Descriptions>

            <div style={{ marginTop: 16 }}>
              <strong>文件列表：</strong>
              {filesLoading ? (
                <Spin />
              ) : files.length === 0 ? (
                <div style={{ color: '#999', marginTop: 8 }}>无文件记录</div>
              ) : (
                <Table
                  dataSource={files}
                  rowKey="id"
                  size="small"
                  pagination={false}
                  columns={[
                    { title: '文件名', dataIndex: 'name', key: 'name' },
                    { title: '大小', dataIndex: 'size', key: 'size', render: (s: number) => formatSize(s) },
                    { title: 'MD5', dataIndex: 'md5', key: 'md5', render: (m: string | null) => m || '-' },
                    { title: '状态', dataIndex: 'status', key: 'status' },
                  ]}
                  style={{ marginTop: 8 }}
                />
              )}
            </div>
          </>
        )}
      </Modal>
    </>
  )
}

export default History