# CI/CD 快速参考

快速查找常用命令和操作的参考文档。

## 🚀 快速开始

### 初始设置

```bash
# Linux/macOS
chmod +x scripts/setup-ci-cd.sh
./scripts/setup-ci-cd.sh

# Windows PowerShell
.\scripts\setup-ci-cd.ps1
```

## 📋 常用命令

### Git 操作

```bash
# 添加上游仓库
git remote add upstream https://github.com/zhaochengcube/augment-token-mng.git

# 获取上游更新
git fetch upstream

# 查看上游更新
git log HEAD..upstream/main --oneline

# 手动合并上游
git merge upstream/main

# 查看所有远程仓库
git remote -v
```

### 版本管理

```bash
# 查看当前版本
node -p "require('./package.json').version"

# 手动更新版本号（使用 npm）
npm version patch  # 1.2.0 → 1.2.1
npm version minor  # 1.2.0 → 1.3.0
npm version major  # 1.2.0 → 2.0.0

# 创建并推送 Tag
VERSION=$(node -p "require('./package.json').version")
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin "v$VERSION"

# 查看所有 Tags
git tag -l -n1

# 删除本地 Tag
git tag -d v1.2.0

# 删除远程 Tag
git push origin --delete v1.2.0
```

### 版本号同步

```bash
# 获取 package.json 版本
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

# 提交版本更新
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to $VERSION"
```

### GitHub CLI 操作

```bash
# 登录 GitHub CLI
gh auth login

# 查看仓库信息
gh repo view

# 列出 Pull Requests
gh pr list

# 查看 PR 详情
gh pr view 123

# 合并 PR
gh pr merge 123 --merge

# 列出 Issues
gh issue list

# 创建 Issue
gh issue create --title "标题" --body "内容"

# 查看 Actions 运行状态
gh run list

# 查看特定 workflow 运行
gh run view 123456

# 手动触发 workflow
gh workflow run "Sync Upstream and Auto Release"

# 列出 Releases
gh release list

# 查看 Release 详情
gh release view v1.2.0
```

## 🔄 工作流操作

### 手动触发同步

**方法 1: GitHub Web UI**
1. 进入仓库 → Actions
2. 选择 "Sync Upstream and Auto Release"
3. 点击 "Run workflow"
4. 配置选项并运行

**方法 2: GitHub CLI**
```bash
gh workflow run "Sync Upstream and Auto Release" \
  -f force_sync=false \
  -f auto_merge=true \
  -f auto_release=true
```

### 手动发布版本

**方法 1: GitHub Web UI**
1. 进入仓库 → Actions
2. 选择 "Version Bump and Tag"
3. 点击 "Run workflow"
4. 选择版本类型（patch/minor/major/custom）
5. 运行

**方法 2: GitHub CLI**
```bash
# Patch 版本
gh workflow run "Version Bump and Tag" -f version_type=patch

# Minor 版本
gh workflow run "Version Bump and Tag" -f version_type=minor

# Major 版本
gh workflow run "Version Bump and Tag" -f version_type=major

# 自定义版本
gh workflow run "Version Bump and Tag" \
  -f version_type=custom \
  -f custom_version=1.5.0
```

### 手动构建

```bash
# 构建所有平台
gh workflow run "Manual Build" -f platform=all

# 构建特定平台
gh workflow run "Manual Build" -f platform=windows
gh workflow run "Manual Build" -f platform=macos-intel
gh workflow run "Manual Build" -f platform=macos-apple-silicon
gh workflow run "Manual Build" -f platform=linux
```

## 🔧 故障排除

### 检查工作流状态

```bash
# 查看最近的运行
gh run list --limit 10

# 查看特定运行的日志
gh run view 123456 --log

# 查看失败的运行
gh run list --status failure
```

### 处理合并冲突

```bash
# 1. 获取上游更新
git fetch upstream

# 2. 创建临时分支
git checkout -b fix-merge-conflict

# 3. 尝试合并
git merge upstream/main

# 4. 查看冲突文件
git status

# 5. 手动解决冲突后
git add .
git commit -m "chore: resolve merge conflicts"

# 6. 推送并创建 PR
git push origin fix-merge-conflict
gh pr create --title "解决合并冲突" --body "手动解决上游同步冲突"
```

### 重新运行失败的工作流

```bash
# 查看失败的运行
gh run list --status failure

# 重新运行
gh run rerun 123456

# 重新运行失败的 jobs
gh run rerun 123456 --failed
```

### 取消运行中的工作流

```bash
# 查看运行中的工作流
gh run list --status in_progress

# 取消运行
gh run cancel 123456
```

## 📦 构建相关

### 本地构建

```bash
# 安装依赖
npm install

# 开发模式
npm run dev

# 构建前端
npm run build

# 构建 Tauri 应用
npm run tauri build

# 构建特定平台（需要交叉编译工具）
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

### 清理构建产物

```bash
# 清理前端构建
rm -rf dist/

# 清理 Tauri 构建
rm -rf src-tauri/target/

# 清理 node_modules
rm -rf node_modules/

# 重新安装依赖
npm install
```

## 🔐 Secrets 配置

### 必需的 Secrets

| Secret | 用途 | 获取方式 |
|--------|------|---------|
| `GITHUB_TOKEN` | 自动提供 | 无需配置 |

### 可选的 Secrets

| Secret | 用途 | 获取方式 |
|--------|------|---------|
| `TAP_DISPATCH_TOKEN` | 通知包管理器 | GitHub PAT |
| `TELEGRAM_BOT_TOKEN` | Telegram 通知 | @BotFather |
| `TELEGRAM_CHAT_ID` | Telegram 聊天 ID | @userinfobot |

### 配置 Secrets

```bash
# 使用 GitHub CLI
gh secret set TAP_DISPATCH_TOKEN < token.txt

# 或通过 Web UI
# 1. 仓库 → Settings → Secrets and variables → Actions
# 2. New repository secret
# 3. 输入名称和值
```

## 📊 监控和日志

### 查看工作流运行历史

```bash
# 所有工作流
gh run list

# 特定工作流
gh run list --workflow="Sync Upstream and Auto Release"

# 最近 20 次运行
gh run list --limit 20

# 查看详细信息
gh run view 123456
```

### 下载构建产物

```bash
# 列出产物
gh run view 123456 --log

# 下载所有产物
gh run download 123456

# 下载特定产物
gh run download 123456 -n windows-build
```

## 🔗 有用的链接

### 仓库链接

```bash
# 获取仓库 URL
gh repo view --web

# 获取 Actions 页面
gh repo view --web --branch main | sed 's/$/\/actions/'

# 获取 Releases 页面
gh release list --web
```

### 快速访问

- **Actions**: `https://github.com/YOUR_USERNAME/YOUR_REPO/actions`
- **Releases**: `https://github.com/YOUR_USERNAME/YOUR_REPO/releases`
- **Settings**: `https://github.com/YOUR_USERNAME/YOUR_REPO/settings`
- **Secrets**: `https://github.com/YOUR_USERNAME/YOUR_REPO/settings/secrets/actions`

## 📝 常用脚本

### 一键同步上游

```bash
#!/bin/bash
# sync-upstream.sh

git fetch upstream
git checkout main
git merge upstream/main
git push origin main

echo "✅ 上游同步完成"
```

### 一键发布

```bash
#!/bin/bash
# release.sh

# 获取当前版本
CURRENT_VERSION=$(node -p "require('./package.json').version")
echo "当前版本: $CURRENT_VERSION"

# 询问新版本
read -p "新版本号: " NEW_VERSION

# 更新版本
npm version $NEW_VERSION --no-git-tag-version

# 同步版本号
VERSION=$NEW_VERSION
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
node -e "
  const fs = require('fs');
  const config = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  config.version = '$VERSION';
  fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(config, null, 2) + '\n');
"

# 提交并创建 Tag
git add .
git commit -m "chore: bump version to $VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin main --tags

echo "✅ 版本 $VERSION 已发布"
```

## 🆘 获取帮助

### 文档

- [CI/CD 完整指南](./CI-CD-GUIDE.md)
- [版本管理策略](./VERSION-STRATEGY.md)

### 命令帮助

```bash
# Git 帮助
git help <command>

# GitHub CLI 帮助
gh help
gh workflow help
gh release help

# npm 帮助
npm help version
```

### 在线资源

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [GitHub CLI 文档](https://cli.github.com/manual/)
- [Tauri 文档](https://tauri.app/)
- [语义化版本](https://semver.org/lang/zh-CN/)

---

**提示**: 将常用命令添加到 shell 别名或创建自定义脚本以提高效率。

