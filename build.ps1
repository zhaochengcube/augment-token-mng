# Windows Build Script for Tauri OAuth App
# Run this script in PowerShell

Write-Host "🚀 Building Tauri OAuth App for Windows..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust is not installed. Please install Rust first:" -ForegroundColor Red
    Write-Host "   Visit: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Check if Node.js is installed
if (!(Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js is not installed. Please install Node.js first:" -ForegroundColor Red
    Write-Host "   Visit: https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}

# Check if Tauri CLI is installed
if (!(Get-Command cargo-tauri -ErrorAction SilentlyContinue)) {
    Write-Host "📦 Installing Tauri CLI..." -ForegroundColor Yellow
    cargo install tauri-cli
}

# Install frontend dependencies
Write-Host "📦 Installing frontend dependencies..." -ForegroundColor Yellow
npm install

# Build the application
Write-Host "🔨 Building application..." -ForegroundColor Blue
cargo tauri build

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📁 Build artifacts:" -ForegroundColor Cyan
    Write-Host "   Executable: src-tauri\target\release\tauri-oauth-app.exe" -ForegroundColor White
    Write-Host "   MSI Installer: src-tauri\target\release\bundle\msi\" -ForegroundColor White
    Write-Host ""
    
    # Check if files exist and show their sizes
    $exe_path = "src-tauri\target\release\tauri-oauth-app.exe"
    if (Test-Path $exe_path) {
        $size = (Get-Item $exe_path).Length / 1MB
        Write-Host "   📊 Executable size: $([math]::Round($size, 2)) MB" -ForegroundColor Gray
    }
    
    $msi_dir = "src-tauri\target\release\bundle\msi"
    if (Test-Path $msi_dir) {
        $msi_files = Get-ChildItem $msi_dir -Filter "*.msi"
        foreach ($msi in $msi_files) {
            $size = $msi.Length / 1MB
            Write-Host "   📊 MSI size: $([math]::Round($size, 2)) MB" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
