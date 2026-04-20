import { useState } from 'react'
import { Card, Row, Col, Typography, Empty, Spin, Button, message } from 'antd'
import { ReloadOutlined } from '@ant-design/icons'
import { invoke } from '@tauri-apps/api/core'
import DeviceList from '../../components/DeviceList'
import FileDropZone, { SelectedItem } from '../../components/FileDropZone'
import TransferProgress from '../../components/TransferProgress'
import ConfirmDialog from '../../components/ConfirmDialog'
import UploadHandler from '../../components/UploadHandler'
import TextSender from '../../components/TextSender'
import TextReceiveDialog from '../../components/TextReceiveDialog'
import { useDeviceStore } from '../../stores/deviceStore'
import { useDeviceDiscovery } from '../../hooks/useDeviceDiscovery'
import { useTransferListener } from '../../hooks/useTransferListener'

const { Title } = Typography

function Home() {
  const { devices } = useDeviceStore()
  const [fileList, setFileList] = useState<SelectedItem[]>([])
  const [textContent, setTextContent] = useState('')
  const [scanning, setScanning] = useState(false)

  useDeviceDiscovery()
  useTransferListener()

  const handleSendComplete = () => {
    // 发送完成后清空文件列表和文本内容
    setFileList([])
    setTextContent('')
  }

  const handleScan = async () => {
    setScanning(true)
    try {
      await invoke('trigger_device_scan')
      message.info('正在扫描设备...')
    } catch (error) {
      message.error('扫描失败')
    }
    // 等待1秒后结束扫描状态（给用户反馈）
    setTimeout(() => setScanning(false), 1000)
  }

  return (
    <div>
      <ConfirmDialog />
      <UploadHandler />
      <TextReceiveDialog />
      {/* 传输通知管理器 - 右上角显示 */}
      <TransferProgress />
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={6}>
          <Card
            title="在线设备"
            extra={
              <Button
                size="small"
                icon={<ReloadOutlined />}
                onClick={handleScan}
                loading={scanning}
              >
                扫描
              </Button>
            }
            styles={{ body: { maxHeight: 280, overflow: 'auto', minWidth: 200 } }}
          >
            {devices.length === 0 ? (
              <Empty description="正在搜索局域网设备..." image={<Spin />} />
            ) : (
              <DeviceList
                fileList={fileList}
                textContent={textContent}
                onSendComplete={handleSendComplete}
              />
            )}
          </Card>
        </Col>
        <Col xs={24} lg={18}>
          <Card title="发送文件">
            <Title level={5} style={{ marginBottom: 12 }}>
              拖拽文件或点击选择要发送的文件，然后点击设备发送
            </Title>
            <FileDropZone fileList={fileList} onFileListChange={setFileList} />
          </Card>
          <TextSender textContent={textContent} onTextContentChange={setTextContent} />
        </Col>
      </Row>
    </div>
  )
}

export default Home