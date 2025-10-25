#!/bin/bash
# 解决首次同步的合并冲突
# 策略: 接受所有上游版本（因为我们主要添加的是 CI/CD 文件）

set -e

echo "========================================="
echo "解决首次同步冲突"
echo "========================================="
echo ""

# 检查是否在合并状态
if ! git status | grep -q "You have unmerged paths"; then
    echo "❌ 当前不在合并冲突状态"
    echo "请先运行: git merge upstream/main --allow-unrelated-histories"
    exit 1
fi

echo "📋 冲突文件列表:"
git diff --name-only --diff-filter=U
echo ""

echo "🔧 解决策略: 接受上游版本（保留我们的 CI/CD 文件）"
echo ""

# 接受上游版本的文件
echo "✅ 接受上游版本..."

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

echo "✅ 所有冲突文件已解决"
echo ""

# 标记为已解决
echo "📝 标记文件为已解决..."
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

echo "✅ 文件已标记为已解决"
echo ""

# 检查是否还有未解决的冲突
if git diff --name-only --diff-filter=U | grep -q .; then
    echo "⚠️ 还有未解决的冲突:"
    git diff --name-only --diff-filter=U
    exit 1
fi

echo "🎉 所有冲突已解决！"
echo ""
echo "📋 下一步:"
echo "1. 检查合并结果: git status"
echo "2. 提交合并: git commit -m 'chore: merge upstream/main'"
echo "3. 推送代码: git push origin main"
echo ""

