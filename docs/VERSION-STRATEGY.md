# 版本管理策略

本文档详细说明了项目的版本管理策略，包括如何区分上游版本和 Fork 版本。

## 📋 目录

- [版本号规范](#版本号规范)
- [版本升级策略](#版本升级策略)
- [区分上游版本的方法](#区分上游版本的方法)
- [实施建议](#实施建议)
- [版本历史追踪](#版本历史追踪)

## 🔢 版本号规范

### 语义化版本 (Semantic Versioning)

采用 `MAJOR.MINOR.PATCH` 格式:

```
1.2.3
│ │ │
│ │ └─→ PATCH: 修复 bug、上游同步、小改进
│ └───→ MINOR: 新增功能（向后兼容）
└─────→ MAJOR: 重大变更（不向后兼容）
```

### 版本号示例

| 变更类型 | 示例 | 说明 |
|---------|------|------|
| Bug 修复 | 1.2.0 → 1.2.1 | 修复已知问题 |
| 上游同步 | 1.2.1 → 1.2.2 | 同步上游更新 |
| 新功能 | 1.2.2 → 1.3.0 | 添加新功能 |
| 重大变更 | 1.3.0 → 2.0.0 | API 变更、架构调整 |

## 📈 版本升级策略

### 自动升级（推荐）

#### 1. 上游同步自动升级

当检测到上游更新时，自动递增 PATCH 版本:

```yaml
# 在 sync-upstream.yml 中
PATCH=$((PATCH + 1))
NEW_VERSION="$MAJOR.$MINOR.$PATCH"
```

**示例流程**:
```
当前版本: 1.2.0
上游有新提交 → 自动同步 → 版本升级到 1.2.1
```

#### 2. 手动版本升级

使用 GitHub Actions 工作流 `version-bump-and-tag.yml`:

```bash
# 通过 GitHub Actions UI 触发
Actions → Version Bump and Tag → Run workflow
```

选项:
- **patch**: 递增 PATCH (1.2.0 → 1.2.1)
- **minor**: 递增 MINOR (1.2.0 → 1.3.0)
- **major**: 递增 MAJOR (1.2.0 → 2.0.0)
- **custom**: 指定版本号 (如 1.5.0)

### 手动升级（本地）

如果需要本地手动升级版本:

```bash
# 1. 更新 package.json
npm version patch  # 或 minor, major

# 2. 更新 Cargo.toml
VERSION=$(node -p "require('./package.json').version")
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# 3. 更新 tauri.conf.json
node -e "
  const fs = require('fs');
  const config = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  config.version = '$VERSION';
  fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(config, null, 2) + '\n');
"

# 4. 提交并创建 Tag
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to $VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin main --tags
```

## 🔀 区分上游版本的方法

### 方法 1: 版本号后缀（推荐）⭐

在版本号后添加 `-fork` 后缀来区分:

```
上游版本: 1.2.0
Fork 版本: 1.2.0-fork.1, 1.2.0-fork.2, ...
```

#### 优点
- ✅ 清晰标识 Fork 版本
- ✅ 保持与上游版本的对应关系
- ✅ 支持多次 Fork 迭代

#### 实施方式

修改 `sync-upstream.yml` 中的版本计算:

```yaml
- name: Calculate new version
  id: version
  run: |
    CURRENT_VERSION=$(node -p "require('./package.json').version")
    
    # 提取基础版本号（去除 -fork 后缀）
    BASE_VERSION=$(echo "$CURRENT_VERSION" | sed 's/-fork\.[0-9]*$//')
    
    # 获取上游版本
    UPSTREAM_VERSION=$(git show upstream/main:package.json | node -p "JSON.parse(require('fs').readFileSync('/dev/stdin', 'utf8')).version")
    
    # 如果上游版本更新，使用上游版本 + fork.1
    # 否则递增 fork 序号
    if [ "$BASE_VERSION" != "$UPSTREAM_VERSION" ]; then
      NEW_VERSION="$UPSTREAM_VERSION-fork.1"
    else
      # 提取当前 fork 序号
      FORK_NUM=$(echo "$CURRENT_VERSION" | grep -oP 'fork\.\K[0-9]+' || echo "0")
      FORK_NUM=$((FORK_NUM + 1))
      NEW_VERSION="$BASE_VERSION-fork.$FORK_NUM"
    fi
    
    echo "new_version=$NEW_VERSION" >> $GITHUB_OUTPUT
```

#### 版本演进示例

```
上游 1.0.0 → Fork 1.0.0-fork.1
Fork 自定义功能 → Fork 1.0.0-fork.2
上游 1.1.0 → Fork 1.1.0-fork.1
Fork 修复 bug → Fork 1.1.0-fork.2
```

### 方法 2: 独立版本号序列

维护完全独立的版本号:

```
上游版本: 1.x.x
Fork 版本: 2.x.x (从 2.0.0 开始)
```

#### 优点
- ✅ 完全独立的版本管理
- ✅ 避免与上游版本混淆

#### 缺点
- ❌ 难以追踪对应的上游版本
- ❌ 需要维护版本映射关系

#### 实施方式

在 `package.json` 中设置起始版本:

```json
{
  "version": "2.0.0"
}
```

在 `sync-upstream.yml` 中保持正常的版本递增逻辑。

#### 版本映射表

维护一个映射表记录对应关系:

| Fork 版本 | 上游版本 | 说明 |
|----------|---------|------|
| 2.0.0 | 1.0.0 | 初始 Fork |
| 2.0.1 | 1.0.0 | Fork 修复 |
| 2.1.0 | 1.1.0 | 同步上游 1.1.0 |
| 2.1.1 | 1.1.0 | Fork 功能 |

### 方法 3: Git Tag 描述标记

使用标准版本号，但在 Git Tag 描述中标记来源:

```bash
# 上游同步
git tag -a "v1.2.0" -m "Release v1.2.0 (synced from upstream v1.2.0)"

# Fork 功能
git tag -a "v1.2.1" -m "Release v1.2.1 (fork: added custom feature)"
```

#### 优点
- ✅ 版本号简洁
- ✅ 通过 Tag 描述追踪来源

#### 缺点
- ❌ 需要查看 Tag 描述才能区分
- ❌ 不够直观

### 方法 4: 分支策略

使用不同的分支管理:

```
main          - Fork 主分支
upstream-sync - 上游同步分支
feature/*     - Fork 功能分支
```

#### 工作流程

```bash
# 1. 同步上游到 upstream-sync 分支
git checkout upstream-sync
git pull upstream main
git push origin upstream-sync

# 2. 合并到 main 并添加 Fork 功能
git checkout main
git merge upstream-sync
# 添加自定义功能
git commit -m "feat: add custom feature"

# 3. 发布版本
git tag -a "v1.2.0-fork.1" -m "Release with custom features"
```

## 💡 实施建议

### 推荐方案: 方法 1（版本号后缀）

**理由**:
1. ✅ 清晰标识 Fork 版本
2. ✅ 保持与上游版本的对应关系
3. ✅ 易于理解和维护
4. ✅ 支持自动化

### 实施步骤

#### 1. 修改自动同步工作流

编辑 `.github/workflows/sync-upstream.yml`:

```yaml
- name: Calculate new version
  id: version
  run: |
    # 使用方法 1 的版本计算逻辑
    # (见上文"方法 1"部分)
```

#### 2. 更新当前版本

如果当前版本是 `1.2.0`，更新为 `1.2.0-fork.1`:

```bash
# 手动更新
npm version 1.2.0-fork.1 --no-git-tag-version

# 同步其他文件
VERSION="1.2.0-fork.1"
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
# ... (更新 tauri.conf.json)

# 提交
git add .
git commit -m "chore: adopt fork version strategy"
git tag -a "v1.2.0-fork.1" -m "Release v1.2.0-fork.1"
git push origin main --tags
```

#### 3. 文档更新

在 `README.md` 中说明版本策略:

```markdown
## 版本说明

本项目基于 [augment-token-mng](https://github.com/zhaochengcube/augment-token-mng) 进行二次开发。

版本号格式: `X.Y.Z-fork.N`
- `X.Y.Z`: 对应的上游版本
- `fork.N`: Fork 版本序号

示例:
- `1.2.0-fork.1`: 基于上游 1.2.0 的第一个 Fork 版本
- `1.2.0-fork.2`: 基于上游 1.2.0 的第二个 Fork 版本
```

## 📊 版本历史追踪

### 使用 Git Tags

查看所有版本:

```bash
git tag -l -n1
```

输出示例:
```
v1.0.0-fork.1    Release v1.0.0-fork.1 (initial fork)
v1.0.0-fork.2    Release v1.0.0-fork.2 (added feature X)
v1.1.0-fork.1    Release v1.1.0-fork.1 (synced upstream 1.1.0)
```

### 使用 CHANGELOG.md

维护详细的变更日志:

```markdown
# Changelog

## [1.1.0-fork.1] - 2025-10-25
### Synced from Upstream
- Synced with upstream v1.1.0
- Includes all changes from upstream

### Fork Changes
- Maintained custom database integration
- Maintained custom email features

## [1.0.0-fork.2] - 2025-10-20
### Added
- Custom feature X
- Custom feature Y

## [1.0.0-fork.1] - 2025-10-15
### Initial Fork
- Forked from upstream v1.0.0
- Added custom database support
```

### 使用 GitHub Releases

在 Release 描述中明确标注:

```markdown
## 🎉 Release v1.2.0-fork.1

### 📦 基于上游版本
- Upstream: v1.2.0
- Fork 序号: 1

### 🔄 上游更新
- Feature A from upstream
- Bug fix B from upstream

### ⭐ Fork 特性
- Custom database integration
- Custom email automation
- Enhanced UI features

### 📥 下载
...
```

## 🔄 版本同步检查清单

每次发布新版本时，确保:

- [ ] `package.json` 版本已更新
- [ ] `src-tauri/Cargo.toml` 版本已更新
- [ ] `src-tauri/tauri.conf.json` 版本已更新
- [ ] `src-tauri/Cargo.lock` 已更新
- [ ] Git Tag 已创建（格式: `v1.2.0-fork.1`）
- [ ] CHANGELOG.md 已更新
- [ ] GitHub Release 已创建
- [ ] Release 描述包含上游版本信息

## 📚 参考资源

- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [Git Tag 最佳实践](https://git-scm.com/book/zh/v2/Git-基础-打标签)
- [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)

---

**最后更新**: 2025-10-25

