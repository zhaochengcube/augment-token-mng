#!/bin/bash
# Docker build script for Tauri OAuth App

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_color() {
    printf "${1}${2}${NC}\n"
}

print_color $BLUE "🐳 Tauri OAuth App Docker Build Script"
echo

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_color $RED "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    print_color $RED "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Function to show usage
show_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo
    echo "Commands:"
    echo "  linux      Build for Linux only (fast)"
    echo "  cross      Build for multiple platforms (slow)"
    echo "  dev        Start development environment"
    echo "  extract    Extract and show build artifacts"
    echo "  clean      Clean all Docker images and containers"
    echo
    echo "Options:"
    echo "  --no-cache    Build without using Docker cache"
    echo "  --help       Show this help message"
    echo
    echo "Examples:"
    echo "  $0 linux              # Build Linux version"
    echo "  $0 cross --no-cache   # Cross-platform build without cache"
    echo "  $0 dev                # Start development environment"
}

# Parse arguments
COMMAND=""
BUILD_ARGS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        linux|cross|dev|extract|clean)
            COMMAND="$1"
            shift
            ;;
        --no-cache)
            BUILD_ARGS="$BUILD_ARGS --no-cache"
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            print_color $RED "❌ Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# If no command specified, show usage
if [[ -z "$COMMAND" ]]; then
    show_usage
    exit 1
fi

# Create artifacts directory
mkdir -p dist/docker-artifacts

# Execute command
case $COMMAND in
    linux)
        print_color $GREEN "🚀 Building for Linux..."
        docker-compose --profile linux --profile build build $BUILD_ARGS
        docker-compose --profile linux run --rm tauri-build-linux
        print_color $GREEN "✅ Linux build completed!"
        ;;
    cross)
        print_color $GREEN "🚀 Building for multiple platforms..."
        docker-compose --profile cross --profile build build $BUILD_ARGS
        docker-compose --profile cross run --rm tauri-build-cross
        print_color $GREEN "✅ Cross-platform build completed!"
        ;;
    dev)
        print_color $GREEN "🚀 Starting development environment..."
        print_color $YELLOW "⚠️  Note: GUI applications may not work without proper X11 setup"
        docker-compose --profile dev build $BUILD_ARGS
        docker-compose --profile dev run --rm tauri-dev
        ;;
    extract)
        print_color $GREEN "📦 Extracting build artifacts..."
        docker-compose --profile extract run --rm artifacts-extractor
        ;;
    clean)
        print_color $YELLOW "🧹 Cleaning Docker images and containers..."
        docker-compose down --remove-orphans
        docker system prune -f
        docker volume prune -f
        print_color $GREEN "✅ Cleanup completed!"
        ;;
esac

print_color $BLUE "🎉 Docker operation completed!"
