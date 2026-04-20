import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useDeviceStore } from '../stores/deviceStore'

interface DiscoveryMessage {
  msg_type: string
  device_id: string
  device_name: string
  ip: string
  port: number
  timestamp: number
}

export function useDeviceDiscovery() {
  const { addDevice, removeDevice } = useDeviceStore()

  useEffect(() => {
    // 监听设备发现事件
    const unlistenDiscovered = listen<DiscoveryMessage>('device-discovered', (event) => {
      const msg = event.payload
      if (msg.msg_type === 'announce' || msg.msg_type === 'response') {
        addDevice({
          id: msg.device_id,
          name: msg.device_name,
          ip: msg.ip,
          port: msg.port,
          lastSeen: msg.timestamp,
          isOnline: true
        })
      } else if (msg.msg_type === 'bye') {
        removeDevice(msg.device_id)
      }
    })

    // 监听设备离线事件（来自超时检测）
    const unlistenLost = listen<string>('device-lost', (event) => {
      removeDevice(event.payload)
    })

    // 清理监听
    return () => {
      unlistenDiscovered.then((fn) => fn())
      unlistenLost.then((fn) => fn())
    }
  }, [addDevice, removeDevice])
}