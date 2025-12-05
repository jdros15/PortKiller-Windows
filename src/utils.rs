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

