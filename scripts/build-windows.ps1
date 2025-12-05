#!/usr/bin/env pwsh
# PortKiller Windows Build Script
# Builds the release binary and creates a Windows installer

param(
    [switch]$SkipBuild,
    [switch]$SkipInstaller,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

# Detect project root (go up one level if we're in scripts directory)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if ((Split-Path -Leaf $ScriptDir) -eq "scripts") {
    $ProjectRoot = Split-Path -Parent $ScriptDir
    Set-Location $ProjectRoot
} else {
    $ProjectRoot = $ScriptDir
}

# Configuration
$AppName = "PortKiller"
$BinaryName = "portkiller"
$BuildDir = "target\release"
$InstallerDir = "installer"
$AssetsDir = "assets"

# Colors for output
function Write-Step {
    param([string]$Message)
    Write-Host "🔨 $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor Yellow
}

# Read version from Cargo.toml
function Get-Version {
    $cargoToml = Get-Content "Cargo.toml" -Raw
    if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
        return $Matches[1]
    }
    throw "Could not find version in Cargo.toml"
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "🚀 $AppName Windows Release Build Pipeline" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

$Version = Get-Version
Write-Host "Version: $Version" -ForegroundColor White
Write-Host ""

# Step 0: Clean if requested
if ($Clean) {
    Write-Step "Cleaning previous builds..."
    if (Test-Path $BuildDir) {
        Remove-Item -Path "$BuildDir\$BinaryName.exe" -ErrorAction SilentlyContinue
        Remove-Item -Path "$BuildDir\installer" -Recurse -ErrorAction SilentlyContinue
    }
    cargo clean --release
    Write-Success "Clean complete"
    Write-Host ""
}

# Step 1: Check for required tools
Write-Step "Checking required tools..."

# Check Rust/Cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
}
Write-Success "Cargo found: $(cargo --version)"

# Check for Inno Setup (optional)
$InnoSetupPath = $null
$PossiblePaths = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 5\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 5\ISCC.exe"
)

foreach ($path in $PossiblePaths) {
    if (Test-Path $path) {
        $InnoSetupPath = $path
        break
    }
}

if ($InnoSetupPath) {
    Write-Success "Inno Setup found: $InnoSetupPath"
} else {
    Write-Info "Inno Setup not found. Installer creation will be skipped."
    Write-Info "Download from: https://jrsoftware.org/isdl.php"
    $SkipInstaller = $true
}

Write-Host ""

# Step 2: Create Windows icon if needed
Write-Step "Checking for Windows icon..."
$IconPath = "$AssetsDir\app-icon.ico"

if (-not (Test-Path $IconPath)) {
    Write-Info "Windows icon not found. Creating from PNG..."
    
    # Check if ImageMagick is available
    if (Get-Command magick -ErrorAction SilentlyContinue) {
        $PngPath = "$AssetsDir\app-logo-color.png"
        if (Test-Path $PngPath) {
            magick convert $PngPath -define icon:auto-resize=256,128,64,48,32,16 $IconPath
            Write-Success "Icon created: $IconPath"
        } else {
            Write-Info "PNG source not found. Skipping icon creation."
        }
    } else {
        Write-Info "ImageMagick not found. Please create $IconPath manually or install ImageMagick."
        Write-Info "Download from: https://imagemagick.org/script/download.php"
    }
} else {
    Write-Success "Icon already exists: $IconPath"
}
Write-Host ""

# Step 3: Build release binary
if (-not $SkipBuild) {
    Write-Step "Building release binary..."
    Write-Host ""
    
    cargo build --release
    
    if (-not (Test-Path "$BuildDir\$BinaryName.exe")) {
        Write-Error "Build failed: $BinaryName.exe not found"
        exit 1
    }
    
    $FileSize = (Get-Item "$BuildDir\$BinaryName.exe").Length / 1MB
    Write-Success "Binary built: $BinaryName.exe ($([math]::Round($FileSize, 2)) MB)"
    Write-Host ""
} else {
    Write-Info "Skipping build (--SkipBuild specified)"
    Write-Host ""
}

# Step 4: Create installer
if (-not $SkipInstaller -and $InnoSetupPath) {
    Write-Step "Creating Windows installer..."
    
    # Ensure installer directory exists
    if (-not (Test-Path $InstallerDir)) {
        Write-Error "Installer directory not found: $InstallerDir"
        Write-Info "Please ensure portkiller.iss exists in the installer directory"
        exit 1
    }
    
    # Check for LICENSE file
    if (-not (Test-Path "LICENSE")) {
        Write-Info "LICENSE file not found. Creating MIT license..."
        @"
MIT License

Copyright (c) 2024 Samarth Gupta

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"@ | Out-File -FilePath "LICENSE" -Encoding UTF8
    }
    
    # Run Inno Setup compiler
    $IssFile = "$InstallerDir\portkiller.iss"
    if (Test-Path $IssFile) {
        Write-Host ""
        
        # Clean version for Windows file version info (must be numeric X.X.X.X)
        $FileVersion = $Version -replace '[^0-9.]', ''
        # Ensure it has enough components (e.g. 0.3.0 -> 0.3.0.0)
        while (($FileVersion.Split('.').Count) -lt 4) {
            $FileVersion += ".0"
        }

        # Define overrides for the script
        $ISCCArgs = @(
            "/DMyAppVersion=$Version",
            "/DMyAppFileVersion=$FileVersion",
            $IssFile
        )
        
        & $InnoSetupPath $ISCCArgs
        
        $InstallerPath = "$BuildDir\installer\PortKiller-$Version-Setup.exe"
        if (Test-Path $InstallerPath) {
            $InstallerSize = (Get-Item $InstallerPath).Length / 1MB
            Write-Success "Installer created: PortKiller-$Version-Setup.exe ($([math]::Round($InstallerSize, 2)) MB)"
        } else {
            Write-Error "Installer creation failed"
            exit 1
        }
    } else {
        Write-Error "Inno Setup script not found: $IssFile"
        exit 1
    }
    Write-Host ""
} else {
    Write-Info "Skipping installer creation"
    Write-Host ""
}

# Step 4.5: Create Portable Version
Write-Step "Creating Portable Zip..."
$PortableDir = "$BuildDir\portable"
$ZipPath = "$PortableDir\PortKiller-$Version-Portable.zip"

# Ensure portable directory exists
if (-not (Test-Path $PortableDir)) {
    New-Item -ItemType Directory -Path $PortableDir | Out-Null
}

$PortableFiles = @(
    "$BuildDir\$BinaryName.exe",
    "README.md",
    "LICENSE"
)

# Verify all files exist
$FilesToZip = @()
foreach ($file in $PortableFiles) {
    if (Test-Path $file) {
        $FilesToZip += $file
    } else {
        Write-Info "Warning: Portable file not found: $file"
    }
}

if ($FilesToZip.Count -gt 0) {
    # Remove existing zip if it exists
    if (Test-Path $ZipPath) {
        Remove-Item $ZipPath -Force
    }
    
    Compress-Archive -Path $FilesToZip -DestinationPath $ZipPath
    
    if (Test-Path $ZipPath) {
        $ZipSize = (Get-Item $ZipPath).Length / 1MB
        Write-Success "Portable Zip created: PortKiller-$Version-Portable.zip ($([math]::Round($ZipSize, 2)) MB)"
    } else {
        Write-Error "Failed to create portable zip"
    }
} else {
    Write-Error "No files to zip!"
}
Write-Host ""

# Step 5: Summary
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "🎉 Build Complete!" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
Write-Host "Artifacts created:" -ForegroundColor White

if (Test-Path "$BuildDir\$BinaryName.exe") {
    Write-Host "  • Binary: $BuildDir\$BinaryName.exe" -ForegroundColor Green
}

$InstallerPath = "$BuildDir\installer\PortKiller-$Version-Setup.exe"
if (Test-Path $InstallerPath) {
    Write-Host "  • Installer: $InstallerPath" -ForegroundColor Green
}

$ZipPath = "$BuildDir\portable\PortKiller-$Version-Portable.zip"
if (Test-Path $ZipPath) {
    Write-Host "  • Portable:  $ZipPath" -ForegroundColor Green
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test the binary: .\$BuildDir\$BinaryName.exe"
if (Test-Path $InstallerPath) {
    Write-Host "  2. Test the installer: .\$InstallerPath"
    Write-Host "  3. Create a GitHub release: gh release create v$Version"
    Write-Host "  4. Upload the installer to the release"
}
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
