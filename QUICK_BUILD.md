# PortKiller Windows Build - Quick Reference

## 🚀 Quick Build Commands

### Full Build (Binary + Installer)
```powershell
cd scripts
.\build-windows.ps1
```

### Clean Build
```powershell
.\build-windows.ps1 -Clean
```

### Binary Only (No Installer)
```powershell
.\build-windows.ps1 -SkipInstaller
```

## 📦 Output Locations

| Artifact | Location |
|----------|----------|
| Binary | `target\release\portkiller.exe` |
| Installer | `target\release\installer\PortKiller-0.3.0-Setup.exe` |
| Icon | `assets\app-icon.ico` |

## ⚡ Prerequisites

```powershell
# Install Rust
winget install Rustlang.Rustup

# Install Inno Setup (for installer)
choco install innosetup -y

# Install Python Pillow (for icon creation)
pip install Pillow
```

## 🎯 Common Tasks

### Create Icon from PNG
```powershell
python scripts\create-icon.py
```

### Test the Binary
```powershell
.\target\release\portkiller.exe
```

### Create GitHub Release
```powershell
git tag v0.3.0
git push origin v0.3.0
# GitHub Actions will build and release automatically
```

### Manual Installer Build
```powershell
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\portkiller.iss
```

## 📖 Documentation

- **`INSTALLER_SETUP.md`** - Complete setup guide and current status
- **`BUILD_WINDOWS.md`** - Detailed build instructions
- **`installer/README.md`** - Installer customization guide

## ✅ Current Status

- ✅ Binary built: `target\release\portkiller.exe` (1.29 MB)
- ✅ Icon created: `assets\app-icon.ico` (15.5 KB)
- ⚠️ Installer: Requires Inno Setup installation

## 🔧 Troubleshooting

| Issue | Solution |
|-------|----------|
| Inno Setup not found | `choco install innosetup -y` |
| Cargo not found | `winget install Rustlang.Rustup` |
| Icon creation fails | `pip install Pillow` |
| Build fails | Install Visual Studio Build Tools |

---

**Need more details?** See `INSTALLER_SETUP.md`
