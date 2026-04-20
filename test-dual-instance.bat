@echo off
echo ============================================
echo    局域网文件传输工具 - 双实例测试
echo ============================================
echo.
echo 启动两个实例（不同HTTP端口，相同UDP端口）
echo 实例1: HTTP 8080
echo 实例2: HTTP 8081
echo.

cd /d E:\dev\project\lan-transmission

echo 启动实例1...
start "实例1-HTTP8080" cmd /c "set LAN_HTTP_PORT=8080 && npm run tauri dev"

timeout /t 8 >nul

echo 启动实例2...
start "实例2-HTTP8081" cmd /c "set LAN_HTTP_PORT=8081 && npm run tauri dev"

echo.
echo ============================================
echo 两个实例已启动！
echo 请在两个窗口中观察是否能互相发现设备
echo ============================================
pause