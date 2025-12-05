# Building PortKiller for Windows

This guide will help you build a full installer release for PortKiller on Windows.

## 🚀 Quick Start

### Prerequisites
1. **Install Rust** (if not already installed)
   ```powershell
   # Download and run rustup-init.exe from https://rustup.rs/
   # Or use winget:
   winget install Rustlang.Rustup
   ```

2. **Install Inno Setup** (for creating the installer)
   ```powershell
   # Using Chocolatey:
   choco install innosetup -y
   
   # Or download manually from:
   # https://jrsoftware.org/isdl.php
   ```

3. **Install ImageMagick** (optional, for icon conversion)
   ```powershell
   # Using Chocolatey:
   choco install imagemagick -y
   
   # Or download manually from:
   # https://imagemagick.org/script/download.php
   ```

### Build the Installer

**Option 1: Using the batch file (easiest)**
```batch
cd scripts
build-windows.bat
```

**Option 2: Using PowerShell**
```powershell
cd scripts
.\build-windows.ps1
```

**Option 3: Clean build**
```powershell
cd scripts
.\build-windows.ps1 -Clean
```

## 📦 What Gets Built

After running the build script, you'll have:

1. **Release Binary**
   - Location: `target\release\portkiller.exe`
   - Optimized for size and performance
   - Ready to run standalone

2. **Windows Installer**
   - Location: `target\release\installer\PortKiller-{version}-Setup.exe`
   - Professional installer with wizard
   - Includes uninstaller
   - Creates Start Menu shortcuts
   - Optional desktop icon
   - Optional startup entry

## 🎯 Build Options

```powershell
# Full clean build (recommended for releases)
.\build-windows.ps1 -Clean

# Only build binary (skip installer)
.\build-windows.ps1 -SkipInstaller

# Only create installer (binary must exist)
.\build-windows.ps1 -SkipBuild

# Combine options
.\build-windows.ps1 -Clean -SkipInstaller
```

## 🧪 Testing

### Test the Binary
```powershell
# Run directly
.\target\release\portkiller.exe

# Check version
.\target\release\portkiller.exe --version
```

### Test the Installer
```powershell
# Run the installer
.\target\release\installer\PortKiller-0.3.0-Setup.exe

# The installer will:
# 1. Show a setup wizard
# 2. Install to C:\Program Files\PortKiller
# 3. Create shortcuts
# 4. Optionally add to startup
# 5. Launch the app
```

## 📤 Distribution

### Manual Distribution
Simply share the installer file:
```
PortKiller-{version}-Setup.exe
```

Users can download and run it - no additional dependencies needed.

### GitHub Release
```powershell
# Create a new release tag
git tag v0.3.0
git push origin v0.3.0

# The GitHub Actions workflow will automatically:
# 1. Build the Windows installer
# 2. Build the macOS DMG
# 3. Create a GitHub release
# 4. Upload both artifacts

# Or create manually:
gh release create v0.3.0 `
  --title "PortKiller v0.3.0" `
  --notes "Release notes here" `
  target\release\installer\PortKiller-0.3.0-Setup.exe
```

## 🔧 Customization

### Change App Icon
1. Replace `assets\app-icon.ico` with your icon
2. Or provide `assets\app-logo-color.png` and it will be converted

### Modify Installer Settings
Edit `installer\portkiller.iss`:
- Installation directory
- Start menu group
- Default options
- Files to include
- Registry keys

### Adjust Build Settings
Edit `Cargo.toml` under `[profile.release]`:
```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Remove debug symbols
panic = "abort"      # Smaller binary
```

## 🐛 Troubleshooting

### "Inno Setup not found"
**Solution**: Install Inno Setup or use `-SkipInstaller`:
```powershell
.\build-windows.ps1 -SkipInstaller
```

### "cargo: command not found"
**Solution**: Install Rust and restart your terminal:
```powershell
winget install Rustlang.Rustup
# Then restart PowerShell
```

### "ImageMagick not found"
**Solution**: Either:
1. Install ImageMagick: `choco install imagemagick -y`
2. Or manually create `assets\app-icon.ico`

### Build fails with linker errors
**Solution**: Install Visual Studio Build Tools:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

### Installer shows SmartScreen warning
**Solution**: This is normal for unsigned installers. For production:
1. Get a code signing certificate
2. Sign the installer with `signtool`
3. This removes the warning

## 📋 Build Checklist

Before creating a release:

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` (if you have one)
- [ ] Test the build: `.\build-windows.ps1 -Clean`
- [ ] Test the binary: `.\target\release\portkiller.exe`
- [ ] Test the installer: Install and verify functionality
- [ ] Create git tag: `git tag v{version}`
- [ ] Push tag: `git push origin v{version}`
- [ ] Verify GitHub Actions build succeeds
- [ ] Download and test the release artifacts
- [ ] Update README.md with new download links

## 🎓 Advanced Topics

### Code Signing
For production releases, sign your installer:
```powershell
# Get a code signing certificate from a CA
# Then sign the installer:
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com target\release\installer\PortKiller-0.3.0-Setup.exe
```

### Cross-Compilation
Build for different architectures:
```powershell
# For ARM64 Windows
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc
```

### Portable Version
Create a portable version (no installer):
```powershell
# Build the binary
cargo build --release

# Create a zip with the binary and config
Compress-Archive -Path target\release\portkiller.exe,README.md -DestinationPath PortKiller-Portable.zip
```

## 📚 Additional Resources

- [Inno Setup Documentation](https://jrsoftware.org/ishelp/)
- [Rust Release Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Windows Code Signing](https://docs.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)
- [GitHub Actions for Rust](https://github.com/actions-rs)

## 💬 Need Help?

- Open an issue on GitHub
- Check the [installer README](installer/README.md)
- Review the [build script](scripts/build-windows.ps1)

---

**Happy Building! 🎉**
