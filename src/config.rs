use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    #[serde(default)]
    pub notifications: NotificationsConfig,
    #[serde(default)]
    pub system: SystemConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MonitoringConfig {
    pub poll_interval_secs: u64,
    pub port_ranges: Vec<(u16, u16)>,
    pub show_project_names: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct IntegrationsConfig {
    pub brew_enabled: bool,
    pub docker_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct NotificationsConfig {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SystemConfig {
    pub launch_at_login: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 2,
            port_ranges: vec![
                (3000, 3010),   // Node.js, React, Next.js, Vite
                (3306, 3306),   // MySQL
                (4000, 4010),   // Alternative Node servers
                (5001, 5010),   // Flask, general dev servers (excluding 5000)
                (5173, 5173),   // Vite default
                (5432, 5432),   // PostgreSQL
                (6379, 6380),   // Redis (6379 default, 6380 for testing)
                (8000, 8100),   // Django, Python HTTP servers
                (8080, 8090),   // Tomcat, alternative HTTP
                (9000, 9010),   // Various dev tools
                (27017, 27017), // MongoDB
            ],
            show_project_names: true,
        }
    }
}

impl Default for IntegrationsConfig {
    fn default() -> Self {
        Self {
            brew_enabled: true,
            docker_enabled: true,
        }
    }
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            launch_at_login: false,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".portkiller.json")
}

pub fn load_or_create_config() -> Result<Config> {
    let path = get_config_path();

    if path.exists() {
        let content = fs::read_to_string(&path).context("failed to read config file")?;
        serde_json::from_str::<Config>(&content).context("failed to parse config file")
    } else {
        let config = Config::default();
        save_config(&config)?;
        Ok(config)
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path();
    let content = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(&path, content).context("failed to write config file")?;
    Ok(())
}
