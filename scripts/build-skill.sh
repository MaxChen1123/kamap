#!/usr/bin/env bash
#
# Build kamap and package into kamap-skill
#
# Usage:
#   ./scripts/build-skill.sh              # release build and package
#   ./scripts/build-skill.sh --debug      # debug build and package
#   ./scripts/build-skill.sh --target <triple>  # cross-compile
#
# This script will:
# 1. Run cargo build in the kamap-rust/ directory
# 2. Copy the built binary (kamap) to kamap-skill/bin/
# 3. Print packaging results

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$PROJECT_ROOT/kamap-rust"
SKILL_DIR="$PROJECT_ROOT/kamap-skill"
BIN_DIR="$SKILL_DIR/bin"

# Default parameters
BUILD_MODE="release"
TARGET=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_MODE="debug"
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--debug] [--target <triple>]"
            exit 1
            ;;
    esac
done

echo "=========================================="
echo "  kamap-skill 打包脚本"
echo "=========================================="
echo ""
echo "  Build mode:  $BUILD_MODE"
echo "  Rust dir:    $RUST_DIR"
echo "  Skill dir:   $SKILL_DIR"
if [[ -n "$TARGET" ]]; then
    echo "  Target:      $TARGET"
fi
echo ""

# 1. 检查 Rust 工具链
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

echo "📦 Step 1: Building kamap ($BUILD_MODE)..."
echo ""

# 2. Build
CARGO_ARGS=("build" "--manifest-path" "$RUST_DIR/Cargo.toml" "-p" "kamap-cli")
if [[ "$BUILD_MODE" == "release" ]]; then
    CARGO_ARGS+=("--release")
fi
if [[ -n "$TARGET" ]]; then
    CARGO_ARGS+=("--target" "$TARGET")
fi

cargo "${CARGO_ARGS[@]}"

echo ""
echo "✅ Build complete."

# 3. 确定产物路径
if [[ -n "$TARGET" ]]; then
    BUILD_DIR="$RUST_DIR/target/$TARGET/$BUILD_MODE"
else
    BUILD_DIR="$RUST_DIR/target/$BUILD_MODE"
fi

BINARY_NAME="kamap"
if [[ "$TARGET" == *"windows"* ]] || [[ "$(uname -s)" == *"MINGW"* && -z "$TARGET" ]]; then
    BINARY_NAME="kamap.exe"
fi

SOURCE_BINARY="$BUILD_DIR/$BINARY_NAME"

if [[ ! -f "$SOURCE_BINARY" ]]; then
    echo "❌ Binary not found at: $SOURCE_BINARY"
    exit 1
fi

# 4. 复制到 skill/bin
echo ""
echo "📂 Step 2: Copying binary to kamap-skill/bin/..."

mkdir -p "$BIN_DIR"
cp "$SOURCE_BINARY" "$BIN_DIR/$BINARY_NAME"
chmod +x "$BIN_DIR/$BINARY_NAME"

# 5. 清理 kamap-skill 目录内残留的旧 zip（防止 zip 套 zip）
rm -f "$SKILL_DIR"/*.zip

# 6. 打包 zip 到项目根目录
ZIP_NAME="kamap-skill.zip"
ZIP_PATH="$PROJECT_ROOT/$ZIP_NAME"
rm -f "$ZIP_PATH"

echo ""
echo "📦 Step 3: Creating $ZIP_NAME..."

(cd "$PROJECT_ROOT" && zip -r "$ZIP_PATH" kamap-skill/ -x "kamap-skill/.DS_Store" "kamap-skill/**/.DS_Store")

# 7. Print results
BINARY_SIZE=$(du -h "$BIN_DIR/$BINARY_NAME" | cut -f1)
ZIP_SIZE=$(du -h "$ZIP_PATH" | cut -f1)
echo ""
echo "=========================================="
echo "  ✅ Packaging complete!"
echo "=========================================="
echo ""
echo "  Binary: kamap-skill/bin/$BINARY_NAME"
echo "  Size:   $BINARY_SIZE"
echo ""
echo "  ZIP:    $ZIP_NAME"
echo "  Size:   $ZIP_SIZE"
echo ""
echo "  Verify: $BIN_DIR/$BINARY_NAME --version"
"$BIN_DIR/$BINARY_NAME" --version 2>/dev/null || echo "  (binary built successfully)"
