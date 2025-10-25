#!/bin/bash
# 本地测试上游同步流程
# 在推送到 GitHub Actions 之前，先在本地验证同步流程

set -e

echo "========================================="
echo "本地测试上游同步流程"
echo "========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查是否在 Git 仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}❌ 当前目录不是 Git 仓库${NC}"
    exit 1
fi

# 检查是否配置了 upstream
if ! git remote get-url upstream > /dev/null 2>&1; then
    echo -e "${RED}❌ 未配置 upstream 远程仓库${NC}"
    echo ""
    echo "请先配置 upstream:"
    echo "  git remote add upstream https://github.com/zhaochengcube/augment-token-mng.git"
    exit 1
fi

echo -e "${GREEN}✅ Git 仓库检查通过${NC}"
echo ""

# 保存当前分支
CURRENT_BRANCH=$(git branch --show-current)
echo -e "${BLUE}📍 当前分支: $CURRENT_BRANCH${NC}"
echo ""

# 检查工作区是否干净
if ! git diff-index --quiet HEAD --; then
    echo -e "${YELLOW}⚠️  工作区有未提交的更改${NC}"
    echo ""
    git status --short
    echo ""
    read -p "是否继续? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ 已取消${NC}"
        exit 0
    fi
fi

echo -e "${BLUE}🔄 步骤 1: 获取上游更新${NC}"
echo "----------------------------------------"
git fetch upstream
echo -e "${GREEN}✅ 完成${NC}"
echo ""

echo -e "${BLUE}📊 步骤 2: 检查上游更新${NC}"
echo "----------------------------------------"

# 检查是否有新提交
UPSTREAM_COMMITS=$(git log HEAD..upstream/main --oneline)

if [ -z "$UPSTREAM_COMMITS" ]; then
    echo -e "${GREEN}✅ 没有新的上游提交${NC}"
    echo ""
    echo "您的代码已经是最新的！"
    exit 0
fi

echo -e "${YELLOW}发现新的上游提交:${NC}"
echo ""
echo "$UPSTREAM_COMMITS"
echo ""

# 统计更改
COMMITS_COUNT=$(echo "$UPSTREAM_COMMITS" | wc -l)
echo -e "${BLUE}📈 统计: $COMMITS_COUNT 个新提交${NC}"
echo ""

echo -e "${BLUE}📋 步骤 3: 查看文件变更${NC}"
echo "----------------------------------------"
git diff --stat HEAD upstream/main
echo ""

read -p "是否继续测试合并? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}⚠️  已取消${NC}"
    exit 0
fi

echo ""
echo -e "${BLUE}🔧 步骤 4: 创建测试分支${NC}"
echo "----------------------------------------"

TEST_BRANCH="test-sync-$(date +%Y%m%d-%H%M%S)"
git checkout -b "$TEST_BRANCH"
echo -e "${GREEN}✅ 创建测试分支: $TEST_BRANCH${NC}"
echo ""

echo -e "${BLUE}🔀 步骤 5: 尝试合并上游${NC}"
echo "----------------------------------------"

if git merge upstream/main --no-edit --allow-unrelated-histories; then
    echo ""
    echo -e "${GREEN}✅ 合并成功！没有冲突${NC}"
    echo ""
    
    echo -e "${BLUE}📊 合并后的状态:${NC}"
    git log --oneline -5
    echo ""
    
    echo -e "${GREEN}🎉 测试成功！${NC}"
    echo ""
    echo "下一步选择:"
    echo ""
    echo "1️⃣  推送测试分支到 GitHub (查看 PR 预览):"
    echo "   git push origin $TEST_BRANCH"
    echo ""
    echo "2️⃣  合并到 main 分支:"
    echo "   git checkout main"
    echo "   git merge $TEST_BRANCH"
    echo "   git push origin main"
    echo ""
    echo "3️⃣  删除测试分支并返回:"
    echo "   git checkout $CURRENT_BRANCH"
    echo "   git branch -D $TEST_BRANCH"
    echo ""
    
    read -p "是否自动合并到 main 并推送? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo -e "${BLUE}🚀 合并到 main 分支...${NC}"
        git checkout main
        git merge "$TEST_BRANCH" --no-edit
        
        echo ""
        echo -e "${BLUE}📤 推送到 GitHub...${NC}"
        git push origin main
        
        echo ""
        echo -e "${GREEN}✅ 完成！${NC}"
        
        # 清理测试分支
        git branch -D "$TEST_BRANCH"
    else
        echo ""
        echo -e "${YELLOW}⚠️  请手动处理测试分支${NC}"
    fi
    
else
    echo ""
    echo -e "${RED}❌ 合并冲突！${NC}"
    echo ""
    echo -e "${YELLOW}冲突文件:${NC}"
    git diff --name-only --diff-filter=U
    echo ""
    
    echo "解决冲突的步骤:"
    echo ""
    echo "1️⃣  查看冲突文件:"
    echo "   git status"
    echo ""
    echo "2️⃣  手动编辑冲突文件，解决冲突标记:"
    echo "   <<<<<<< HEAD"
    echo "   您的代码"
    echo "   ======="
    echo "   上游的代码"
    echo "   >>>>>>> upstream/main"
    echo ""
    echo "3️⃣  标记为已解决:"
    echo "   git add <文件名>"
    echo ""
    echo "4️⃣  完成合并:"
    echo "   git commit"
    echo ""
    echo "5️⃣  推送:"
    echo "   git push origin $TEST_BRANCH"
    echo ""
    echo "或者使用快速解决脚本（接受所有上游版本）:"
    echo "   ./scripts/resolve-first-sync-conflicts.sh"
    echo ""
    
    # 中止合并
    git merge --abort
    
    # 返回原分支
    git checkout "$CURRENT_BRANCH"
    
    # 删除测试分支
    git branch -D "$TEST_BRANCH"
    
    echo -e "${YELLOW}⚠️  已中止合并并清理测试分支${NC}"
    echo ""
    echo "建议: 先手动解决冲突，或查看 docs/MERGE-SAFETY-GUIDE.md"
    exit 1
fi

