use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config file not found (searched: {0})")]
    NotFound(String),
    #[error("failed to read config: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    ParseError(#[from] serde_yaml::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub version: u32,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub hosts: HashMap<String, Host>,
    #[serde(default)]
    pub shares: Shares,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Defaults {
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_path_alias")]
    pub default_path_alias: String,
    #[serde(default)]
    pub zip: ZipDefaults,
    #[serde(default = "default_staging_dir")]
    pub staging_dir: String,
}

fn default_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "chris".to_string())
}

fn default_port() -> u16 {
    22
}

fn default_path_alias() -> String {
    "scratch".to_string()
}

fn default_staging_dir() -> String {
    "/tmp".to_string()
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ZipDefaults {
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub follow_symlinks: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Host {
    pub host: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub identity_file: Option<PathBuf>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub paths: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Shares {
    #[serde(default = "default_share")]
    pub default: String,
    #[serde(default = "default_layout")]
    pub layout: String,
}

fn default_share() -> String {
    "ganymede:dumps".to_string()
}

fn default_layout() -> String {
    "{source}/{date}".to_string()
}

impl Config {
    pub fn load(explicit_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let paths = Self::config_paths(explicit_path);

        for path in &paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let config: Config = serde_yaml::from_str(&content)?;
                return Ok(config);
            }
        }

        Err(ConfigError::NotFound(
            paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ))
    }

    fn config_paths(explicit: Option<PathBuf>) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(p) = explicit {
            paths.push(p);
        }

        // Project-local config
        paths.push(PathBuf::from(".quick-copy.yaml"));

        // XDG config
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("quick-copy").join("config.yaml"));
        }

        // Home fallback
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".quick-copy.yaml"));
        }

        paths
    }

    pub fn get_host(&self, name: &str) -> Option<&Host> {
        self.hosts.get(name)
    }

    pub fn host_names(&self) -> Vec<&String> {
        self.hosts.keys().collect()
    }

    /// Find similar host names for typo suggestions
    pub fn find_similar_host(&self, name: &str) -> Option<&String> {
        let name_lower = name.to_lowercase();
        self.hosts
            .keys()
            .find(|h| h.to_lowercase().starts_with(&name_lower) || name_lower.starts_with(&h.to_lowercase()))
    }
}
