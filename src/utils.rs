/// Find an executable in common locations, falling back to PATH.
/// Results are cached for efficiency.
#[cfg(target_os = "macos")]
use std::path::Path;
#[cfg(target_os = "macos")]
use std::sync::OnceLock;

#[cfg(target_os = "macos")]
pub fn find_command(name: &str) -> &'static str {
    // Use static caches for common commands
    match name {
        "docker" => {
            static DOCKER: OnceLock<&'static str> = OnceLock::new();
            DOCKER.get_or_init(|| find_in_paths(name, HOMEBREW_PATHS))
        }
        "brew" => {
            static BREW: OnceLock<&'static str> = OnceLock::new();
            BREW.get_or_init(|| find_in_paths(name, HOMEBREW_PATHS))
        }
        "terminal-notifier" => {
            static NOTIFIER: OnceLock<&'static str> = OnceLock::new();
            NOTIFIER.get_or_init(|| find_in_paths(name, HOMEBREW_PATHS))
        }
        _ => find_in_paths(name, HOMEBREW_PATHS),
    }
}

#[cfg(target_os = "macos")]
const HOMEBREW_PATHS: &[&str] = &[
    "/opt/homebrew/bin", // Apple Silicon
    "/usr/local/bin",    // Intel Mac
];

#[cfg(target_os = "macos")]
fn find_in_paths(name: &str, prefix_paths: &[&str]) -> &'static str {
    for prefix in prefix_paths {
        let full_path = format!("{}/{}", prefix, name);
        if Path::new(&full_path).exists() {
            // Leak the string to get a 'static lifetime (acceptable for small, cached strings)
            return Box::leak(full_path.into_boxed_str());
        }
    }
    // Fallback to PATH lookup
    Box::leak(name.to_string().into_boxed_str())
}

/// Find an executable on Windows - just returns the name for PATH lookup
#[cfg(target_os = "windows")]
pub fn find_command(name: &str) -> &'static str {
    // On Windows, commands like docker and sc are typically in PATH
    // Just return the name and let the OS handle path resolution
    match name {
        "docker" => "docker",
        _ => Box::leak(name.to_string().into_boxed_str()),
    }
}

/// Create a Command that runs hidden on Windows (no console window).
/// This prevents the brief console window flicker when spawning processes.
#[cfg(target_os = "windows")]
pub fn hidden_command(program: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// On non-Windows, just return a normal Command
#[cfg(not(target_os = "windows"))]
pub fn hidden_command(program: &str) -> std::process::Command {
    std::process::Command::new(program)
}

/// Check if Windows is using dark mode for apps
/// Returns true if dark mode is enabled, false for light mode
#[cfg(target_os = "windows")]
pub fn is_windows_dark_mode() -> bool {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(personalize) =
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
    {
        // AppsUseLightTheme: 0 = dark mode, 1 = light mode
        if let Ok(value) = personalize.get_value::<u32, _>("AppsUseLightTheme") {
            return value == 0;
        }
    }
    // Default to light mode if we can't read the registry
    false
}
