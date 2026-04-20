import { useEffect, useState } from 'react'
import { Card, Button, Input, Space, message, Typography, Spin, Alert } from 'antd'
import { FolderOpenOutlined, ReloadOutlined, DeleteOutlined } from '@ant-design/icons'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingsStore } from '../../stores/settingsStore'

const { Title, Text } = Typography

function Settings() {
  const {
    defaultSavePath,
    isLoading,
    error,
    loadSettings,
    setDefaultSavePath,
    validatePath
  } = useSettingsStore()

  const [localPath, setLocalPath] = useState('')
  const [saving, setSaving] = useState(false)
  const [pathError, setPathError] = useState<string | null>(null)
  const [validating, setValidating] = useState(false)

  useEffect(() => {
    loadSettings()
  }, [])

  useEffect(() => {
    setLocalPath(defaultSavePath)
  }, [defaultSavePath])

  // 实时路径验证（带防抖）
  useEffect(() => {
    if (!localPath) {
      setPathError(null)
      return
    }

    setValidating(true)
    const timer = setTimeout(async () => {
      const result = await validatePath(localPath)
      setPathError(result.valid ? null : result.error || '路径无效')
      setValidating(false)
    }, 500)

    return () => clearTimeout(timer)
  }, [localPath, validatePath])

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择默认保存文件夹'
      })
      if (selected) {
        const path = selected as string
        // 验证路径
        const validation = await validatePath(path)
        if (validation.valid) {
          setLocalPath(path)
        } else {
          message.error(validation.error || '路径无效')
        }
      }
    } catch (err) {
      message.error('选择文件夹失败')
    }
  }

  const handleSave = async () => {
    if (!localPath) {
      // 清空路径
      setSaving(true)
      try {
        await setDefaultSavePath('')
        message.success('已清除默认保存路径')
      } catch (err) {
        message.error(`保存失败: ${err}`)
      }
      setSaving(false)
      return
    }

    // 先验证路径
    const validation = await validatePath(localPath)
    if (!validation.valid) {
      message.error(validation.error || '路径无效')
      return
    }

    setSaving(true)
    try {
      await setDefaultSavePath(localPath)
      message.success('保存成功')
    } catch (err) {
      message.error(`保存失败: ${err}`)
    }
    setSaving(false)
  }

  const handleClear = async () => {
    setLocalPath('')
  }

  return (
    <Card
      title="设置"
      extra={
        <Button
          icon={<ReloadOutlined />}
          onClick={loadSettings}
          loading={isLoading}
          size="small"
        >
          刷新
        </Button>
      }
      style={{ maxWidth: 600 }}
    >
      <Spin spinning={isLoading}>
        {error && (
          <Alert
            message="错误"
            description={error}
            type="error"
            closable
            style={{ marginBottom: 16 }}
          />
        )}

        <div style={{ marginBottom: 24 }}>
          <Title level={5}>默认保存路径</Title>
          <Text type="secondary">
            接收文件时的默认保存位置。设置后，接收确认框会自动填入此路径，您仍可临时修改。
          </Text>
        </div>

        <Space.Compact style={{ width: '100%', marginBottom: 8 }}>
          <Input
            value={localPath}
            onChange={(e) => setLocalPath(e.target.value)}
            placeholder="未设置 - 每次接收时需选择路径"
            status={pathError ? 'error' : undefined}
          />
          <Button icon={<FolderOpenOutlined />} onClick={handleSelectFolder}>
            选择
          </Button>
          {localPath && (
            <Button icon={<DeleteOutlined />} onClick={handleClear}>
              清除
            </Button>
          )}
        </Space.Compact>

        {/* 实时验证反馈 */}
        {validating && (
          <Text type="secondary" style={{ fontSize: 12 }}>
            正在验证路径...
          </Text>
        )}
        {pathError && !validating && (
          <Text type="danger" style={{ fontSize: 12, color: '#ff4d4f' }}>
            {pathError}
          </Text>
        )}

        <Button
          type="primary"
          onClick={handleSave}
          loading={saving}
          disabled={pathError !== null || validating}
          style={{ marginTop: 16 }}
        >
          保存设置
        </Button>

        {defaultSavePath && (
          <Alert
            message="当前已设置默认路径"
            description={`文件将默认保存到: ${defaultSavePath}`}
            type="info"
            showIcon
            style={{ marginTop: 16 }}
          />
        )}
      </Spin>
    </Card>
  )
}

export default Settings