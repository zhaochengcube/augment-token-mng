# 解决首次同步的合并冲突 (PowerShell 版本)
# 策略: 接受所有上游版本（因为我们主要添加的是 CI/CD 文件）

$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Blue
Write-Host "解决首次同步冲突" -ForegroundColor Blue
Write-Host "=========================================" -ForegroundColor Blue
Write-Host ""

# 检查是否在合并状态
$status = git status
if ($status -notmatch "You have unmerged paths") {
    Write-Host "❌ 当前不在合并冲突状态" -ForegroundColor Red
    Write-Host "请先运行: git merge upstream/main --allow-unrelated-histories"
    exit 1
}

Write-Host "📋 冲突文件列表:" -ForegroundColor Cyan
git diff --name-only --diff-filter=U
Write-Host ""

Write-Host "🔧 解决策略: 接受上游版本（保留我们的 CI/CD 文件）" -ForegroundColor Yellow
Write-Host ""

# 接受上游版本的文件
Write-Host "✅ 接受上游版本..." -ForegroundColor Green

# 配置文件
git checkout --theirs package.json
git checkout --theirs package-lock.json
git checkout --theirs src-tauri/capabilities/default.json
git checkout --theirs src-tauri/gen/schemas/capabilities.json
git checkout --theirs src-tauri/gen/schemas/macOS-schema.json

# 源代码文件
git checkout --theirs src-tauri/src/augment_oauth.rs
git checkout --theirs src-tauri/src/augment_user_info.rs
git checkout --theirs src-tauri/src/main.rs
git checkout --theirs src/App.vue
git checkout --theirs src/components/TokenCard.vue
git checkout --theirs src/components/TokenList.vue
git checkout --theirs src/locales/en-US.js
git checkout --theirs src/locales/zh-CN.js

Write-Host "✅ 所有冲突文件已解决" -ForegroundColor Green
Write-Host ""

# 标记为已解决
Write-Host "📝 标记文件为已解决..." -ForegroundColor Cyan
git add package.json
git add package-lock.json
git add src-tauri/capabilities/default.json
git add src-tauri/gen/schemas/capabilities.json
git add src-tauri/gen/schemas/macOS-schema.json
git add src-tauri/src/augment_oauth.rs
git add src-tauri/src/augment_user_info.rs
git add src-tauri/src/main.rs
git add src/App.vue
git add src/components/TokenCard.vue
git add src/components/TokenList.vue
git add src/locales/en-US.js
git add src/locales/zh-CN.js

Write-Host "✅ 文件已标记为已解决" -ForegroundColor Green
Write-Host ""

# 检查是否还有未解决的冲突
$unresolvedConflicts = git diff --name-only --diff-filter=U
if ($unresolvedConflicts) {
    Write-Host "⚠️ 还有未解决的冲突:" -ForegroundColor Yellow
    Write-Host $unresolvedConflicts
    exit 1
}

Write-Host "🎉 所有冲突已解决！" -ForegroundColor Green
Write-Host ""
Write-Host "📋 下一步:" -ForegroundColor Cyan
Write-Host "1. 检查合并结果: git status"
Write-Host "2. 提交合并: git commit -m 'chore: merge upstream/main'"
Write-Host "3. 推送代码: git push origin main"
Write-Host ""

