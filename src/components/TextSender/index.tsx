import { Input, Typography, Card } from 'antd'

const { TextArea } = Input
const { Text } = Typography

interface TextSenderProps {
  textContent: string
  onTextContentChange: (text: string) => void
}

function TextSender({ textContent, onTextContentChange }: TextSenderProps) {
  return (
    <Card title="发送文本" style={{ marginTop: 16 }}>
      <TextArea
        value={textContent}
        onChange={(e) => onTextContentChange(e.target.value)}
        placeholder="输入要发送的文本内容，然后在设备列表点击发送..."
        autoSize={{ minRows: 3, maxRows: 10 }}
        style={{ marginBottom: 12 }}
      />
      <Text type="secondary">
        {textContent.length} 字符 | 在左侧设备列表点击"发送"按钮发送文本
      </Text>
    </Card>
  )
}

export default TextSender