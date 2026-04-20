// 设备信息
export interface Device {
  id: string
  name: string
  ip: string
  port: number
  lastSeen: number
  isOnline: boolean
}

// 文件信息
export interface FileInfo {
  id: string
  name: string
  size: number
  type: string
  relativePath?: string
  filePath?: string  // 文件完整路径（发送方使用）
  md5?: string  // 文件MD5校验值
}

// 传输请求
export interface TransferRequest {
  transferId: string
  senderId: string
  senderName: string
  files: FileInfo[]
  totalSize: number
  timestamp: number
}

// 传输状态
export type TransferStatus = 'pending' | 'waiting_accept' | 'in_progress' | 'completed' | 'cancelled' | 'rejected' | 'failed'

// 传输方向
export type TransferDirection = 'send' | 'receive'

// 传输记录
export interface TransferRecord {
  id: string
  direction: TransferDirection
  status: TransferStatus
  peerDeviceId: string
  peerDeviceName: string
  peerIp?: string  // 对方设备IP
  totalSize: number
  transferredSize: number
  files: FileInfo[]
  createdAt: number
  completedAt?: number
}