# PortKiller Windows Installer

This directory contains the configuration and scripts needed to build a Windows installer for PortKiller.

## Prerequisites

### Required
- **Rust & Cargo** - Install from [rustup.rs](https://rustup.rs/)
- **Windows 10/11** - 64-bit

### Optional (for installer creation)
- **Inno Setup 6** - Download from [jrsoftware.org/isdl.php](https://jrsoftware.org/isdl.php)
  - Required to create the `.exe` installer
  - Without it, you can still build the binary
  
- **ImageMagick** - Download from [imagemagick.org](https://imagemagick.org/script/download.php)
  - Required to create `.ico` icon from PNG
  - Optional if `app-icon.ico` already exists

## Quick Start

### Option 1: Using the Batch File (Easiest)
```batch
cd scripts
build-windows.bat
```

### Option 2: Using PowerShell Directly
```powershell
cd scripts
.\build-windows.ps1
```

### Option 3: Manual Build
```powershell
# Build the binary
cargo build --release

# The binary will be at: target\release\portkiller.exe

# To create the installer (requires Inno Setup)
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\portkiller.iss
```

## Build Options

The PowerShell script supports several options:

```powershell
# Clean build (removes previous artifacts)
.\build-windows.ps1 -Clean

# Skip building (only create installer from existing binary)
.\build-windows.ps1 -SkipBuild

# Skip installer creation (only build binary)
.\build-windows.ps1 -SkipInstaller

# Combine options
.\build-windows.ps1 -Clean -SkipInstaller
```

## Output

After a successful build, you'll find:

- **Binary**: `target\release\portkiller.exe`
- **Installer**: `target\release\installer\PortKiller-{version}-Setup.exe`

## Installer Features

The Windows installer includes:

✅ **Automatic Installation** - Installs to `C:\Program Files\PortKiller`  
✅ **Start Menu Shortcuts** - Adds shortcuts to Start Menu  
✅ **Desktop Icon** - Optional desktop shortcut  
✅ **Startup Option** - Optional launch at Windows startup  
✅ **Default Config** - Creates `~\.portkiller.json` with sensible defaults  
✅ **Clean Uninstall** - Removes all files and registry entries  
✅ **Version Detection** - Detects and offers to uninstall previous versions  

## Customization

### Changing the App Icon

1. Replace `assets\app-icon.ico` with your custom icon
2. Or provide a PNG at `assets\app-logo-color.png` and the build script will convert it

### Modifying Installer Behavior

Edit `installer\portkiller.iss` to customize:
- Installation directory
- Start menu group name
- Default tasks (desktop icon, startup)
- Files to include
- Registry keys
- Post-install actions

See the [Inno Setup documentation](https://jrsoftware.org/ishelp/) for details.

## Troubleshooting

### "Inno Setup not found"
- Download and install from [jrsoftware.org/isdl.php](https://jrsoftware.org/isdl.php)
- Or use `-SkipInstaller` to only build the binary

### "ImageMagick not found"
- Download from [imagemagick.org](https://imagemagick.org/script/download.php)
- Or manually create `assets\app-icon.ico` using an icon editor

### "cargo: command not found"
- Install Rust from [rustup.rs](https://rustup.rs/)
- Restart your terminal after installation

### Build fails with linking errors
- Make sure you have the Windows SDK installed
- Install Visual Studio Build Tools if needed

## Distribution

### GitHub Release
```powershell
# Create a new release
gh release create v0.3.0 --title "PortKiller v0.3.0" --notes "Release notes here"

# Upload the installer
gh release upload v0.3.0 target\release\installer\PortKiller-0.3.0-Setup.exe
```

### Manual Distribution
Simply share the `PortKiller-{version}-Setup.exe` file. Users can:
1. Download the installer
2. Run it (may require admin privileges)
3. Follow the installation wizard
4. Launch PortKiller from Start Menu or Desktop

## Code Signing (Optional)

For production releases, consider code signing the installer:

1. Obtain a code signing certificate
2. Add to `portkiller.iss`:
   ```ini
   [Setup]
   SignTool=signtool
   SignedUninstaller=yes
   ```
3. Configure `signtool` in Inno Setup

This removes Windows SmartScreen warnings and builds user trust.

## License

MIT License - See LICENSE file for details
