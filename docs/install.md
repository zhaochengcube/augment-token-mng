# 安装与构建

## 包管理器安装（推荐）

### macOS - Homebrew

```bash
brew tap zhaochengcube/atm
brew install --cask atm
# 更新
brew update && brew upgrade --cask atm
```

### Windows - Scoop

```powershell
scoop bucket add atm https://github.com/zhaochengcube/scoop-atm
scoop install atm
scoop update atm
```

## Release 下载

在 [Releases](https://github.com/zhaochengcube/augment-token-mng/releases) 中按平台下载安装包。

## 从源码构建

### 环境要求

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)（建议 18+）
- pnpm：`npm install -g pnpm`

### 开发模式

```bash
pnpm install
pnpm tauri dev
```

### 生产构建

```bash
pnpm install
pnpm tauri build
```

产出在 `src-tauri/target/release/`（或 `debug/`）。
