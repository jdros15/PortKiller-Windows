# PortKiller - Windows Installer Release Setup

## ✅ What's Been Created

I've set up a complete Windows installer build system for PortKiller. Here's what's ready:

### 📁 Files Created

1. **`installer/portkiller.iss`** - Inno Setup installer configuration
   - Professional Windows installer with wizard
   - Automatic version detection and upgrade handling
   - Creates Start Menu shortcuts
   - Optional desktop icon and startup entry
   - Creates default config file
   - Clean uninstall support

2. **`scripts/build-windows.ps1`** - PowerShell build script
   - Automated build process
   - Checks for required tools
   - Creates Windows icon from PNG
   - Builds release binary
   - Creates installer (if Inno Setup is installed)
   - Comprehensive error handling

3. **`scripts/build-windows.bat`** - Batch file wrapper
   - Simple double-click execution
   - Works with both PowerShell Core and Windows PowerShell

4. **`scripts/create-icon.py`** - Icon creation utility
   - Converts PNG to ICO format
   - Creates multi-resolution icon (256x256 down to 16x16)
   - Already executed - icon created at `assets/app-icon.ico`

5. **`.github/workflows/release.yml`** - GitHub Actions workflow
   - Automated builds on tag push
   - Builds both Windows installer and macOS DMG
   - Uploads artifacts
   - Creates GitHub releases automatically

6. **`BUILD_WINDOWS.md`** - Comprehensive build guide
   - Step-by-step instructions
   - Troubleshooting section
   - Distribution guidelines
   - Advanced topics

7. **`installer/README.md`** - Installer documentation
   - Quick start guide
   - Customization options
   - Build options
   - Distribution methods

8. **`LICENSE`** - MIT License file
   - Required by the installer
   - Standard MIT license

9. **`assets/app-icon.ico`** - Windows icon
   - ✅ Already created (15.5 KB)
   - Multi-resolution (256x256 to 16x16)
   - Ready to use

### 🎯 Current Status

✅ **Binary Built Successfully**
- Location: `target\release\portkiller.exe`
- Size: 1.29 MB
- Optimized for release

⚠️ **Installer Not Created**
- Reason: Inno Setup not installed
- Solution: Install Inno Setup (see below)

## 🚀 Next Steps

### Option 1: Create Full Installer (Recommended)

1. **Install Inno Setup**
   ```powershell
   # Using Chocolatey (easiest):
   choco install innosetup -y
   
   # Or download manually from:
   # https://jrsoftware.org/isdl.php
   ```

2. **Run the build script again**
   ```powershell
   cd scripts
   .\build-windows.ps1
   ```

3. **You'll get:**
   - `target\release\portkiller.exe` (binary)
   - `target\release\installer\PortKiller-0.3.0-Setup.exe` (installer)

### Option 2: Use Binary Only

The binary is already built and ready to use:
```powershell
# Test it:
.\target\release\portkiller.exe

# Distribute it:
# Just share the portkiller.exe file
```

### Option 3: Manual Installer Creation

If you have Inno Setup installed:
```powershell
# Compile the installer manually:
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\portkiller.iss
```

## 📦 Distribution Options

### For End Users (Recommended)
Create the installer and distribute `PortKiller-0.3.0-Setup.exe`:
- Professional installation experience
- Automatic shortcuts and uninstaller
- No manual configuration needed
- Windows SmartScreen compatible

### For Developers
Distribute the binary `portkiller.exe`:
- Portable, no installation needed
- Requires manual config file creation
- Good for testing and development

### For GitHub Releases
Use the automated workflow:
```powershell
# Create and push a tag:
git tag v0.3.0
git push origin v0.3.0

# GitHub Actions will automatically:
# - Build Windows installer
# - Build macOS DMG
# - Create GitHub release
# - Upload both artifacts
```

## 🧪 Testing Checklist

Before distributing:

- [ ] Test the binary: `.\target\release\portkiller.exe`
- [ ] Verify system tray icon appears
- [ ] Test port killing functionality
- [ ] Check config file creation (`~\.portkiller.json`)
- [ ] Test the installer (if created)
- [ ] Verify shortcuts are created
- [ ] Test uninstaller
- [ ] Check on a clean Windows machine

## 📝 Installer Features

The Windows installer includes:

✅ **Professional Installation**
- Wizard-based setup
- Custom installation directory
- Progress indicators

✅ **Shortcuts**
- Start Menu entry
- Optional desktop icon
- Optional startup entry

✅ **Configuration**
- Automatic config file creation
- Sensible defaults for Windows
- Docker and Windows Services integration

✅ **Upgrade Support**
- Detects previous versions
- Offers to uninstall old version
- Preserves user configuration

✅ **Clean Uninstall**
- Removes all files
- Removes shortcuts
- Optionally removes config

## 🔧 Customization

### Change Version
Edit `Cargo.toml`:
```toml
[package]
version = "0.4.0"  # Update this
```
The installer will automatically use the new version.

### Modify Installer Behavior
Edit `installer/portkiller.iss`:
- Change installation directory
- Add/remove shortcuts
- Modify default tasks
- Add registry keys
- Customize wizard pages

### Adjust Binary Size
Edit `Cargo.toml` under `[profile.release]`:
```toml
opt-level = "z"      # "z" for size, "3" for speed
strip = true         # Remove debug symbols
lto = true           # Link-time optimization
```

## 📚 Documentation

- **Build Guide**: `BUILD_WINDOWS.md` - Complete build instructions
- **Installer README**: `installer/README.md` - Installer details
- **Main README**: `README.md` - Project overview

## 🆘 Troubleshooting

### "Inno Setup not found"
Install from https://jrsoftware.org/isdl.php or use Chocolatey:
```powershell
choco install innosetup -y
```

### "Build failed"
Check you have:
- Rust installed: `cargo --version`
- Windows SDK (comes with Visual Studio Build Tools)

### "Icon not found"
The icon was already created at `assets/app-icon.ico`. If missing:
```powershell
python scripts/create-icon.py
```

## 🎉 Summary

You now have a complete Windows installer build system:

1. ✅ Binary is built and ready
2. ✅ Icon is created
3. ✅ Build scripts are ready
4. ✅ Installer configuration is ready
5. ⚠️ Just need to install Inno Setup to create the installer

**To create the full installer:**
```powershell
# Install Inno Setup
choco install innosetup -y

# Build everything
cd scripts
.\build-windows.ps1

# Result: Professional Windows installer ready for distribution!
```

---

**Questions?** Check `BUILD_WINDOWS.md` or `installer/README.md` for detailed documentation.
