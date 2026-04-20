import { List, Avatar, Button, Tag } from 'antd'
import { DesktopOutlined } from '@ant-design/icons'
import { useDeviceStore } from '../../stores/deviceStore'
import './index.css'

function DeviceList() {
  const { devices } = useDeviceStore()

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
            <Button type="primary" size="small">选择</Button>
          </div>
        </List.Item>
      )}
    />
  )
}

export default DeviceList