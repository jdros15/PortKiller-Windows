//! Windows process termination using TerminateProcess API

use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};

use crate::model::KillOutcome;
use crate::platform::windows::ports::verify_pid_is_listener;

const GRACEFUL_TIMEOUT: Duration = Duration::from_secs(2);
const FORCE_TIMEOUT: Duration = Duration::from_secs(1);
const POLL_STEP: Duration = Duration::from_millis(200);

pub fn terminate_pid(pid: i32) -> KillOutcome {
    // TOCTOU mitigation: verify PID is still a TCP listener before killing
    if !verify_pid_is_listener(pid) {
        log::warn!(
            "PID {} is no longer a TCP listener, skipping kill to avoid TOCTOU race",
            pid
        );
        return KillOutcome::AlreadyExited;
    }

    unsafe {
        // Open process with terminate rights
        let handle = match OpenProcess(
            PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid as u32,
        ) {
            Ok(h) => h,
            Err(e) => {
                let code = e.code().0 as u32;
                // ERROR_INVALID_PARAMETER (87) or ERROR_NOT_FOUND (1168) = process doesn't exist
                if code == 87 || code == 1168 {
                    return KillOutcome::AlreadyExited;
                }
                // ERROR_ACCESS_DENIED (5)
                if code == 5 {
                    return KillOutcome::PermissionDenied;
                }
                log::error!("OpenProcess failed: {:?}", e);
                return KillOutcome::Failed(code as i32);
            }
        };

        // Try to close gracefully first by waiting a bit
        // Console apps don't have message queues, so we just wait briefly
        // This gives apps a chance to handle their cleanup if they're monitoring for termination
        if wait_for_exit(handle, GRACEFUL_TIMEOUT) {
            let _ = CloseHandle(handle);
            return KillOutcome::Success;
        }

        // Force terminate
        match TerminateProcess(handle, 1) {
            Ok(()) => {
                if wait_for_exit(handle, FORCE_TIMEOUT) {
                    let _ = CloseHandle(handle);
                    return KillOutcome::Success;
                }
                let _ = CloseHandle(handle);
                KillOutcome::TimedOut
            }
            Err(e) => {
                let _ = CloseHandle(handle);
                let code = e.code().0 as u32;
                if code == 5 {
                    return KillOutcome::PermissionDenied;
                }
                // Process may have exited between open and terminate
                if code == 87 || code == 1168 {
                    return KillOutcome::AlreadyExited;
                }
                KillOutcome::Failed(code as i32)
            }
        }
    }
}

/// Wait for process to exit
unsafe fn wait_for_exit(handle: HANDLE, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        // WaitForSingleObject with 0 timeout = check status without waiting
        // SAFETY: handle is valid and obtained from OpenProcess
        let result = unsafe { WaitForSingleObject(handle, 0) };
        match result {
            WAIT_OBJECT_0 => return true, // Process exited
            _ => {}
        }
        
        if std::time::Instant::now() >= deadline {
            return false;
        }
        thread::sleep(POLL_STEP);
    }
}
