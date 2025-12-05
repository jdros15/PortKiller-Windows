# PortKiller Windows Release Checklist

Use this checklist when preparing a new release.

## Pre-Release

- [ ] Update version in `Cargo.toml`
- [ ] Update version references in documentation (if any)
- [ ] Test all features on Windows:
  - [ ] Port monitoring works
  - [ ] Process killing works
  - [ ] System tray icon displays correctly
  - [ ] Notifications work
  - [ ] Config file is created/loaded
  - [ ] Docker integration works (if Docker installed)
  - [ ] Windows Services integration works

## Build

- [ ] Clean build: `.\scripts\build-windows.ps1 -Clean`
- [ ] Verify binary size is reasonable (~1-2 MB)
- [ ] Test binary on development machine
- [ ] Install Inno Setup (if not already installed)
- [ ] Create installer: `.\scripts\build-windows.ps1`
- [ ] Verify installer was created successfully

## Testing

- [ ] Test binary standalone:
  - [ ] Run `.\target\release\portkiller.exe`
  - [ ] Check system tray icon appears
  - [ ] Kill a test process
  - [ ] Verify config file creation
  
- [ ] Test installer on clean VM or test machine:
  - [ ] Run installer
  - [ ] Verify installation directory
  - [ ] Check Start Menu shortcuts created
  - [ ] Test desktop icon (if selected)
  - [ ] Launch application
  - [ ] Test all features
  - [ ] Uninstall and verify clean removal

## Release

- [ ] Commit all changes
- [ ] Create git tag: `git tag v{version}`
- [ ] Push tag: `git push origin v{version}`
- [ ] Wait for GitHub Actions to complete
- [ ] Download and test artifacts from GitHub
- [ ] Update release notes on GitHub
- [ ] Test download links

## Post-Release

- [ ] Update README.md download links (if needed)
- [ ] Announce release (if applicable)
- [ ] Monitor for issues
- [ ] Respond to user feedback

## Optional Enhancements

- [ ] Code sign the installer (requires certificate)
- [ ] Create portable version (zip with binary)
- [ ] Update screenshots if UI changed
- [ ] Create video demo
- [ ] Update documentation

## Notes

**Version**: ___________
**Release Date**: ___________
**Build Machine**: ___________
**Test Machines**: ___________

**Issues Found**:
- 
- 
- 

**Resolved**:
- 
- 
- 
