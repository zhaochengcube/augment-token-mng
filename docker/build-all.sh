#!/bin/bash
# Multi-platform build script for Tauri application in Docker

set -e

echo "🚀 Starting multi-platform Tauri build..."

# Create output directory
mkdir -p /app/dist/artifacts

# Build for Linux x86_64 (default)
echo "📦 Building for Linux x86_64..."
cd /app
export PKG_CONFIG_ALLOW_CROSS=1
cargo tauri build --target x86_64-unknown-linux-gnu

# Copy Linux artifacts
if [ -d "src-tauri/target/x86_64-unknown-linux-gnu/release/bundle" ]; then
    cp -r src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/* /app/dist/artifacts/ 2>/dev/null || true
    cp src-tauri/target/x86_64-unknown-linux-gnu/release/atm /app/dist/artifacts/atm-linux-x86_64 2>/dev/null || true
fi

# Build for Linux ARM64 (if cross-compilation tools are available)
if [ "$BUILD_ARM64" = "true" ]; then
    echo "📦 Building for Linux ARM64..."
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
    export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    
    cargo tauri build --target aarch64-unknown-linux-gnu || echo "⚠️  ARM64 build failed, skipping..."
    
    # Copy ARM64 artifacts
    if [ -d "src-tauri/target/aarch64-unknown-linux-gnu/release/bundle" ]; then
        mkdir -p /app/dist/artifacts/arm64
        cp -r src-tauri/target/aarch64-unknown-linux-gnu/release/bundle/* /app/dist/artifacts/arm64/ 2>/dev/null || true
        cp src-tauri/target/aarch64-unknown-linux-gnu/release/atm /app/dist/artifacts/atm-linux-arm64 2>/dev/null || true
    fi
fi

# Show build results
echo "✅ Build completed!"
echo "📁 Artifacts location: /app/dist/artifacts"

if [ -d "/app/dist/artifacts" ]; then
    echo "📊 Build artifacts:"
    find /app/dist/artifacts -type f -exec ls -lh {} \; | awk '{print "   " $9 " (" $5 ")"}'
fi

echo "🎉 Multi-platform build finished!"
