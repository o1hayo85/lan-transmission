import { create } from 'zustand'
import { TransferRecord, TransferStatus } from '../types'

interface TransferState {
  transfers: TransferRecord[]
  currentTransfer: TransferRecord | null
  pendingRequests: TransferRecord[]
  setTransfers: (transfers: TransferRecord[]) => void
  addTransfer: (transfer: TransferRecord) => void
  removeTransfer: (transferId: string) => void
  updateTransferStatus: (transferId: string, status: TransferStatus) => void
  updateTransferProgress: (transferId: string, transferredSize: number) => void
  setCurrentTransfer: (transfer: TransferRecord | null) => void
  addPendingRequest: (request: TransferRecord) => void
  removePendingRequest: (transferId: string) => void
}

export const useTransferStore = create<TransferState>((set) => ({
  transfers: [],
  currentTransfer: null,
  pendingRequests: [],
  setTransfers: (transfers) => set({ transfers }),
  addTransfer: (transfer) => set((state) => ({
    transfers: [...state.transfers, transfer]
  })),
  removeTransfer: (transferId) => set((state) => ({
    transfers: state.transfers.filter(t => t.id !== transferId)
  })),
  updateTransferStatus: (transferId, status) => set((state) => ({
    transfers: state.transfers.map(t => t.id === transferId ? { ...t, status } : t)
  })),
  updateTransferProgress: (transferId, transferredSize) => set((state) => ({
    transfers: state.transfers.map(t => t.id === transferId ? { ...t, transferredSize } : t)
  })),
  setCurrentTransfer: (transfer) => set({ currentTransfer: transfer }),
  addPendingRequest: (request) => set((state) => ({
    pendingRequests: [...state.pendingRequests, request]
  })),
  removePendingRequest: (transferId) => set((state) => ({
    pendingRequests: state.pendingRequests.filter(r => r.id !== transferId)
  }))
}))