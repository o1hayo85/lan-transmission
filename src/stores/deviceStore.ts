import { create } from 'zustand'
import { Device } from '../types'

interface DeviceState {
  devices: Device[]
  currentDevice: Device | null
  setDevices: (devices: Device[]) => void
  addDevice: (device: Device) => void
  removeDevice: (deviceId: string) => void
  updateDeviceStatus: (deviceId: string, isOnline: boolean) => void
  setCurrentDevice: (device: Device | null) => void
}

export const useDeviceStore = create<DeviceState>((set) => ({
  devices: [],
  currentDevice: null,
  setDevices: (devices) => set({ devices }),
  addDevice: (device) => set((state) => {
    const exists = state.devices.find(d => d.id === device.id)
    if (exists) {
      return { devices: state.devices.map(d => d.id === device.id ? device : d) }
    }
    return { devices: [...state.devices, device] }
  }),
  removeDevice: (deviceId) => set((state) => ({
    devices: state.devices.filter(d => d.id !== deviceId)
  })),
  updateDeviceStatus: (deviceId, isOnline) => set((state) => ({
    devices: state.devices.map(d => d.id === deviceId ? { ...d, isOnline } : d)
  })),
  setCurrentDevice: (device) => set({ currentDevice: device })
}))