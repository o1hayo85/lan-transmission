import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

interface AppSettings {
  default_save_path: string
}

interface SettingsState {
  defaultSavePath: string
  isLoading: boolean
  error: string | null
  loadSettings: () => Promise<void>
  setDefaultSavePath: (path: string) => Promise<void>
  validatePath: (path: string) => Promise<{ valid: boolean; error?: string }>
}

export const useSettingsStore = create<SettingsState>((set) => ({
  defaultSavePath: '',
  isLoading: false,
  error: null,

  loadSettings: async () => {
    set({ isLoading: true, error: null })
    try {
      const settings = await invoke<AppSettings>('get_settings')
      set({ defaultSavePath: settings.default_save_path || '', isLoading: false })
    } catch (error) {
      set({ error: String(error), isLoading: false })
    }
  },

  setDefaultSavePath: async (path: string) => {
    set({ isLoading: true, error: null })
    try {
      await invoke('set_default_save_path', { path })
      set({ defaultSavePath: path, isLoading: false })
    } catch (error) {
      set({ error: String(error), isLoading: false })
      throw error
    }
  },

  validatePath: async (path: string) => {
    try {
      await invoke('validate_save_path', { path })
      return { valid: true }
    } catch (error) {
      return { valid: false, error: String(error) }
    }
  },
}))