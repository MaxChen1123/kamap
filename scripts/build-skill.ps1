#
# Build kamap and package into kamap-skill (Windows PowerShell)
#
# Usage:
#   .\scripts\build-skill.ps1              # release build and package
#   .\scripts\build-skill.ps1 -Debug       # debug build and package
#   .\scripts\build-skill.ps1 -Target <triple>  # cross-compile
#

param(
    [switch]$Debug,
    [string]$Target = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$RustDir = Join-Path $ProjectRoot "kamap-rust"
$SkillDir = Join-Path $ProjectRoot "kamap-skill"
$BinDir = Join-Path $SkillDir "bin"

# 确定编译模式
$BuildMode = if ($Debug) { "debug" } else { "release" }

Write-Host "=========================================="
Write-Host "  kamap-skill 打包脚本 (Windows)"
Write-Host "=========================================="
Write-Host ""
Write-Host "  Build mode:  $BuildMode"
Write-Host "  Rust dir:    $RustDir"
Write-Host "  Skill dir:   $SkillDir"
if ($Target) {
    Write-Host "  Target:      $Target"
}
Write-Host ""

# 1. 检查 Rust 工具链
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ cargo not found. Please install Rust: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

Write-Host "📦 Step 1: Building kamap ($BuildMode)..."
Write-Host ""

# 2. 编译
$CargoArgs = @("build", "--manifest-path", "$RustDir\Cargo.toml", "-p", "kamap-cli")
if ($BuildMode -eq "release") {
    $CargoArgs += "--release"
}
if ($Target) {
    $CargoArgs += @("--target", $Target)
}

& cargo @CargoArgs
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Build complete."

# 3. 确定产物路径
if ($Target) {
    $BuildDir = Join-Path $RustDir "target\$Target\$BuildMode"
} else {
    $BuildDir = Join-Path $RustDir "target\$BuildMode"
}

# Determine binary name
$BinaryName = if ($Target -like "*windows*" -or (-not $Target)) {
    "kamap.exe"
} else {
    "kamap"
}

$SourceBinary = Join-Path $BuildDir $BinaryName

if (-not (Test-Path $SourceBinary)) {
    Write-Host "❌ Binary not found at: $SourceBinary" -ForegroundColor Red
    exit 1
}

# 4. 复制到 skill/bin
Write-Host ""
Write-Host "📂 Step 2: Copying binary to kamap-skill\bin\..."

if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

Copy-Item $SourceBinary (Join-Path $BinDir $BinaryName) -Force

# 5. 清理 kamap-skill 目录内残留的旧 zip（防止 zip 套 zip）
Get-ChildItem -Path $SkillDir -Filter "*.zip" -ErrorAction SilentlyContinue | Remove-Item -Force

# 6. 打包 zip 到项目根目录
$ZipName = "kamap-skill.zip"
$ZipPath = Join-Path $ProjectRoot $ZipName
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}

Write-Host ""
Write-Host "📦 Step 3: Creating $ZipName..."

Compress-Archive -Path $SkillDir -DestinationPath $ZipPath -Force

# 7. 输出结果
$BinaryPath = Join-Path $BinDir $BinaryName
$BinarySize = (Get-Item $BinaryPath).Length
$SizeDisplay = if ($BinarySize -ge 1MB) {
    "{0:N1} MB" -f ($BinarySize / 1MB)
} elseif ($BinarySize -ge 1KB) {
    "{0:N1} KB" -f ($BinarySize / 1KB)
} else {
    "$BinarySize B"
}

$ZipSize = (Get-Item $ZipPath).Length
$ZipSizeDisplay = if ($ZipSize -ge 1MB) {
    "{0:N1} MB" -f ($ZipSize / 1MB)
} elseif ($ZipSize -ge 1KB) {
    "{0:N1} KB" -f ($ZipSize / 1KB)
} else {
    "$ZipSize B"
}

Write-Host ""
Write-Host "=========================================="
Write-Host "  ✅ 打包完成!"
Write-Host "=========================================="
Write-Host ""
Write-Host "  Binary: kamap-skill\bin\$BinaryName"
Write-Host "  Size:   $SizeDisplay"
Write-Host ""
Write-Host "  ZIP:    $ZipName"
Write-Host "  Size:   $ZipSizeDisplay"
Write-Host ""
Write-Host "  验证: $BinaryPath --version"
try {
    & $BinaryPath --version
} catch {
    Write-Host "  (binary built successfully)"
}
