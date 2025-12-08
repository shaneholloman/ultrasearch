
$ErrorActionPreference = "Stop"

function Get-WorkspaceVersion {
    param([string]$CargoTomlPath = "Cargo.toml")
    if (-not (Test-Path $CargoTomlPath)) { return $null }
    $content = Get-Content $CargoTomlPath -Raw
    $match = [regex]::Match($content, '(?m)^\s*version\s*=\s*\"([^\"]+)\"')
    if ($match.Success) { return $match.Groups[1].Value }
    return $null
}

$Version = $Env:ULTRASEARCH_VERSION
if (-not $Version) {
    $Version = Get-WorkspaceVersion
}
if (-not $Version) {
    $Version = "0.1.0"
}

Write-Host "Building release binaries (version $Version)..."
cargo build --release -p service
cargo build --release -p ui
cargo build --release -p launcher

Write-Host "Checking for WiX Toolset..."
if (-not (Get-Command "candle.exe" -ErrorAction SilentlyContinue)) {
    Write-Warning "WiX Toolset (candle.exe/light.exe) not found in PATH."
    Write-Warning "Please install WiX Toolset v3.11 or v4: https://wixtoolset.org/releases/"
    Write-Warning "Skipping MSI generation."
    exit 0
}

$WxsFile = "ultrasearch\wix\main.wxs"
$ObjFile = "target\wix\main.wixobj"
$MsiFile = "target\wix\UltraSearch-$Version.msi"

New-Item -ItemType Directory -Force -Path "target\wix" | Out-Null

Write-Host "Compiling WiX source..."
candle.exe -nologo -out $ObjFile $WxsFile -arch x64 -ext WixUtilExtension -dProductVersion="$Version"

Write-Host "Linking MSI..."
light.exe -nologo -out $MsiFile $ObjFile -ext WixUtilExtension -cultures:en-us

Write-Host "Success! MSI created at: $MsiFile"
