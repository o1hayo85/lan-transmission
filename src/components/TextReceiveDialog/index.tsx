import { useEffect, useState } from 'react'
import { Modal, Button, Input, Space, message, Typography } from 'antd'
import { CopyOutlined, CheckOutlined, ExclamationCircleOutlined } from '@ant-design/icons'
import { listen } from '@tauri-apps/api/event'

const { Text } = Typography

interface TextRequest {
  transfer_id: string
  sender_name: string
  text_content: string
  total_size: number
}

function TextReceiveDialog() {
  const [currentRequest, setCurrentRequest] = useState<TextRequest | null>(null)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    // 监听文本请求事件
    const unlistenPromise = listen<TextRequest>('text-request', (event) => {
      console.log('收到文本请求:', event.payload)
      setCurrentRequest(event.payload)
      setCopied(false)
    })

    return () => {
      unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  const handleCopy = async () => {
    if (!currentRequest?.text_content) return

    try {
      await navigator.clipboard.writeText(currentRequest.text_content)
      setCopied(true)
      message.success('已复制到剪贴板')
      setTimeout(() => setCopied(false), 2000)
    } catch {
      message.error('复制失败')
    }
  }

  const handleClose = () => {
    // 如果文本较长（超过100字符）且用户未复制，提示确认
    if (currentRequest && currentRequest.total_size > 100 && !copied) {
      Modal.confirm({
        title: '确认关闭',
        icon: <ExclamationCircleOutlined />,
        content: '您尚未复制文本内容，关闭后文本将丢失。确定要关闭吗？',
        okText: '确定关闭',
        cancelText: '取消',
        onOk: () => {
          setCurrentRequest(null)
          setCopied(false)
        }
      })
    } else {
      setCurrentRequest(null)
      setCopied(false)
    }
  }

  if (!currentRequest) return null

  return (
    <Modal
      open={true}
      title={`${currentRequest.sender_name} 发送了文本`}
      onCancel={handleClose}
      width={600}
      footer={
        <Space>
          <Button onClick={handleClose}>关闭</Button>
          <Button
            type="primary"
            icon={copied ? <CheckOutlined /> : <CopyOutlined />}
            onClick={handleCopy}
          >
            {copied ? '已复制' : '复制文本'}
          </Button>
        </Space>
      }
    >
      <Text type="secondary" style={{ marginBottom: 12 }}>
        文本长度: {currentRequest.total_size} 字符
      </Text>
      <Input.TextArea
        value={currentRequest.text_content}
        readOnly
        autoSize={{ minRows: 6, maxRows: 20 }}
        style={{ fontFamily: 'monospace', marginTop: 8 }}
      />
    </Modal>
  )
}

export default TextReceiveDialog