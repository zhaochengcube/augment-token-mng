# 本地测试上游同步流程 (PowerShell 版本)
# 在推送到 GitHub Actions 之前，先在本地验证同步流程

$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Blue
Write-Host "本地测试上游同步流程" -ForegroundColor Blue
Write-Host "=========================================" -ForegroundColor Blue
Write-Host ""

# 检查是否在 Git 仓库中
$gitCheck = git rev-parse --git-dir 2>$null
if (-not $gitCheck) {
    Write-Host "❌ 当前目录不是 Git 仓库" -ForegroundColor Red
    exit 1
}

# 检查是否配置了 upstream
$upstreamCheck = git remote get-url upstream 2>$null
if (-not $upstreamCheck) {
    Write-Host "❌ 未配置 upstream 远程仓库" -ForegroundColor Red
    Write-Host ""
    Write-Host "请先配置 upstream:"
    Write-Host "  git remote add upstream https://github.com/zhaochengcube/augment-token-mng.git"
    exit 1
}

Write-Host "✅ Git 仓库检查通过" -ForegroundColor Green
Write-Host ""

# 保存当前分支
$currentBranch = git branch --show-current
Write-Host "📍 当前分支: $currentBranch" -ForegroundColor Cyan
Write-Host ""

# 检查工作区是否干净
$status = git status --porcelain
if ($status) {
    Write-Host "⚠️  工作区有未提交的更改" -ForegroundColor Yellow
    Write-Host ""
    git status --short
    Write-Host ""
    $response = Read-Host "是否继续? (y/N)"
    if ($response -notmatch '^[Yy]$') {
        Write-Host "❌ 已取消" -ForegroundColor Red
        exit 0
    }
}

Write-Host "🔄 步骤 1: 获取上游更新" -ForegroundColor Cyan
Write-Host "----------------------------------------"
git fetch upstream
Write-Host "✅ 完成" -ForegroundColor Green
Write-Host ""

Write-Host "📊 步骤 2: 检查上游更新" -ForegroundColor Cyan
Write-Host "----------------------------------------"

# 检查是否有新提交
$upstreamCommits = git log HEAD..upstream/main --oneline

if (-not $upstreamCommits) {
    Write-Host "✅ 没有新的上游提交" -ForegroundColor Green
    Write-Host ""
    Write-Host "您的代码已经是最新的！"
    exit 0
}

Write-Host "发现新的上游提交:" -ForegroundColor Yellow
Write-Host ""
Write-Host $upstreamCommits
Write-Host ""

# 统计更改
$commitsCount = ($upstreamCommits | Measure-Object -Line).Lines
Write-Host "📈 统计: $commitsCount 个新提交" -ForegroundColor Cyan
Write-Host ""

Write-Host "📋 步骤 3: 查看文件变更" -ForegroundColor Cyan
Write-Host "----------------------------------------"
git diff --stat HEAD upstream/main
Write-Host ""

$response = Read-Host "是否继续测试合并? (y/N)"
if ($response -notmatch '^[Yy]$') {
    Write-Host "⚠️  已取消" -ForegroundColor Yellow
    exit 0
}

Write-Host ""
Write-Host "🔧 步骤 4: 创建测试分支" -ForegroundColor Cyan
Write-Host "----------------------------------------"

$testBranch = "test-sync-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
git checkout -b $testBranch
Write-Host "✅ 创建测试分支: $testBranch" -ForegroundColor Green
Write-Host ""

Write-Host "🔀 步骤 5: 尝试合并上游" -ForegroundColor Cyan
Write-Host "----------------------------------------"

try {
    git merge upstream/main --no-edit --allow-unrelated-histories 2>&1 | Out-Null
    $mergeSuccess = $true
} catch {
    $mergeSuccess = $false
}

if ($mergeSuccess) {
    Write-Host ""
    Write-Host "✅ 合并成功！没有冲突" -ForegroundColor Green
    Write-Host ""
    
    Write-Host "📊 合并后的状态:" -ForegroundColor Cyan
    git log --oneline -5
    Write-Host ""
    
    Write-Host "🎉 测试成功！" -ForegroundColor Green
    Write-Host ""
    Write-Host "下一步选择:"
    Write-Host ""
    Write-Host "1️⃣  推送测试分支到 GitHub (查看 PR 预览):"
    Write-Host "   git push origin $testBranch"
    Write-Host ""
    Write-Host "2️⃣  合并到 main 分支:"
    Write-Host "   git checkout main"
    Write-Host "   git merge $testBranch"
    Write-Host "   git push origin main"
    Write-Host ""
    Write-Host "3️⃣  删除测试分支并返回:"
    Write-Host "   git checkout $currentBranch"
    Write-Host "   git branch -D $testBranch"
    Write-Host ""
    
    $response = Read-Host "是否自动合并到 main 并推送? (y/N)"
    if ($response -match '^[Yy]$') {
        Write-Host ""
        Write-Host "🚀 合并到 main 分支..." -ForegroundColor Cyan
        git checkout main
        git merge $testBranch --no-edit
        
        Write-Host ""
        Write-Host "📤 推送到 GitHub..." -ForegroundColor Cyan
        git push origin main
        
        Write-Host ""
        Write-Host "✅ 完成！" -ForegroundColor Green
        
        # 清理测试分支
        git branch -D $testBranch
    } else {
        Write-Host ""
        Write-Host "⚠️  请手动处理测试分支" -ForegroundColor Yellow
    }
    
} else {
    Write-Host ""
    Write-Host "❌ 合并冲突！" -ForegroundColor Red
    Write-Host ""
    Write-Host "冲突文件:" -ForegroundColor Yellow
    git diff --name-only --diff-filter=U
    Write-Host ""
    
    Write-Host "解决冲突的步骤:"
    Write-Host ""
    Write-Host "1️⃣  查看冲突文件:"
    Write-Host "   git status"
    Write-Host ""
    Write-Host "2️⃣  手动编辑冲突文件，解决冲突标记"
    Write-Host ""
    Write-Host "3️⃣  标记为已解决:"
    Write-Host "   git add <文件名>"
    Write-Host ""
    Write-Host "4️⃣  完成合并:"
    Write-Host "   git commit"
    Write-Host ""
    Write-Host "5️⃣  推送:"
    Write-Host "   git push origin $testBranch"
    Write-Host ""
    Write-Host "或者使用快速解决脚本（接受所有上游版本）:"
    Write-Host "   .\scripts\resolve-first-sync-conflicts.ps1"
    Write-Host ""
    
    # 中止合并
    git merge --abort 2>$null
    
    # 返回原分支
    git checkout $currentBranch
    
    # 删除测试分支
    git branch -D $testBranch
    
    Write-Host "⚠️  已中止合并并清理测试分支" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "建议: 先手动解决冲突，或查看 docs/MERGE-SAFETY-GUIDE.md"
    exit 1
}

