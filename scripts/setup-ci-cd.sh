#!/bin/bash

# CI/CD 自动化设置脚本
# 用于配置上游同步和自动发布工作流

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# 检查是否在 Git 仓库中
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "当前目录不是 Git 仓库"
        print_info "请在项目根目录运行此脚本"
        exit 1
    fi
    print_success "Git 仓库检查通过"
}

# 检查必需的工具
check_dependencies() {
    print_header "检查依赖工具"
    
    local missing_deps=()
    
    if ! command -v git &> /dev/null; then
        missing_deps+=("git")
    fi
    
    if ! command -v node &> /dev/null; then
        missing_deps+=("node")
    fi
    
    if ! command -v gh &> /dev/null; then
        print_warning "GitHub CLI (gh) 未安装"
        print_info "某些功能需要 GitHub CLI，建议安装: https://cli.github.com/"
    else
        print_success "GitHub CLI 已安装"
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "缺少必需的工具: ${missing_deps[*]}"
        exit 1
    fi
    
    print_success "所有必需工具已安装"
}

# 配置上游仓库
setup_upstream() {
    print_header "配置上游仓库"
    
    local upstream_url="https://github.com/zhaochengcube/augment-token-mng.git"
    
    # 检查是否已配置上游
    if git remote | grep -q "^upstream$"; then
        print_warning "上游仓库已配置"
        local current_url=$(git remote get-url upstream)
        echo "当前上游 URL: $current_url"
        
        read -p "是否更新上游 URL? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git remote set-url upstream "$upstream_url"
            print_success "上游 URL 已更新"
        fi
    else
        git remote add upstream "$upstream_url"
        print_success "上游仓库已添加: $upstream_url"
    fi
    
    # 获取上游更新
    print_info "获取上游更新..."
    git fetch upstream
    print_success "上游更新已获取"
}

# 检查版本号一致性
check_version_consistency() {
    print_header "检查版本号一致性"
    
    local pkg_version=$(node -p "require('./package.json').version")
    local cargo_version=$(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    local tauri_version=$(node -p "require('./src-tauri/tauri.conf.json').version")
    
    echo "package.json:        $pkg_version"
    echo "Cargo.toml:          $cargo_version"
    echo "tauri.conf.json:     $tauri_version"
    
    if [ "$pkg_version" = "$cargo_version" ] && [ "$pkg_version" = "$tauri_version" ]; then
        print_success "版本号一致"
    else
        print_warning "版本号不一致"
        read -p "是否统一版本号为 $pkg_version? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            # 更新 Cargo.toml
            sed -i.bak "s/^version = \".*\"/version = \"$pkg_version\"/" src-tauri/Cargo.toml
            rm -f src-tauri/Cargo.toml.bak
            
            # 更新 tauri.conf.json
            node -e "
                const fs = require('fs');
                const config = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
                config.version = '$pkg_version';
                fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(config, null, 2) + '\n');
            "
            
            print_success "版本号已统一为 $pkg_version"
        fi
    fi
}

# 选择版本策略
choose_version_strategy() {
    print_header "选择版本管理策略"
    
    echo "请选择版本管理策略:"
    echo "1) 版本号后缀 (推荐) - 例如: 1.2.0-fork.1"
    echo "2) 独立版本号 - 例如: 2.0.0 (Fork 从 2.x 开始)"
    echo "3) 标准版本号 - 例如: 1.2.0 (与上游相同格式)"
    echo ""
    read -p "请选择 (1-3): " -n 1 -r
    echo
    
    case $REPLY in
        1)
            print_info "已选择: 版本号后缀策略"
            echo "VERSION_STRATEGY=fork-suffix" > .ci-cd-config
            
            # 询问是否立即应用
            local current_version=$(node -p "require('./package.json').version")
            if [[ ! $current_version =~ -fork\. ]]; then
                read -p "是否将当前版本 $current_version 更新为 $current_version-fork.1? (y/N): " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    apply_fork_suffix_version "$current_version"
                fi
            fi
            ;;
        2)
            print_info "已选择: 独立版本号策略"
            echo "VERSION_STRATEGY=independent" > .ci-cd-config
            print_warning "请确保在 package.json 中设置了独立的版本号 (如 2.0.0)"
            ;;
        3)
            print_info "已选择: 标准版本号策略"
            echo "VERSION_STRATEGY=standard" > .ci-cd-config
            print_warning "将使用与上游相同的版本号格式"
            ;;
        *)
            print_error "无效选择"
            exit 1
            ;;
    esac
    
    print_success "版本策略已保存到 .ci-cd-config"
}

# 应用 fork 后缀版本
apply_fork_suffix_version() {
    local base_version=$1
    local new_version="$base_version-fork.1"
    
    # 更新 package.json
    node -e "
        const fs = require('fs');
        const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
        pkg.version = '$new_version';
        fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    
    # 更新 Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" src-tauri/Cargo.toml
    rm -f src-tauri/Cargo.toml.bak
    
    # 更新 tauri.conf.json
    node -e "
        const fs = require('fs');
        const config = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
        config.version = '$new_version';
        fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(config, null, 2) + '\n');
    "
    
    print_success "版本已更新为 $new_version"
    
    read -p "是否提交此更改? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
        git commit -m "chore: adopt fork version strategy ($new_version)"
        print_success "更改已提交"
    fi
}

# 检查 GitHub Secrets
check_github_secrets() {
    print_header "检查 GitHub Secrets"
    
    if ! command -v gh &> /dev/null; then
        print_warning "GitHub CLI 未安装，跳过 Secrets 检查"
        print_info "请手动在 GitHub 仓库设置中配置 Secrets"
        return
    fi
    
    # 检查是否已登录
    if ! gh auth status &> /dev/null; then
        print_warning "未登录 GitHub CLI"
        print_info "运行 'gh auth login' 登录后再检查 Secrets"
        return
    fi
    
    print_info "检查必需的 Secrets..."
    
    # GITHUB_TOKEN 是自动提供的，不需要检查
    print_success "GITHUB_TOKEN (自动提供)"
    
    # 检查可选的 Secrets
    local optional_secrets=("TAP_DISPATCH_TOKEN" "TELEGRAM_BOT_TOKEN" "TELEGRAM_CHAT_ID")
    
    for secret in "${optional_secrets[@]}"; do
        # gh secret list 只能列出名称，无法检查是否存在
        print_info "$secret (可选) - 请在 GitHub 仓库设置中手动检查"
    done
    
    echo ""
    print_info "配置 Secrets 的步骤:"
    echo "1. 访问: https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/settings/secrets/actions"
    echo "2. 点击 'New repository secret'"
    echo "3. 添加所需的 Secrets"
}

# 测试工作流
test_workflow() {
    print_header "测试工作流配置"
    
    # 检查工作流文件是否存在
    local workflows=(
        ".github/workflows/sync-upstream.yml"
        ".github/workflows/build.yml"
        ".github/workflows/version-bump-and-tag.yml"
    )
    
    for workflow in "${workflows[@]}"; do
        if [ -f "$workflow" ]; then
            print_success "$(basename $workflow) 存在"
        else
            print_error "$(basename $workflow) 不存在"
        fi
    done
    
    echo ""
    print_info "工作流文件检查完成"
    print_info "可以通过 GitHub Actions 界面手动触发测试"
}

# 生成设置报告
generate_report() {
    print_header "设置报告"
    
    local report_file="ci-cd-setup-report.txt"
    
    {
        echo "CI/CD 自动化设置报告"
        echo "生成时间: $(date)"
        echo ""
        echo "=== Git 配置 ==="
        git remote -v
        echo ""
        echo "=== 版本信息 ==="
        echo "package.json:     $(node -p "require('./package.json').version")"
        echo "Cargo.toml:       $(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"
        echo "tauri.conf.json:  $(node -p "require('./src-tauri/tauri.conf.json').version")"
        echo ""
        echo "=== 版本策略 ==="
        if [ -f .ci-cd-config ]; then
            cat .ci-cd-config
        else
            echo "未配置"
        fi
        echo ""
        echo "=== 工作流文件 ==="
        ls -la .github/workflows/
        echo ""
        echo "=== 下一步 ==="
        echo "1. 配置 GitHub Secrets (如需要)"
        echo "2. 测试手动触发工作流"
        echo "3. 等待定时任务自动运行"
        echo "4. 查看文档: docs/CI-CD-GUIDE.md"
    } > "$report_file"
    
    print_success "设置报告已生成: $report_file"
    cat "$report_file"
}

# 主函数
main() {
    print_header "CI/CD 自动化设置向导"
    
    check_git_repo
    check_dependencies
    setup_upstream
    check_version_consistency
    choose_version_strategy
    check_github_secrets
    test_workflow
    generate_report
    
    print_header "设置完成"
    print_success "CI/CD 自动化已配置完成！"
    echo ""
    print_info "下一步:"
    echo "1. 查看设置报告: cat ci-cd-setup-report.txt"
    echo "2. 阅读文档: docs/CI-CD-GUIDE.md"
    echo "3. 配置 GitHub Secrets (如需要)"
    echo "4. 测试工作流: GitHub Actions → Sync Upstream and Auto Release → Run workflow"
    echo ""
    print_info "定时任务将在每天 UTC 00:00 自动运行"
}

# 运行主函数
main

