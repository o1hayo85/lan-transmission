import { Card, Row, Col, Typography, Empty, Spin, Badge } from 'antd'
import DeviceList from '../../components/DeviceList'
import FileDropZone from '../../components/FileDropZone'
import TransferProgress from '../../components/TransferProgress'
import ConfirmDialog from '../../components/ConfirmDialog'
import { useDeviceStore } from '../../stores/deviceStore'
import { useDeviceDiscovery } from '../../hooks/useDeviceDiscovery'
import { useTransferListener } from '../../hooks/useTransferListener'

const { Title } = Typography

function Home() {
  const { devices } = useDeviceStore()
  useDeviceDiscovery()
  useTransferListener()

  return (
    <div>
      <ConfirmDialog />
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={6}>
          <Card
            title="在线设备"
            extra={<Badge count={devices.filter(d => d.isOnline).length} />}
            styles={{ body: { maxHeight: 280, overflow: 'auto', minWidth: 200 } }}
          >
            {devices.length === 0 ? (
              <Empty description="正在搜索局域网设备..." image={<Spin />} />
            ) : (
              <DeviceList />
            )}
          </Card>
        </Col>
        <Col xs={24} lg={18}>
          <Card title="发送文件">
            <Title level={5} style={{ marginBottom: 12 }}>
              拖拽文件或点击选择要发送的文件
            </Title>
            <FileDropZone />
          </Card>
          <Card title="传输进度" style={{ marginTop: 16 }} styles={{ body: { maxHeight: 200, overflow: 'auto' } }}>
            <TransferProgress />
          </Card>
        </Col>
      </Row>
    </div>
  )
}

export default Home