import { useState, useEffect } from 'react'
import { BrowserRouter, Routes, Route, useNavigate, useLocation } from 'react-router-dom'
import { Layout, Menu, Button } from 'antd'
import { MenuFoldOutlined, MenuUnfoldOutlined, SendOutlined, HistoryOutlined, SettingOutlined } from '@ant-design/icons'
import Home from './pages/Home'
import History from './pages/History'
import Settings from './pages/Settings'
import UploadHandler from './components/UploadHandler'
import ReceiveHandler from './components/ReceiveHandler'
import './App.css'

const { Sider, Content, Header } = Layout

function AppContent() {
  const navigate = useNavigate()
  const location = useLocation()
  const [collapsed, setCollapsed] = useState(false)

  // 小屏时自动收起侧边栏
  useEffect(() => {
    const checkWidth = () => {
      setCollapsed(window.innerWidth < 800)
    }
    checkWidth()
    window.addEventListener('resize', checkWidth)
    return () => window.removeEventListener('resize', checkWidth)
  }, [])

  return (
    <Layout style={{ height: '100vh', overflow: 'hidden' }}>
      <Sider
        width={200}
        collapsedWidth={60}
        collapsed={collapsed}
        theme="light"
        style={{ overflow: 'hidden' }}
      >
        <div className="logo">{collapsed ? '传' : '文件传输'}</div>
        <Menu
          mode="inline"
          selectedKeys={[location.pathname]}
          onClick={({ key }) => navigate(key)}
          items={[
            {
              key: '/',
              label: collapsed ? '' : '发送文件',
              icon: <SendOutlined />,
              title: '发送文件'
            },
            {
              key: '/history',
              label: collapsed ? '' : '传输历史',
              icon: <HistoryOutlined />,
              title: '传输历史'
            },
            {
              key: '/settings',
              label: collapsed ? '' : '设置',
              icon: <SettingOutlined />,
              title: '设置'
            },
          ]}
        />
      </Sider>
      <Layout style={{ display: 'flex', flexDirection: 'column' }}>
        <Header style={{ padding: '0 16px', background: '#fff', height: 48, lineHeight: '48px', flex: '0 0 48px' }}>
          <Button
            type="text"
            icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
            onClick={() => setCollapsed(!collapsed)}
            style={{ fontSize: 16 }}
          />
        </Header>
        <Content style={{ padding: '16px', background: '#f5f5f5', flex: 1, overflow: 'auto' }}>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/history" element={<History />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  )
}

function App() {
  return (
    <BrowserRouter>
      <UploadHandler />
      <ReceiveHandler />
      <AppContent />
    </BrowserRouter>
  )
}

export default App