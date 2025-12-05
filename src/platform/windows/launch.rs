//! Windows launch-at-login using Registry

use anyhow::Result;
use winreg::enums::*;
use winreg::RegKey;

const APP_NAME: &str = "PortKiller";
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Enables launch-at-login by adding to registry Run key
pub fn enable_launch_at_login() -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let path_str = exe_path.to_string_lossy().to_string();
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(RUN_KEY)?;
    key.set_value(APP_NAME, &path_str)?;
    
    log::info!("Enabled launch-at-login via registry: {}", path_str);
    Ok(())
}

/// Disables launch-at-login by removing from registry Run key
pub fn disable_launch_at_login() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    match hkcu.open_subkey_with_flags(RUN_KEY, KEY_WRITE) {
        Ok(key) => {
            // Ignore error if value doesn't exist
            let _ = key.delete_value(APP_NAME);
            log::info!("Disabled launch-at-login");
            Ok(())
        }
        Err(e) => {
            // Key doesn't exist = already disabled
            log::debug!("Registry key not found (already disabled): {}", e);
            Ok(())
        }
    }
}

/// Checks if launch-at-login is currently enabled
pub fn is_launch_at_login_enabled() -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    let key = match hkcu.open_subkey(RUN_KEY) {
        Ok(k) => k,
        Err(_) => return Ok(false),
    };
    
    match key.get_value::<String, _>(APP_NAME) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
