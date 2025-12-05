//! Windows toast notifications using PowerShell

use std::collections::HashSet;
use std::process::Command;

use crate::model::{AppState, ProcessInfo};

pub fn maybe_notify_changes(state: &AppState, prev: &[ProcessInfo]) {
    if !state.config.notifications.enabled {
        return;
    }

    let prev_ports: HashSet<u16> = prev.iter().map(|p| p.port).collect();
    let curr_ports: HashSet<u16> = state.processes.iter().map(|p| p.port).collect();

    // Notify for added ports
    let added: Vec<u16> = curr_ports.difference(&prev_ports).copied().collect();
    for port in added {
        if let Some(process) = state.processes.iter().find(|p| p.port == port) {
            let title = format!("Port {} Started", port);
            let body = format_body(process, state);
            show_toast(&title, &body);
        }
    }

    // Notify for removed ports
    let removed: Vec<u16> = prev_ports.difference(&curr_ports).copied().collect();
    for port in removed {
        if let Some(process) = prev.iter().find(|p| p.port == port) {
            let title = format!("Port {} Stopped", port);
            let body = format_body(process, state);
            show_toast(&title, &body);
        }
    }
}

fn format_body(process: &ProcessInfo, state: &AppState) -> String {
    let command = truncate(&process.command, 40);
    if let Some(project) = state.project_cache.get(&process.pid) {
        format!("{} ({}) • {}", command, process.pid, project.name)
    } else {
        format!("{} ({})", command, process.pid)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn show_toast(title: &str, body: &str) {
    // Use PowerShell to show Windows toast notification
    // This works without requiring app identity/manifest
    show_toast_powershell(title, body);
}

fn show_toast_powershell(title: &str, body: &str) {
    // Escape single quotes for PowerShell
    let title = title.replace('\'', "''").replace('`', "``");
    let body = body.replace('\'', "''").replace('`', "``");
    
    // Use BurntToast module if available, otherwise fall back to basic toast
    // BurntToast provides better toast functionality but isn't guaranteed to be installed
    let script = format!(
        r#"
$ErrorActionPreference = 'SilentlyContinue'

# Try BurntToast first (better UX)
if (Get-Module -ListAvailable -Name BurntToast) {{
    Import-Module BurntToast
    New-BurntToastNotification -Text '{title}', '{body}' -AppLogo $null
}} else {{
    # Fallback to Windows Runtime toast
    [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
    [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
    
    $template = @'
<toast>
    <visual>
        <binding template="ToastText02">
            <text id="1">{title}</text>
            <text id="2">{body}</text>
        </binding>
    </visual>
    <audio silent="true"/>
</toast>
'@
    
    $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
    $xml.LoadXml($template)
    $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
    [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('PortKiller').Show($toast)
}}
"#,
        title = title,
        body = body
    );
    
    // Run PowerShell in background without waiting
    let _ = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive", 
            "-WindowStyle", "Hidden",
            "-Command", &script
        ])
        .spawn();
}
