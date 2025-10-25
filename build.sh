#!/bin/bash
# macOS/Linux Build Script for Tauri OAuth App

echo "🚀 Building Tauri OAuth App..."

# Detect OS
OS=$(uname -s)
case $OS in
    Darwin)
        PLATFORM="macOS"
        ;;
    Linux)
        PLATFORM="Linux"
        ;;
    *)
        echo "❌ Unsupported operating system: $OS"
        exit 1
        ;;
esac

echo "📱 Platform: $PLATFORM"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if Node.js is installed
if ! command -v npm &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first:"
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

# Install system dependencies for Linux
if [ "$PLATFORM" = "Linux" ]; then
    echo "📦 Checking Linux dependencies..."
    
    # Check if we're on Ubuntu/Debian
    if command -v apt-get &> /dev/null; then
        echo "🔧 Installing required packages..."
        sudo apt-get update
        sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf
    else
        echo "⚠️  Please install the following packages manually:"
        echo "   - libgtk-3-dev"
        echo "   - libwebkit2gtk-4.0-dev" 
        echo "   - libappindicator3-dev"
        echo "   - librsvg2-dev"
        echo "   - patchelf"
    fi
fi

# Check if Tauri CLI is installed
if ! command -v cargo-tauri &> /dev/null; then
    echo "📦 Installing Tauri CLI..."
    cargo install tauri-cli
fi

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
npm install

# Build the application
echo "🔨 Building application..."
cargo tauri build

if [ $? -eq 0 ]; then
    echo "✅ Build completed successfully!"
    echo ""
    echo "📁 Build artifacts:"
    
    if [ "$PLATFORM" = "macOS" ]; then
        echo "   App Bundle: src-tauri/target/release/bundle/macos/"
        echo "   DMG Installer: src-tauri/target/release/bundle/dmg/"
        
        # Show file sizes
        if [ -d "src-tauri/target/release/bundle/macos" ]; then
            for app in src-tauri/target/release/bundle/macos/*.app; do
                if [ -d "$app" ]; then
                    size=$(du -sh "$app" | cut -f1)
                    echo "   📊 App size: $size"
                fi
            done
        fi
        
        if [ -d "src-tauri/target/release/bundle/dmg" ]; then
            for dmg in src-tauri/target/release/bundle/dmg/*.dmg; do
                if [ -f "$dmg" ]; then
                    size=$(du -sh "$dmg" | cut -f1)
                    echo "   📊 DMG size: $size"
                fi
            done
        fi
        
    elif [ "$PLATFORM" = "Linux" ]; then
        echo "   Executable: src-tauri/target/release/tauri-oauth-app"
        echo "   DEB Package: src-tauri/target/release/bundle/deb/"
        echo "   AppImage: src-tauri/target/release/bundle/appimage/"
        
        # Show file sizes
        if [ -f "src-tauri/target/release/tauri-oauth-app" ]; then
            size=$(du -sh "src-tauri/target/release/tauri-oauth-app" | cut -f1)
            echo "   📊 Executable size: $size"
        fi
        
        if [ -d "src-tauri/target/release/bundle/deb" ]; then
            for deb in src-tauri/target/release/bundle/deb/*.deb; do
                if [ -f "$deb" ]; then
                    size=$(du -sh "$deb" | cut -f1)
                    echo "   📊 DEB size: $size"
                fi
            done
        fi
        
        if [ -d "src-tauri/target/release/bundle/appimage" ]; then
            for appimage in src-tauri/target/release/bundle/appimage/*.AppImage; do
                if [ -f "$appimage" ]; then
                    size=$(du -sh "$appimage" | cut -f1)
                    echo "   📊 AppImage size: $size"
                fi
            done
        fi
    fi
else
    echo "❌ Build failed!"
    exit 1
fi
