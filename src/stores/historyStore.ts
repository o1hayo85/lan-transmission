import { create } from 'zustand'
import { TransferRecord } from '../types'

interface HistoryState {
  history: TransferRecord[]
  setHistory: (history: TransferRecord[]) => void
  addRecord: (record: TransferRecord) => void
  clearHistory: () => void
}

export const useHistoryStore = create<HistoryState>((set) => ({
  history: [],
  setHistory: (history) => set({ history }),
  addRecord: (record) => set((state) => ({
    history: [record, ...state.history]
  })),
  clearHistory: () => set({ history: [] })
}))