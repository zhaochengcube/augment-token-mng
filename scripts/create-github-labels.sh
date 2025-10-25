#!/bin/bash
# 创建 GitHub 仓库标签
# 用于 CI/CD 自动化工作流

set -e

echo "========================================="
echo "创建 GitHub 仓库标签"
echo "========================================="
echo ""

# 检查是否安装了 gh CLI
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) 未安装"
    echo ""
    echo "请先安装 GitHub CLI:"
    echo "  Windows: winget install GitHub.cli"
    echo "  macOS:   brew install gh"
    echo "  Linux:   https://github.com/cli/cli#installation"
    exit 1
fi

# 检查是否已登录
if ! gh auth status &> /dev/null; then
    echo "❌ 未登录 GitHub CLI"
    echo ""
    echo "请先登录:"
    echo "  gh auth login"
    exit 1
fi

echo "✅ GitHub CLI 已就绪"
echo ""

# 定义标签
declare -A labels=(
    ["upstream-sync"]="0366d6|自动同步上游仓库的 PR"
    ["automated"]="ededed|自动化创建的 PR 或 Issue"
    ["conflict"]="d93f0b|存在合并冲突需要手动处理"
    ["release"]="00ff00|发布相关"
    ["ci-cd"]="1d76db|CI/CD 相关"
)

echo "📋 将创建以下标签:"
echo ""
for label in "${!labels[@]}"; do
    IFS='|' read -r color description <<< "${labels[$label]}"
    echo "  🏷️  $label"
    echo "      颜色: #$color"
    echo "      描述: $description"
    echo ""
done

read -p "是否继续? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ 已取消"
    exit 0
fi

echo ""
echo "🔧 开始创建标签..."
echo ""

# 创建标签
for label in "${!labels[@]}"; do
    IFS='|' read -r color description <<< "${labels[$label]}"
    
    echo -n "  创建标签 '$label'... "
    
    if gh label create "$label" --color "$color" --description "$description" 2>/dev/null; then
        echo "✅"
    else
        # 如果标签已存在，尝试更新
        if gh label edit "$label" --color "$color" --description "$description" 2>/dev/null; then
            echo "✅ (已更新)"
        else
            echo "⚠️ (失败)"
        fi
    fi
done

echo ""
echo "🎉 标签创建完成！"
echo ""
echo "📋 查看所有标签:"
echo "  gh label list"
echo ""
echo "或访问:"
echo "  https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/labels"
echo ""

