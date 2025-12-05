//! Platform abstraction layer
//!
//! This module provides platform-specific implementations for:
//! - Port scanning
//! - Process termination
//! - Desktop notifications
//! - Launch-at-login

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

// Re-export the current platform's modules
#[cfg(target_os = "macos")]
pub use macos as current;

#[cfg(target_os = "windows")]
pub use windows as current;
