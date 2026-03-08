# Install & Build

## Package managers

### macOS - Homebrew

```bash
brew tap zhaochengcube/atm
brew install --cask atm
brew update && brew upgrade --cask atm
```

### Windows - Scoop

```powershell
scoop bucket add atm https://github.com/zhaochengcube/scoop-atm
scoop install atm
scoop update atm
```

## Release builds

Download installers from [Releases](https://github.com/zhaochengcube/augment-token-mng/releases).

## Build from source

### Requirements

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (18+)
- pnpm: `npm install -g pnpm`

### Dev

```bash
pnpm install
pnpm tauri dev
```

### Production build

```bash
pnpm install
pnpm tauri build
```

Artifacts under `src-tauri/target/release/` (or `debug/`).
