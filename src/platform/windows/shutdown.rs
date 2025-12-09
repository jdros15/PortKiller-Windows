//! Windows shutdown detection and error suppression
//!
//! Handles two aspects of graceful shutdown:
//! 1. Suppresses error dialogs for child processes (netstat, etc.) that may fail during shutdown
//! 2. Provides a shutdown flag for background threads to check

use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::System::Diagnostics::Debug::{
    SetErrorMode, SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SEM_NOOPENFILEERRORBOX,
};

/// Global flag indicating the system is shutting down.
/// Set to `true` when we receive a shutdown/logoff/close signal.
static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Check if the application is in shutdown mode.
/// Background threads should check this before executing external commands.
pub fn is_shutting_down() -> bool {
    SHUTTING_DOWN.load(Ordering::SeqCst)
}

/// Mark the application as shutting down.
/// This should be called when the event loop is exiting.
pub fn set_shutting_down() {
    SHUTTING_DOWN.store(true, Ordering::SeqCst);
}

/// Initialize error suppression for child processes.
/// 
/// This sets the error mode so that child processes (like netstat.exe) will not
/// display error dialogs when they fail. Instead, errors are returned to the
/// calling process silently. This is crucial during Windows shutdown when
/// system components may already be unloaded, causing child processes to fail
/// with error 0xc0000142.
///
/// Child processes inherit the error mode from the parent process, so this
/// only needs to be called once at startup.
pub fn init_shutdown_handler() {
    unsafe {
        // Set error mode to suppress error dialogs for this process and child processes
        // SEM_FAILCRITICALERRORS: Don't show critical error dialogs
        // SEM_NOGPFAULTERRORBOX: Don't show Windows Error Reporting dialogs  
        // SEM_NOOPENFILEERRORBOX: Don't show file not found dialogs
        let error_mode = SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX;
        let _previous = SetErrorMode(error_mode);
        log::debug!(
            "Set error mode to suppress child process error dialogs (flags: {:?})",
            error_mode
        );
    }
}

