# CI/CD 自动化设置脚本 (PowerShell 版本)
# 用于配置上游同步和自动发布工作流

# 设置错误处理
$ErrorActionPreference = "Stop"

# 颜色函数
function Write-Header {
    param([string]$Message)
    Write-Host "`n========================================" -ForegroundColor Blue
    Write-Host $Message -ForegroundColor Blue
    Write-Host "========================================`n" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor Cyan
}

# 检查是否在 Git 仓库中
function Test-GitRepo {
    Write-Header "检查 Git 仓库"
    
    try {
        git rev-parse --git-dir | Out-Null
        Write-Success "Git 仓库检查通过"
        return $true
    }
    catch {
        Write-Error "当前目录不是 Git 仓库"
        Write-Info "请在项目根目录运行此脚本"
        return $false
    }
}

# 检查必需的工具
function Test-Dependencies {
    Write-Header "检查依赖工具"
    
    $missingDeps = @()
    
    # 检查 Git
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        $missingDeps += "git"
    }
    
    # 检查 Node.js
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        $missingDeps += "node"
    }
    
    # 检查 GitHub CLI (可选)
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        Write-Warning "GitHub CLI (gh) 未安装"
        Write-Info "某些功能需要 GitHub CLI，建议安装: https://cli.github.com/"
    }
    else {
        Write-Success "GitHub CLI 已安装"
    }
    
    if ($missingDeps.Count -gt 0) {
        Write-Error "缺少必需的工具: $($missingDeps -join ', ')"
        return $false
    }
    
    Write-Success "所有必需工具已安装"
    return $true
}

# 配置上游仓库
function Set-Upstream {
    Write-Header "配置上游仓库"
    
    $upstreamUrl = "https://github.com/zhaochengcube/augment-token-mng.git"
    
    # 检查是否已配置上游
    $remotes = git remote
    if ($remotes -contains "upstream") {
        Write-Warning "上游仓库已配置"
        $currentUrl = git remote get-url upstream
        Write-Host "当前上游 URL: $currentUrl"
        
        $response = Read-Host "是否更新上游 URL? (y/N)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            git remote set-url upstream $upstreamUrl
            Write-Success "上游 URL 已更新"
        }
    }
    else {
        git remote add upstream $upstreamUrl
        Write-Success "上游仓库已添加: $upstreamUrl"
    }
    
    # 获取上游更新
    Write-Info "获取上游更新..."
    git fetch upstream
    Write-Success "上游更新已获取"
}

# 检查版本号一致性
function Test-VersionConsistency {
    Write-Header "检查版本号一致性"
    
    # 读取各文件的版本号
    $pkgVersion = node -p "require('./package.json').version"
    $cargoVersion = (Select-String -Path "src-tauri/Cargo.toml" -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value
    $tauriVersion = node -p "require('./src-tauri/tauri.conf.json').version"
    
    Write-Host "package.json:        $pkgVersion"
    Write-Host "Cargo.toml:          $cargoVersion"
    Write-Host "tauri.conf.json:     $tauriVersion"
    
    if ($pkgVersion -eq $cargoVersion -and $pkgVersion -eq $tauriVersion) {
        Write-Success "版本号一致"
        return $pkgVersion
    }
    else {
        Write-Warning "版本号不一致"
        $response = Read-Host "是否统一版本号为 $pkgVersion? (y/N)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            # 更新 Cargo.toml
            (Get-Content "src-tauri/Cargo.toml") -replace '^version = ".+"', "version = `"$pkgVersion`"" | Set-Content "src-tauri/Cargo.toml"
            
            # 更新 tauri.conf.json
            $tauriConfig = Get-Content "src-tauri/tauri.conf.json" | ConvertFrom-Json
            $tauriConfig.version = $pkgVersion
            $tauriConfig | ConvertTo-Json -Depth 10 | Set-Content "src-tauri/tauri.conf.json"
            
            Write-Success "版本号已统一为 $pkgVersion"
        }
        return $pkgVersion
    }
}

# 选择版本策略
function Select-VersionStrategy {
    Write-Header "选择版本管理策略"
    
    Write-Host "请选择版本管理策略:"
    Write-Host "1) 版本号后缀 (推荐) - 例如: 1.2.0-fork.1"
    Write-Host "2) 独立版本号 - 例如: 2.0.0 (Fork 从 2.x 开始)"
    Write-Host "3) 标准版本号 - 例如: 1.2.0 (与上游相同格式)"
    Write-Host ""
    
    $choice = Read-Host "请选择 (1-3)"
    
    switch ($choice) {
        "1" {
            Write-Info "已选择: 版本号后缀策略"
            "VERSION_STRATEGY=fork-suffix" | Out-File -FilePath ".ci-cd-config" -Encoding UTF8
            
            # 询问是否立即应用
            $currentVersion = node -p "require('./package.json').version"
            if ($currentVersion -notmatch "-fork\.") {
                $response = Read-Host "是否将当前版本 $currentVersion 更新为 $currentVersion-fork.1? (y/N)"
                if ($response -eq 'y' -or $response -eq 'Y') {
                    Set-ForkSuffixVersion $currentVersion
                }
            }
        }
        "2" {
            Write-Info "已选择: 独立版本号策略"
            "VERSION_STRATEGY=independent" | Out-File -FilePath ".ci-cd-config" -Encoding UTF8
            Write-Warning "请确保在 package.json 中设置了独立的版本号 (如 2.0.0)"
        }
        "3" {
            Write-Info "已选择: 标准版本号策略"
            "VERSION_STRATEGY=standard" | Out-File -FilePath ".ci-cd-config" -Encoding UTF8
            Write-Warning "将使用与上游相同的版本号格式"
        }
        default {
            Write-Error "无效选择"
            exit 1
        }
    }
    
    Write-Success "版本策略已保存到 .ci-cd-config"
}

# 应用 fork 后缀版本
function Set-ForkSuffixVersion {
    param([string]$BaseVersion)
    
    $newVersion = "$BaseVersion-fork.1"
    
    # 更新 package.json
    $pkg = Get-Content "package.json" | ConvertFrom-Json
    $pkg.version = $newVersion
    $pkg | ConvertTo-Json -Depth 10 | Set-Content "package.json"
    
    # 更新 Cargo.toml
    (Get-Content "src-tauri/Cargo.toml") -replace '^version = ".+"', "version = `"$newVersion`"" | Set-Content "src-tauri/Cargo.toml"
    
    # 更新 tauri.conf.json
    $tauriConfig = Get-Content "src-tauri/tauri.conf.json" | ConvertFrom-Json
    $tauriConfig.version = $newVersion
    $tauriConfig | ConvertTo-Json -Depth 10 | Set-Content "src-tauri/tauri.conf.json"
    
    Write-Success "版本已更新为 $newVersion"
    
    $response = Read-Host "是否提交此更改? (y/N)"
    if ($response -eq 'y' -or $response -eq 'Y') {
        git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
        git commit -m "chore: adopt fork version strategy ($newVersion)"
        Write-Success "更改已提交"
    }
}

# 检查 GitHub Secrets
function Test-GitHubSecrets {
    Write-Header "检查 GitHub Secrets"
    
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        Write-Warning "GitHub CLI 未安装，跳过 Secrets 检查"
        Write-Info "请手动在 GitHub 仓库设置中配置 Secrets"
        return
    }
    
    # 检查是否已登录
    try {
        gh auth status | Out-Null
    }
    catch {
        Write-Warning "未登录 GitHub CLI"
        Write-Info "运行 'gh auth login' 登录后再检查 Secrets"
        return
    }
    
    Write-Info "检查必需的 Secrets..."
    
    # GITHUB_TOKEN 是自动提供的
    Write-Success "GITHUB_TOKEN (自动提供)"
    
    # 检查可选的 Secrets
    $optionalSecrets = @("TAP_DISPATCH_TOKEN", "TELEGRAM_BOT_TOKEN", "TELEGRAM_CHAT_ID")
    
    foreach ($secret in $optionalSecrets) {
        Write-Info "$secret (可选) - 请在 GitHub 仓库设置中手动检查"
    }
    
    Write-Host ""
    Write-Info "配置 Secrets 的步骤:"
    $repoInfo = gh repo view --json nameWithOwner | ConvertFrom-Json
    Write-Host "1. 访问: https://github.com/$($repoInfo.nameWithOwner)/settings/secrets/actions"
    Write-Host "2. 点击 'New repository secret'"
    Write-Host "3. 添加所需的 Secrets"
}

# 测试工作流
function Test-Workflow {
    Write-Header "测试工作流配置"
    
    # 检查工作流文件是否存在
    $workflows = @(
        ".github/workflows/sync-upstream.yml",
        ".github/workflows/build.yml",
        ".github/workflows/version-bump-and-tag.yml"
    )
    
    foreach ($workflow in $workflows) {
        if (Test-Path $workflow) {
            Write-Success "$(Split-Path $workflow -Leaf) 存在"
        }
        else {
            Write-Error "$(Split-Path $workflow -Leaf) 不存在"
        }
    }
    
    Write-Host ""
    Write-Info "工作流文件检查完成"
    Write-Info "可以通过 GitHub Actions 界面手动触发测试"
}

# 生成设置报告
function New-Report {
    Write-Header "设置报告"
    
    $reportFile = "ci-cd-setup-report.txt"
    
    $report = @"
CI/CD 自动化设置报告
生成时间: $(Get-Date)

=== Git 配置 ===
$(git remote -v)

=== 版本信息 ===
package.json:     $(node -p "require('./package.json').version")
Cargo.toml:       $((Select-String -Path "src-tauri/Cargo.toml" -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value)
tauri.conf.json:  $(node -p "require('./src-tauri/tauri.conf.json').version")

=== 版本策略 ===
$(if (Test-Path .ci-cd-config) { Get-Content .ci-cd-config } else { "未配置" })

=== 工作流文件 ===
$(Get-ChildItem .github/workflows/ | Format-Table -AutoSize | Out-String)

=== 下一步 ===
1. 配置 GitHub Secrets (如需要)
2. 测试手动触发工作流
3. 等待定时任务自动运行
4. 查看文档: docs/CI-CD-GUIDE.md
"@
    
    $report | Out-File -FilePath $reportFile -Encoding UTF8
    
    Write-Success "设置报告已生成: $reportFile"
    Write-Host $report
}

# 主函数
function Main {
    Write-Header "CI/CD 自动化设置向导"
    
    if (-not (Test-GitRepo)) { exit 1 }
    if (-not (Test-Dependencies)) { exit 1 }
    
    Set-Upstream
    Test-VersionConsistency
    Select-VersionStrategy
    Test-GitHubSecrets
    Test-Workflow
    New-Report
    
    Write-Header "设置完成"
    Write-Success "CI/CD 自动化已配置完成！"
    Write-Host ""
    Write-Info "下一步:"
    Write-Host "1. 查看设置报告: Get-Content ci-cd-setup-report.txt"
    Write-Host "2. 阅读文档: docs/CI-CD-GUIDE.md"
    Write-Host "3. 配置 GitHub Secrets (如需要)"
    Write-Host "4. 测试工作流: GitHub Actions → Sync Upstream and Auto Release → Run workflow"
    Write-Host ""
    Write-Info "定时任务将在每天 UTC 00:00 自动运行"
}

# 运行主函数
Main

