//! Windows Services integration for common dev services
//!
//! Replaces Homebrew services on Windows with detection of common
//! Windows services like PostgreSQL, MySQL, SQL Server, Redis, etc.

use std::collections::HashMap;

use crate::utils::hidden_command;

use crate::model::KillFeedback;

/// Query Windows services that commonly use dev ports
pub fn query_windows_services_map() -> anyhow::Result<HashMap<String, String>> {
    let mut map = HashMap::new();

    // Check common dev services
    let services = [
        // PostgreSQL
        "postgresql-x64-16",
        "postgresql-x64-15",
        "postgresql-x64-14",
        "postgresql-x64-13",
        "postgresql",
        // MySQL
        "MySQL80",
        "MySQL57",
        "MySQL",
        // SQL Server
        "MSSQLSERVER",
        "MSSQL$SQLEXPRESS",
        "SQLAgent$SQLEXPRESS",
        // Redis
        "Redis",
        // MongoDB
        "MongoDB",
    ];

    for service in services {
        if let Some(status) = get_service_status(service) {
            map.insert(service.to_string(), status);
        }
    }

    Ok(map)
}

fn get_service_status(service: &str) -> Option<String> {
    let output = hidden_command("sc")
        .args(["query", service])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("RUNNING") {
        Some("running".to_string())
    } else if stdout.contains("STOPPED") {
        Some("stopped".to_string())
    } else if stdout.contains("PENDING") {
        Some("pending".to_string())
    } else {
        None
    }
}

/// Check if a process is managed by a Windows service
pub fn get_windows_managed_service(
    cmd: &str,
    port: u16,
    services_map: &HashMap<String, String>,
) -> Option<String> {
    let lc = cmd.to_lowercase();

    let potential_service = if lc.contains("postgres") {
        find_running_service(services_map, &["postgresql"])
    } else if lc.contains("mysqld") || lc.contains("mysql") {
        find_running_service(services_map, &["mysql"])
    } else if lc.contains("sqlservr") {
        find_running_service(services_map, &["mssqlserver", "mssql$"])
    } else if lc.contains("redis-server") || lc.contains("redis") {
        find_running_service(services_map, &["redis"])
    } else if lc.contains("mongod") {
        find_running_service(services_map, &["mongodb"])
    } else {
        None
    };

    potential_service.and_then(|service| {
        let expected_port = get_default_port_for_service(&service);
        if Some(port) == expected_port {
            Some(service)
        } else {
            None
        }
    })
}

fn find_running_service(map: &HashMap<String, String>, prefixes: &[&str]) -> Option<String> {
    for (name, status) in map {
        if status == "running" {
            let name_lower = name.to_lowercase();
            for prefix in prefixes {
                if name_lower.contains(prefix) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

fn get_default_port_for_service(service: &str) -> Option<u16> {
    let lc = service.to_lowercase();
    if lc.contains("postgres") {
        Some(5432)
    } else if lc.contains("mysql") {
        Some(3306)
    } else if lc.contains("mssql") {
        Some(1433)
    } else if lc.contains("redis") {
        Some(6379)
    } else if lc.contains("mongodb") {
        Some(27017)
    } else {
        None
    }
}

/// Stop a Windows service
pub fn run_service_stop(service: &str) -> KillFeedback {
    let result = hidden_command("sc").args(["stop", service]).output();

    match result {
        Ok(out) if out.status.success() => {
            KillFeedback::info(format!("Stopped service {}.", service))
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let output = if stderr.is_empty() { stdout } else { stderr };

            if output.contains("Access is denied") || output.contains("5)") {
                KillFeedback::error(format!(
                    "Access denied stopping {}. Run as Administrator.",
                    service
                ))
            } else if output.contains("not started") || output.contains("1062)") {
                KillFeedback::warning(format!("Service {} is not running.", service))
            } else {
                KillFeedback::error(format!("Failed to stop {}: {}", service, output.trim()))
            }
        }
        Err(e) => KillFeedback::error(format!("Service control error: {}", e)),
    }
}

/// Get a friendly display name for a Windows service
pub fn friendly_service_name(service: &str) -> String {
    let lc = service.to_lowercase();

    if lc.contains("postgres") {
        "PostgreSQL".to_string()
    } else if lc.contains("mysql") {
        "MySQL".to_string()
    } else if lc.contains("mssql") {
        if lc.contains("express") {
            "SQL Server Express".to_string()
        } else {
            "SQL Server".to_string()
        }
    } else if lc.contains("redis") {
        "Redis".to_string()
    } else if lc.contains("mongodb") {
        "MongoDB".to_string()
    } else {
        service.to_string()
    }
}
