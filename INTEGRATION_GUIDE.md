# ATM 卡密兑换功能集成指南

## 概述

已成功将卡密兑换功能集成到 Augment Token Manager (ATM) 客户端。用户现在可以通过输入卡密代码直接在 ATM 中兑换并导入 Session。

## 架构说明

### 服务端（Session Server）
- **位置**: `../augment-session-server/`
- **技术栈**: Next.js 14 + React + SQLite
- **功能**: 管理 Session 和卡密，提供兑换 API

### 客户端（ATM）
- **位置**: `./augment-token-mng-main/`
- **技术栈**: Tauri 2.0 + Vue 3 + Rust
- **新增功能**: 卡密兑换界面和后端处理

## 新增文件

### 服务端
- `../augment-session-server/` - 完整的 Session 管理服务
  - `app/api/redeem/route.ts` - 卡密兑换 API
  - `lib/services/keyService.ts` - 卡密业务逻辑
  - `lib/services/sessionService.ts` - Session 业务逻辑
  - `README.md` - 服务端文档
  - `TESTING.md` - 测试指南

### 客户端
- `src-tauri/src/key_redeem.rs` - 卡密兑换 Rust 模块
  - `redeem_activation_key()` - 调用服务端 API 兑换卡密
  - `RedeemRequest/Response` - 请求/响应数据结构

### 修改的文件
- `src-tauri/src/main.rs`
  - 添加 `mod key_redeem;`
  - 添加 `redeem_and_import_key` Tauri 命令
  - 在 `invoke_handler` 中注册命令

- `src/components/TokenList.vue`
  - 添加"卡密兑换"按钮
  - 添加卡密兑换模态框
  - 添加 `handleRedeemKey()` 处理函数
  - 添加相关响应式状态和样式

## 使用流程

### 1. 启动服务端

```bash
cd ../augment-session-server
npm run dev
```

服务端将在 http://localhost:3000 启动

### 2. 管理员操作

1. 访问 http://localhost:3000
2. 使用默认密码 `admin123` 登录
3. 添加 Session（粘贴 auth_session 字符串）
4. 生成卡密（选择 Session、类型、数量等）
5. 复制生成的卡密分发给用户

### 3. 用户兑换（在 ATM 客户端）

1. 构建并运行 ATM 客户端：
   ```bash
   cd augment-token-mng-main
   npm run dev
   # 或
   cargo tauri build
   ```

2. 在 ATM 中点击"Token 列表"
3. 点击"卡密兑换"按钮
4. 输入服务器地址（默认 http://localhost:3000）
5. 输入卡密代码
6. 点击"兑换"
7. 等待兑换完成，Session 自动导入

## API 接口

### 兑换卡密

**端点**: `POST /api/redeem`

**请求体**:
```json
{
  "key_code": "550e8400-e29b-41d4-a716-446655440000"
}
```

**成功响应**:
```json
{
  "success": true,
  "data": {
    "auth_session": "session_string_here",
    "message": "兑换成功"
  }
}
```

**错误响应**:
```json
{
  "success": false,
  "error": "卡密不存在"
}
```

## 数据流

```
用户输入卡密
    ↓
ATM Vue 前端 (TokenList.vue)
    ↓
调用 invoke('redeem_and_import_key')
    ↓
Rust 后端 (key_redeem.rs)
    ↓
HTTP POST /api/redeem
    ↓
Session Server (Next.js API)
    ↓
验证卡密 → 返回 auth_session
    ↓
Rust 后端接收 auth_session
    ↓
调用现有的 add_token_from_session_internal_with_cache()
    ↓
Session 导入完成
    ↓
刷新 Token 列表
```

## 测试步骤

### 完整端到端测试

1. **启动服务端**
   ```bash
   cd ../augment-session-server
   npm run dev
   ```

2. **登录管理后台**
   - 访问 http://localhost:3000
   - 密码: `admin123`

3. **添加测试 Session**
   - 进入 "Session 管理"
   - 添加一个测试 session（可以使用任意字符串）
   - 备注: "测试账号"

4. **生成卡密**
   - 进入 "卡密管理"
   - 选择刚才添加的 Session
   - 类型: 一次性
   - 生成数量: 1
   - 点击"生成"
   - 复制生成的 UUID 卡密

5. **构建 ATM 客户端**
   ```bash
   cd augment-token-mng-main
   cargo tauri build
   # 或开发模式
   npm run dev
   ```

6. **在 ATM 中兑换**
   - 打开 ATM 应用
   - 点击 "Token 列表"
   - 点击 "卡密兑换"
   - 服务器地址: `http://localhost:3000`
   - 粘贴卡密代码
   - 点击 "兑换"
   - 观察进度提示
   - 确认 Session 成功导入

7. **验证结果**
   - 检查 Token 列表中是否出现新的 Token
   - 返回服务端查看卡密状态（应显示已使用）

### 错误场景测试

1. **无效卡密**
   - 输入不存在的卡密
   - 应显示错误: "卡密不存在"

2. **已使用的卡密**
   - 使用已兑换过的一次性卡密
   - 应显示错误: "卡密使用次数已达上限"

3. **服务器不可用**
   - 停止服务端
   - 尝试兑换
   - 应显示网络错误

4. **禁用的卡密**
   - 在服务端禁用一个卡密
   - 尝试兑换
   - 应显示错误: "卡密已被禁用"

## 配置选项

### 服务端配置

**环境变量** (`.env.local`):
```env
ADMIN_PASSWORD=admin123
```

修改管理员密码：直接编辑 `.env.local` 文件

### 客户端配置

用户可以在兑换界面修改服务器地址，支持：
- 本地服务器: `http://localhost:3000`
- 远程服务器: `https://your-domain.com`

## 部署建议

### 开发环境
- 服务端: `npm run dev` (http://localhost:3000)
- 客户端: `npm run dev` 或直接运行构建的应用

### 生产环境

**服务端部署**:
1. 构建项目: `npm run build`
2. 启动服务: `npm start`
3. 配置 HTTPS（推荐使用 Nginx 反向代理）
4. 设置强密码（修改 `.env.local`）
5. 配置防火墙限制管理后台访问

**客户端分发**:
1. 构建应用: `cargo tauri build`
2. 分发安装包:
   - MSI: `src-tauri/target/release/bundle/msi/ATM_*.msi`
   - NSIS: `src-tauri/target/release/bundle/nsis/ATM_*-setup.exe`
3. 告知用户服务器地址

## 安全注意事项

1. **修改默认密码**: 务必修改服务端默认管理员密码
2. **使用 HTTPS**: 生产环境必须使用 HTTPS
3. **限制访问**: 使用防火墙限制管理后台访问
4. **定期备份**: 备份 `data/sessions.db` 数据库
5. **监控日志**: 监控异常的兑换请求

## 故障排查

### 问题: 兑换失败，提示网络错误
**解决**:
- 检查服务端是否正常运行
- 检查服务器地址是否正确
- 检查防火墙设置

### 问题: 卡密兑换成功但 Session 未导入
**解决**:
- 检查 ATM 控制台日志
- 确认 auth_session 格式正确
- 尝试手动导入 Session

### 问题: 服务端登录失败
**解决**:
- 检查 `.env.local` 文件是否存在
- 确认密码哈希正确
- 重启服务端

## 下一步优化建议

1. **服务端**
   - 添加 IP 限流防止暴力破解
   - 支持多管理员账号
   - 添加卡密使用统计图表
   - 支持卡密批量导出

2. **客户端**
   - 保存服务器地址配置
   - 支持扫描二维码兑换
   - 添加兑换历史记录
   - 支持批量兑换

3. **安全性**
   - 实现 HTTPS 强制
   - 添加 CAPTCHA 验证
   - 实现 JWT 认证
   - 添加审计日志

## 相关文档

- [服务端 README](../augment-session-server/README.md)
- [服务端测试指南](../augment-session-server/TESTING.md)
- [ATM 项目 README](README.md)

