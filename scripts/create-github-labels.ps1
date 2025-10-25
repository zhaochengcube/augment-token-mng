# 创建 GitHub 仓库标签 (PowerShell 版本)
# 用于 CI/CD 自动化工作流

$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Blue
Write-Host "创建 GitHub 仓库标签" -ForegroundColor Blue
Write-Host "=========================================" -ForegroundColor Blue
Write-Host ""

# 检查是否安装了 gh CLI
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host "❌ GitHub CLI (gh) 未安装" -ForegroundColor Red
    Write-Host ""
    Write-Host "请先安装 GitHub CLI:"
    Write-Host "  winget install GitHub.cli"
    Write-Host ""
    Write-Host "或访问: https://cli.github.com/"
    exit 1
}

# 检查是否已登录
try {
    gh auth status 2>$null | Out-Null
} catch {
    Write-Host "❌ 未登录 GitHub CLI" -ForegroundColor Red
    Write-Host ""
    Write-Host "请先登录:"
    Write-Host "  gh auth login"
    exit 1
}

Write-Host "✅ GitHub CLI 已就绪" -ForegroundColor Green
Write-Host ""

# 定义标签
$labels = @{
    "upstream-sync" = @{
        color = "0366d6"
        description = "自动同步上游仓库的 PR"
    }
    "automated" = @{
        color = "ededed"
        description = "自动化创建的 PR 或 Issue"
    }
    "conflict" = @{
        color = "d93f0b"
        description = "存在合并冲突需要手动处理"
    }
    "release" = @{
        color = "00ff00"
        description = "发布相关"
    }
    "ci-cd" = @{
        color = "1d76db"
        description = "CI/CD 相关"
    }
}

Write-Host "📋 将创建以下标签:" -ForegroundColor Cyan
Write-Host ""
foreach ($label in $labels.Keys) {
    $color = $labels[$label].color
    $description = $labels[$label].description
    Write-Host "  🏷️  $label" -ForegroundColor Yellow
    Write-Host "      颜色: #$color"
    Write-Host "      描述: $description"
    Write-Host ""
}

$response = Read-Host "是否继续? (y/N)"
if ($response -notmatch '^[Yy]$') {
    Write-Host "❌ 已取消" -ForegroundColor Red
    exit 0
}

Write-Host ""
Write-Host "🔧 开始创建标签..." -ForegroundColor Cyan
Write-Host ""

# 创建标签
foreach ($label in $labels.Keys) {
    $color = $labels[$label].color
    $description = $labels[$label].description
    
    Write-Host "  创建标签 '$label'... " -NoNewline
    
    try {
        gh label create $label --color $color --description $description 2>$null
        Write-Host "✅" -ForegroundColor Green
    } catch {
        # 如果标签已存在，尝试更新
        try {
            gh label edit $label --color $color --description $description 2>$null
            Write-Host "✅ (已更新)" -ForegroundColor Green
        } catch {
            Write-Host "⚠️ (失败)" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "🎉 标签创建完成！" -ForegroundColor Green
Write-Host ""
Write-Host "📋 查看所有标签:" -ForegroundColor Cyan
Write-Host "  gh label list"
Write-Host ""

try {
    $repoInfo = gh repo view --json nameWithOwner | ConvertFrom-Json
    $repoName = $repoInfo.nameWithOwner
    Write-Host "或访问:"
    Write-Host "  https://github.com/$repoName/labels"
} catch {
    Write-Host "提示: 可以在 GitHub 仓库的 Issues 标签页查看所有标签"
}
Write-Host ""

