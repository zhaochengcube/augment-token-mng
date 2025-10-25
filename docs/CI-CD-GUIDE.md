# CI/CD 自动化指南

本文档说明了项目的完整 CI/CD 自动化流程，包括上游同步、构建和发布。

## 📋 目录

- [工作流概览](#工作流概览)
- [上游同步](#上游同步)
- [版本管理策略](#版本管理策略)
- [构建和发布](#构建和发布)
- [配置要求](#配置要求)
- [使用指南](#使用指南)
- [故障排除](#故障排除)

## 🔄 工作流概览

### 自动化流程图

```
上游仓库更新
    ↓
定时检查 (每天 00:00 UTC)
    ↓
检测到新提交
    ↓
创建同步分支
    ↓
尝试自动合并
    ↓
    ├─→ 无冲突 → 创建 PR → 自动合并 → 版本升级 → 创建 Tag → 触发构建 → 发布 Release
    └─→ 有冲突 → 创建 Issue → 等待人工处理
```

### 工作流文件说明

| 文件 | 触发条件 | 功能 |
|------|---------|------|
| `sync-upstream.yml` | 定时/手动 | 监控并同步上游更新 |
| `build.yml` | Tag 推送 | 多平台构建和发布 |
| `version-bump-and-tag.yml` | 手动 | 手动版本升级 |
| `manual-build.yml` | 手动 | 手动触发构建 |
| `package-dispatch.yml` | Release 发布 | 通知包管理器 |

## 🔄 上游同步

### 自动同步流程

1. **定时检查**
   - 每天 UTC 00:00 自动运行
   - 检查上游仓库 `zhaochengcube/augment-token-mng` 是否有新提交

2. **创建同步分支**
   - 分支命名: `sync-upstream-YYYYMMDD-HHMMSS`
   - 尝试自动合并上游更改

3. **处理合并结果**
   - **无冲突**: 自动创建 PR 并合并
   - **有冲突**: 创建 Issue 通知，需要人工处理

4. **自动发布**
   - PR 合并后自动升级版本号
   - 创建新的 Git Tag
   - 触发构建工作流

### 手动同步

可以通过 GitHub Actions 界面手动触发同步：

1. 进入 **Actions** → **Sync Upstream and Auto Release**
2. 点击 **Run workflow**
3. 配置选项：
   - `force_sync`: 强制同步（即使没有新提交）
   - `auto_merge`: 自动合并 PR
   - `auto_release`: 同步后自动发布

## 📦 版本管理策略

### 版本号格式

采用语义化版本 (Semantic Versioning): `MAJOR.MINOR.PATCH`

```
1.2.3
│ │ │
│ │ └─→ PATCH: 修复 bug、上游同步
│ └───→ MINOR: 新增功能（向后兼容）
└─────→ MAJOR: 重大变更（不向后兼容）
```

### 版本升级规则

#### 自动升级（上游同步）
- 上游同步时自动递增 **PATCH** 版本
- 例如: `1.2.0` → `1.2.1`

#### 手动升级
使用 `version-bump-and-tag.yml` 工作流：

```bash
# Patch 版本 (1.2.0 → 1.2.1)
# 用于: bug 修复、小改进

# Minor 版本 (1.2.0 → 1.3.0)
# 用于: 新功能、向后兼容的改进

# Major 版本 (1.2.0 → 2.0.0)
# 用于: 重大变更、API 不兼容

# Custom 版本
# 用于: 指定特定版本号
```

### 版本同步

版本号需要在以下文件中保持一致：
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**注意**: 工作流会自动同步这些文件的版本号。

### 区分上游版本

#### 方法 1: 版本号后缀（推荐）
如果需要明确区分，可以添加后缀：
- 上游版本: `1.2.0`
- Fork 版本: `1.2.0-fork.1`

修改 `sync-upstream.yml` 中的版本计算逻辑：
```bash
NEW_VERSION="$MAJOR.$MINOR.$PATCH-fork.1"
```

#### 方法 2: 独立版本号
维护独立的版本号序列：
- 上游: `1.x.x`
- Fork: `2.x.x` (从 2.0.0 开始)

#### 方法 3: Git Tag 标记
使用 Git Tag 的描述信息标记来源：
```bash
git tag -a "v1.2.0" -m "Release v1.2.0 (synced from upstream)"
```

## 🏗️ 构建和发布

### 自动构建触发

当推送新的 Tag 时（格式: `v*`），自动触发构建：

```bash
git tag v1.2.0
git push origin v1.2.0
```

### 构建平台

支持以下平台的自动构建：

| 平台 | 架构 | 产物 |
|------|------|------|
| Windows | x64 | `.msi`, `.exe` |
| macOS | Apple Silicon (M1/M2/M3) | `.dmg`, `.app` |
| macOS | Intel | `.dmg`, `.app` |
| Linux | x64 | `.deb`, `.AppImage` |

### Release 发布流程

1. **自动生成 Changelog**
   - 从上一个 Tag 到当前 Tag 的所有提交
   - 自动分类和格式化

2. **构建所有平台**
   - 并行构建 4 个平台
   - 使用 Rust 缓存加速

3. **创建 Draft Release**
   - 上传所有构建产物
   - 包含完整的 Changelog
   - 状态为 Draft（草稿）

4. **人工审核**
   - 检查构建产物
   - 验证 Changelog
   - 发布 Release

5. **通知包管理器**
   - 自动通知 Homebrew Tap
   - 自动通知 Scoop Bucket

## ⚙️ 配置要求

### GitHub Secrets

需要配置以下 Secrets（在仓库 Settings → Secrets and variables → Actions）：

| Secret | 用途 | 必需 |
|--------|------|------|
| `GITHUB_TOKEN` | 自动提供，用于基本操作 | ✅ |
| `TAP_DISPATCH_TOKEN` | 通知包管理器仓库 | ⚠️ 可选 |
| `TELEGRAM_BOT_TOKEN` | Telegram 通知 | ⚠️ 可选 |
| `TELEGRAM_CHAT_ID` | Telegram 聊天 ID | ⚠️ 可选 |

### 创建 Personal Access Token (PAT)

如果需要 `TAP_DISPATCH_TOKEN`:

1. 进入 GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. 点击 **Generate new token (classic)**
3. 设置权限:
   - `repo` (完整权限)
   - `workflow`
4. 复制 Token 并添加到仓库 Secrets

### 配置上游仓库

确保上游仓库 URL 正确（在 `sync-upstream.yml` 中）：

```yaml
git remote add upstream https://github.com/zhaochengcube/augment-token-mng.git
```

## 📖 使用指南

### 场景 1: 自动同步上游更新

**无需操作** - 系统每天自动检查并同步。

查看同步状态:
1. 进入 **Actions** 标签
2. 查看 **Sync Upstream and Auto Release** 工作流
3. 检查最近的运行记录

### 场景 2: 手动同步上游

1. 进入 **Actions** → **Sync Upstream and Auto Release**
2. 点击 **Run workflow**
3. 选择选项:
   - ✅ `auto_merge`: 自动合并
   - ✅ `auto_release`: 自动发布
4. 点击 **Run workflow**

### 场景 3: 手动发布新版本

1. 进入 **Actions** → **Version Bump and Tag**
2. 点击 **Run workflow**
3. 选择版本类型:
   - `patch`: 1.2.0 → 1.2.1
   - `minor`: 1.2.0 → 1.3.0
   - `major`: 1.2.0 → 2.0.0
   - `custom`: 指定版本号
4. 点击 **Run workflow**
5. 等待构建完成
6. 进入 **Releases** 审核并发布

### 场景 4: 处理合并冲突

当自动同步遇到冲突时:

1. 查看自动创建的 Issue
2. 本地解决冲突:
   ```bash
   # 克隆仓库
   git clone <your-repo>
   cd <repo-name>
   
   # 添加上游
   git remote add upstream https://github.com/zhaochengcube/augment-token-mng.git
   git fetch upstream
   
   # 合并并解决冲突
   git merge upstream/main
   
   # 解决冲突后
   git add .
   git commit -m "chore: resolve merge conflicts with upstream"
   git push origin main
   ```
3. 关闭 Issue
4. 手动触发版本发布

## 🔧 故障排除

### 问题 1: 同步失败

**症状**: Sync workflow 失败

**解决方案**:
1. 检查上游仓库是否可访问
2. 检查 GitHub Token 权限
3. 查看 workflow 日志获取详细错误

### 问题 2: 构建失败

**症状**: Build workflow 失败

**解决方案**:
1. 检查依赖是否正确安装
2. 查看特定平台的构建日志
3. 本地测试构建:
   ```bash
   npm install
   npm run tauri build
   ```

### 问题 3: 版本号不一致

**症状**: 不同文件中的版本号不匹配

**解决方案**:
1. 使用 `version-bump-and-tag.yml` 工作流统一更新
2. 或手动同步:
   ```bash
   # 获取当前版本
   VERSION=$(node -p "require('./package.json').version")
   
   # 更新 Cargo.toml
   sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
   
   # 更新 tauri.conf.json
   node -e "
     const fs = require('fs');
     const config = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
     config.version = '$VERSION';
     fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(config, null, 2) + '\n');
   "
   ```

### 问题 4: PR 未自动合并

**症状**: 同步 PR 创建但未自动合并

**解决方案**:
1. 检查是否启用了 `auto_merge` 选项
2. 检查仓库设置是否允许自动合并
3. 手动合并 PR

### 问题 5: Release 未自动创建

**症状**: Tag 推送后没有创建 Release

**解决方案**:
1. 检查 Tag 格式是否为 `v*` (如 `v1.2.0`)
2. 检查 `build.yml` workflow 是否启用
3. 查看 Actions 日志

## 📚 相关资源

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri 构建指南](https://tauri.app/v1/guides/building/)
- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [上游仓库](https://github.com/zhaochengcube/augment-token-mng)

## 🤝 贡献

如果你发现 CI/CD 流程有改进空间，欢迎:
1. 创建 Issue 讨论
2. 提交 Pull Request
3. 更新本文档

---

**最后更新**: 2025-10-25
**维护者**: GitHub Actions Bot

