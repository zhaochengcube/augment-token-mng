# 快速开始指南

## 系统概述

本系统包含两个部分：
1. **Session 管理服务端** - 管理和分发 Augment Session
2. **ATM 客户端** - 使用卡密兑换并导入 Session

## 快速启动

### 1. 启动 Session 管理服务端

```bash
cd ../augment-session-server
npm run dev
```

服务端将在 **http://localhost:3000** 启动

### 2. 启动 ATM 客户端

```bash
cd augment-token-mng-main
npm run dev
```

ATM 将在 **http://localhost:1420** 启动

## 完整使用流程

### 步骤 1: 登录管理后台

1. 打开浏览器访问 http://localhost:3000
2. 输入密码：`admin123`
3. 点击"登录"

### 步骤 2: 添加 Session

1. 在管理后台点击 "Session 管理"
2. 点击 "添加 Session" 按钮
3. 粘贴你的 `auth_session` 字符串
4. 可选：填写备注（如邮箱或账号名称）
5. 点击 "添加"

**示例 auth_session**（测试用）：
```
test_session_12345678901234567890
```

### 步骤 3: 生成卡密

1. 点击 "卡密管理" 标签
2. 点击 "生成卡密" 按钮
3. 选择刚才添加的 Session
4. 选择卡密类型：
   - **一次性**：只能使用一次
   - **多次使用**：可设置使用次数上限
5. 设置生成数量（1-100）
6. 可选：设置过期时间
7. 点击 "生成"
8. **复制生成的卡密代码**（UUID 格式）

### 步骤 4: 在 ATM 中兑换卡密

1. 打开 ATM 应用（http://localhost:1420）
2. 点击 "Token 列表" 按钮
3. 点击 "卡密兑换" 按钮（绿色按钮）
4. 输入服务器地址：`http://localhost:3000`
5. 粘贴卡密代码
6. 点击 "兑换"
7. 等待兑换完成
8. Session 将自动导入到 Token 列表

## 配置说明

### 服务端配置

**文件位置**: `../augment-session-server/.env.local`

```env
# 管理员密码（明文）
ADMIN_PASSWORD=admin123
```

**修改密码**：直接编辑 `.env.local` 文件，然后重启服务端

### 客户端配置

- **服务器地址**：在兑换界面可以修改
- **默认地址**：http://localhost:3000

## 卡密类型说明

### 一次性卡密
- 只能兑换一次
- 兑换后立即失效
- 适合单个用户使用

### 多次使用卡密
- 可设置最大使用次数（如 5 次）
- 达到上限后失效
- 适合多设备或多次导入

### 过期时间
- 可选设置
- 到期后卡密自动失效
- 未设置则永久有效（直到使用完）

## 常见问题

### Q: 兑换时提示"卡密不存在"
**A**: 检查卡密代码是否完整复制，UUID 格式应为：`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`

### Q: 兑换时提示"卡密使用次数已达上限"
**A**: 该卡密已被使用完，需要生成新卡密

### Q: 兑换时提示"网络请求失败"
**A**: 
1. 检查服务端是否正常运行（http://localhost:3000）
2. 检查服务器地址是否正确
3. 检查防火墙设置

### Q: 登录管理后台失败
**A**: 
1. 确认密码为 `admin123`
2. 检查 `.env.local` 文件是否存在
3. 重启服务端

### Q: Session 导入后无法使用
**A**: 
1. 确认 auth_session 格式正确
2. 检查 Session 是否有效
3. 查看 ATM 控制台日志

## 数据管理

### 查看数据库

数据库文件位置：`../augment-session-server/data/sessions.db`

使用 SQLite 客户端查看：
```bash
sqlite3 ../augment-session-server/data/sessions.db

# 查看所有 Session
SELECT * FROM sessions;

# 查看所有卡密
SELECT * FROM activation_keys;

# 查看兑换记录
SELECT * FROM redemption_logs;
```

### 备份数据

```bash
# 备份数据库
cp ../augment-session-server/data/sessions.db ../augment-session-server/data/sessions.db.backup

# 恢复数据库
cp ../augment-session-server/data/sessions.db.backup ../augment-session-server/data/sessions.db
```

## 生产部署

### 服务端部署

1. **构建项目**
   ```bash
   cd ../augment-session-server
   npm run build
   ```

2. **启动生产服务**
   ```bash
   npm start
   ```

3. **配置 HTTPS**（推荐使用 Nginx）
   ```nginx
   server {
       listen 443 ssl;
       server_name your-domain.com;
       
       ssl_certificate /path/to/cert.pem;
       ssl_certificate_key /path/to/key.pem;
       
       location / {
           proxy_pass http://localhost:3000;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   ```

4. **修改管理员密码**
   ```env
   ADMIN_PASSWORD=your_strong_password_here
   ```

5. **设置环境变量**
   ```bash
   export NODE_ENV=production
   export ADMIN_PASSWORD=your_strong_password
   ```

### 客户端分发

1. **构建应用**
   ```bash
   cd augment-token-mng-main
   cargo tauri build
   ```

2. **安装包位置**
   - MSI: `src-tauri/target/release/bundle/msi/ATM_*.msi`
   - NSIS: `src-tauri/target/release/bundle/nsis/ATM_*-setup.exe`

3. **分发给用户**
   - 提供安装包
   - 告知服务器地址（如 https://your-domain.com）
   - 提供卡密

## 安全建议

1. ✅ **修改默认密码** - 务必修改 `admin123`
2. ✅ **使用 HTTPS** - 生产环境必须使用 HTTPS
3. ✅ **限制访问** - 使用防火墙限制管理后台 IP
4. ✅ **定期备份** - 定期备份数据库文件
5. ✅ **监控日志** - 监控异常的兑换请求
6. ✅ **设置过期时间** - 为卡密设置合理的过期时间

## 技术支持

### 查看日志

**服务端日志**：
- 开发模式：终端输出
- 生产模式：使用 PM2 或 systemd 管理日志

**客户端日志**：
- 浏览器控制台（F12）
- Tauri 日志文件

### 调试模式

**服务端**：
```bash
npm run dev  # 自动重载
```

**客户端**：
```bash
npm run dev  # 开发模式，支持热重载
```

## 下一步

- 📖 阅读 [集成指南](INTEGRATION_GUIDE.md) 了解技术细节
- 📖 阅读 [服务端文档](../augment-session-server/README.md) 了解 API
- 📖 阅读 [测试指南](../augment-session-server/TESTING.md) 进行测试

## 总结

现在你已经完成了：
- ✅ 服务端启动并运行
- ✅ ATM 客户端启动并运行
- ✅ 了解完整的使用流程
- ✅ 知道如何添加 Session 和生成卡密
- ✅ 知道如何在 ATM 中兑换卡密

**开始使用吧！** 🚀

