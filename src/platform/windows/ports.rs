//! Windows port scanning implementation using netstat

use std::collections::HashSet;

use crate::utils::hidden_command;

use anyhow::{Context, Result, anyhow};

use crate::model::ProcessInfo;

pub fn scan_ports(port_ranges: &[(u16, u16)]) -> Result<Vec<ProcessInfo>> {
    fn in_ranges(port: u16, ranges: &[(u16, u16)]) -> bool {
        ranges.iter().any(|(s, e)| port >= *s && port <= *e)
    }

    // Run netstat to get listening ports (hidden to prevent console flicker)
    let output = hidden_command("netstat")
        .args(["-ano", "-p", "TCP"])
        .output()
        .context("failed to execute netstat")?;

    if !output.status.success() {
        return Err(anyhow!(
            "netstat failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results: Vec<ProcessInfo> = Vec::new();
    let mut seen: HashSet<(u16, i32)> = HashSet::new();

    for line in stdout.lines() {
        // Parse lines like: TCP    0.0.0.0:3000    0.0.0.0:0    LISTENING    1234
        // or:               TCP    [::]:3000       [::]:0       LISTENING    1234
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        // Need at least: TCP, local_addr, foreign_addr, state, PID
        if parts.len() < 5 {
            continue;
        }
        
        // Check for TCP and LISTENING state
        if parts[0] != "TCP" || parts[3] != "LISTENING" {
            continue;
        }

        // Extract port from local address (e.g., "0.0.0.0:3000" or "[::]:3000")
        let port = match parse_port_from_address(parts[1]) {
            Some(p) => p,
            None => continue,
        };
        
        if !in_ranges(port, port_ranges) {
            continue;
        }

        // Parse PID (last column)
        let pid: i32 = match parts[4].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // Skip PID 0 (System Idle Process)
        if pid == 0 {
            continue;
        }

        if !seen.insert((port, pid)) {
            continue;
        }

        // Get process name from PID
        let command = get_process_name(pid as u32).unwrap_or_else(|| format!("PID {}", pid));

        results.push(ProcessInfo { port, pid, command });
    }

    results.sort();
    Ok(results)
}

/// Parse port from address like "0.0.0.0:3000" or "[::]:3000" or "127.0.0.1:8080"
fn parse_port_from_address(addr: &str) -> Option<u16> {
    // Handle IPv6 format like "[::]:3000" or "[::1]:3000"
    if addr.contains('[') {
        // Find the last ]:port pattern
        if let Some(bracket_pos) = addr.rfind(']') {
            let after_bracket = &addr[bracket_pos + 1..];
            if let Some(port_str) = after_bracket.strip_prefix(':') {
                return port_str.parse().ok();
            }
        }
        return None;
    }
    
    // Handle IPv4 format like "0.0.0.0:3000" or "127.0.0.1:8080"
    addr.rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
}

/// Get process name from PID using Windows API
fn get_process_name(pid: u32) -> Option<String> {
    use windows::Win32::System::ProcessStatus::K32GetModuleBaseNameW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ};
    use windows::Win32::Foundation::CloseHandle;

    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        ).ok()?;
        
        let mut name = [0u16; 260];
        let len = K32GetModuleBaseNameW(handle, None, &mut name);
        let _ = CloseHandle(handle);
        
        if len > 0 {
            Some(String::from_utf16_lossy(&name[..len as usize]))
        } else {
            None
        }
    }
}

/// Verify that a PID is still associated with a TCP listener.
/// Used to mitigate TOCTOU race conditions before killing a process.
pub fn verify_pid_is_listener(pid: i32) -> bool {
    // Re-scan and check if PID is still listening
    if let Ok(output) = hidden_command("netstat")
        .args(["-ano", "-p", "TCP"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().any(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.len() >= 5 
                && parts[0] == "TCP"
                && parts[3] == "LISTENING" 
                && parts[4].parse::<i32>().ok() == Some(pid)
        })
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::parse_port_from_address;

    #[test]
    fn parses_ipv4_any() {
        assert_eq!(parse_port_from_address("0.0.0.0:3000"), Some(3000));
    }

    #[test]
    fn parses_ipv4_localhost() {
        assert_eq!(parse_port_from_address("127.0.0.1:5173"), Some(5173));
    }

    #[test]
    fn parses_ipv6_any() {
        assert_eq!(parse_port_from_address("[::]:8000"), Some(8000));
    }

    #[test]
    fn parses_ipv6_localhost() {
        assert_eq!(parse_port_from_address("[::1]:9000"), Some(9000));
    }
}
