# CI/CD 自动化系统

本项目实现了完整的 CI/CD 自动化流程，支持自动监控上游更新、代码同步、构建和发布。

## ✨ 功能特性

### 🔄 自动上游同步
- ✅ 每天自动检查上游仓库更新
- ✅ 自动创建同步分支并尝试合并
- ✅ 无冲突时自动创建并合并 PR
- ✅ 有冲突时自动创建 Issue 通知

### 📦 自动版本管理
- ✅ 自动递增版本号
- ✅ 支持多种版本策略（后缀/独立/标准）
- ✅ 自动同步多个配置文件的版本号
- ✅ 自动创建 Git Tag

### 🏗️ 自动构建发布
- ✅ 多平台并行构建（Windows/macOS/Linux）
- ✅ 自动生成 Changelog
- ✅ 自动创建 GitHub Release
- ✅ 自动通知包管理器

### 🔔 通知系统
- ✅ Telegram 通知（可选）
- ✅ GitHub Issue 通知
- ✅ 构建状态通知

## 🚀 快速开始

### 1. 运行设置脚本

**Linux/macOS:**
```bash
chmod +x scripts/setup-ci-cd.sh
./scripts/setup-ci-cd.sh
```

**Windows PowerShell:**
```powershell
.\scripts\setup-ci-cd.ps1
```

### 2. 配置 GitHub Secrets（可选）

进入仓库设置 → Secrets and variables → Actions，添加以下 Secrets：

| Secret | 必需 | 用途 |
|--------|------|------|
| `GITHUB_TOKEN` | ✅ | 自动提供，无需配置 |
| `TAP_DISPATCH_TOKEN` | ⚠️ | 通知包管理器（可选） |
| `TELEGRAM_BOT_TOKEN` | ⚠️ | Telegram 通知（可选） |
| `TELEGRAM_CHAT_ID` | ⚠️ | Telegram 聊天 ID（可选） |

### 3. 测试工作流

进入 **Actions** → **Sync Upstream and Auto Release** → **Run workflow**

## 📋 工作流说明

### 1. Sync Upstream and Auto Release

**触发条件:**
- 定时: 每天 UTC 00:00
- 手动: GitHub Actions UI

**功能:**
- 检查上游更新
- 自动同步代码
- 自动升级版本
- 自动创建 Tag
- 触发构建

**配置选项:**
- `force_sync`: 强制同步
- `auto_merge`: 自动合并 PR
- `auto_release`: 自动发布版本

### 2. Build and Release

**触发条件:**
- Tag 推送: `v*`
- 手动: GitHub Actions UI

**功能:**
- 多平台构建
- 生成 Changelog
- 创建 Draft Release
- 上传构建产物

**支持平台:**
- Windows (x64)
- macOS (Apple Silicon)
- macOS (Intel)
- Linux (x64)

### 3. Version Bump and Tag

**触发条件:**
- 手动: GitHub Actions UI

**功能:**
- 手动升级版本号
- 支持 patch/minor/major/custom
- 自动同步配置文件
- 自动创建 Tag

### 4. Manual Build

**触发条件:**
- 手动: GitHub Actions UI

**功能:**
- 手动触发构建
- 支持选择特定平台
- 上传构建产物

## 📖 版本管理策略

### 推荐策略: 版本号后缀

```
上游版本: 1.2.0
Fork 版本: 1.2.0-fork.1, 1.2.0-fork.2, ...
```

**优点:**
- ✅ 清晰标识 Fork 版本
- ✅ 保持与上游版本的对应关系
- ✅ 支持多次迭代

**版本演进示例:**
```
上游 1.0.0 → Fork 1.0.0-fork.1
Fork 功能  → Fork 1.0.0-fork.2
上游 1.1.0 → Fork 1.1.0-fork.1
```

详见: [版本管理策略文档](./VERSION-STRATEGY.md)

## 🔄 使用场景

### 场景 1: 自动同步上游（推荐）

**无需操作** - 系统每天自动运行

1. 每天 UTC 00:00 自动检查上游
2. 发现更新后自动创建同步分支
3. 无冲突时自动合并
4. 自动升级版本并创建 Tag
5. 自动触发构建
6. 构建完成后创建 Draft Release

### 场景 2: 手动同步上游

1. Actions → Sync Upstream and Auto Release
2. Run workflow
3. 配置选项:
   - ✅ `auto_merge`
   - ✅ `auto_release`
4. 等待完成

### 场景 3: 手动发布新版本

1. Actions → Version Bump and Tag
2. Run workflow
3. 选择版本类型:
   - `patch`: 1.2.0 → 1.2.1
   - `minor`: 1.2.0 → 1.3.0
   - `major`: 1.2.0 → 2.0.0
   - `custom`: 指定版本
4. 等待构建完成
5. Releases → 审核并发布

### 场景 4: 处理合并冲突

当自动同步遇到冲突:

1. 查看自动创建的 Issue
2. 本地解决冲突:
   ```bash
   git fetch upstream
   git merge upstream/main
   # 解决冲突
   git add .
   git commit
   git push
   ```
3. 关闭 Issue
4. 手动触发版本发布

## 📊 工作流程图

```
┌─────────────────┐
│  上游仓库更新    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  定时检查        │
│  (每天 00:00)    │
└────────┬────────┘
         │
         ▼
    ┌────────┐
    │有更新？ │
    └───┬────┘
        │ Yes
        ▼
┌─────────────────┐
│  创建同步分支    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  尝试自动合并    │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
   无冲突    有冲突
    │         │
    ▼         ▼
┌────────┐ ┌────────┐
│创建 PR │ │创建Issue│
└───┬────┘ └────────┘
    │
    ▼
┌────────┐
│自动合并 │
└───┬────┘
    │
    ▼
┌────────┐
│版本升级 │
└───┬────┘
    │
    ▼
┌────────┐
│创建 Tag │
└───┬────┘
    │
    ▼
┌────────┐
│触发构建 │
└───┬────┘
    │
    ▼
┌────────────────┐
│ 多平台并行构建  │
│ • Windows      │
│ • macOS (AS)   │
│ • macOS (Intel)│
│ • Linux        │
└───┬────────────┘
    │
    ▼
┌────────────────┐
│ 创建 Release   │
│ (Draft)        │
└───┬────────────┘
    │
    ▼
┌────────────────┐
│ 人工审核发布   │
└────────────────┘
```

## 🔧 配置文件

### 工作流文件

| 文件 | 说明 |
|------|------|
| `.github/workflows/sync-upstream.yml` | 上游同步 |
| `.github/workflows/build.yml` | 构建发布 |
| `.github/workflows/version-bump-and-tag.yml` | 版本管理 |
| `.github/workflows/manual-build.yml` | 手动构建 |
| `.github/workflows/package-dispatch.yml` | 包管理器通知 |

### 版本配置文件

| 文件 | 说明 |
|------|------|
| `package.json` | npm 版本 |
| `src-tauri/Cargo.toml` | Rust 版本 |
| `src-tauri/tauri.conf.json` | Tauri 版本 |

**注意**: 所有版本号必须保持一致，工作流会自动同步。

## 📚 文档

- [完整 CI/CD 指南](./CI-CD-GUIDE.md) - 详细的配置和使用说明
- [版本管理策略](./VERSION-STRATEGY.md) - 版本号管理详解
- [快速参考](./QUICK-REFERENCE.md) - 常用命令速查

## 🆘 故障排除

### 问题 1: 同步失败

**检查:**
- 上游仓库是否可访问
- GitHub Token 权限
- 工作流日志

### 问题 2: 构建失败

**检查:**
- 依赖是否正确
- 平台特定的构建日志
- 本地测试构建

### 问题 3: 版本号不一致

**解决:**
使用 `Version Bump and Tag` 工作流统一更新

### 问题 4: PR 未自动合并

**检查:**
- `auto_merge` 选项是否启用
- 仓库设置是否允许自动合并
- 是否有合并冲突

详见: [故障排除指南](./CI-CD-GUIDE.md#故障排除)

## 🤝 贡献

欢迎改进 CI/CD 流程！

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

## 📄 许可证

本项目基于 [augment-token-mng](https://github.com/zhaochengcube/augment-token-mng) 进行二次开发。

## 🔗 相关链接

- [上游仓库](https://github.com/zhaochengcube/augment-token-mng)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri 文档](https://tauri.app/)

---

**最后更新**: 2025-10-25
**维护者**: GitHub Actions Bot

