//! Windows toast notifications using PowerShell

use std::collections::HashSet;
use std::env;

use crate::utils::hidden_command;

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

fn get_icon_path() -> Option<String> {
    // Try to find assets/app-logo-color.png relative to executable or current dir
    let filename = "app-logo-color.png";
    
    // Priority 1: Relative to executable (portability)
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let path = parent.join("assets").join(filename);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
            // Also check root if we're in /target/debug/
            let path = parent.parent().and_then(|p| p.parent()).map(|p| p.join("assets").join(filename));
             if let Some(path) = path {
                if path.exists() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Priority 2: Current working directory
    if let Ok(cwd) = env::current_dir() {
        let path = cwd.join("assets").join(filename);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    None
}

fn show_toast_powershell(title: &str, body: &str) {
    // Escape single quotes for PowerShell
    let title = title.replace('\'', "''").replace('`', "``");
    let body = body.replace('\'', "''").replace('`', "``");
    
    // Resolve icon path
    let icon_path = get_icon_path().unwrap_or_default();
    let icon_arg = if icon_path.is_empty() { "$null".to_string() } else { format!("'{}'", icon_path) };
    
    // Windows XML needs explicit file:/// URI for images usually, strict path handling
    // However, local paths usually work if absolute.
    // For XML injection:
    let xml_image_node = if icon_path.is_empty() {
        "".to_string()
    } else {
        format!(r#"<image placement="appLogoOverride" src="{}" />"#, icon_path)
    };

    // Use BurntToast module if available, otherwise fall back to basic toast
    // BurntToast provides better toast functionality but isn't guaranteed to be installed
    let script = format!(
        r#"
$ErrorActionPreference = 'SilentlyContinue'

# Try BurntToast first (better UX)
if (Get-Module -ListAvailable -Name BurntToast) {{
    Import-Module BurntToast
    New-BurntToastNotification -Text '{title}', '{body}' -AppLogo {icon_arg}
}} else {{
    # Fallback to Windows Runtime toast
    [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
    [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
    
    $template = @'
<toast>
    <visual>
        <binding template="ToastGeneric">
            <text>{title}</text>
            <text>{body}</text>
            {xml_image_node}
        </binding>
    </visual>

</toast>
'@
    
    $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
    $xml.LoadXml($template)
    $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
    [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('PortKiller.App').Show($toast)
}}
"#,
        title = title,
        body = body,
        icon_arg = icon_arg,
        xml_image_node = xml_image_node
    );
    
    // Run PowerShell in background without waiting (hidden to prevent console flicker)
    let _ = hidden_command("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive", 
            "-WindowStyle", "Hidden",
            "-Command", &script
        ])
        .spawn();
}
