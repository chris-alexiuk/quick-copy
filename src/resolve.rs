use crate::config::{Config, Host};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("unknown host '{name}'{}", suggestion.as_ref().map(|s| format!(" (did you mean '{}'?)", s)).unwrap_or_default())]
    UnknownHost { name: String, suggestion: Option<String> },
    #[error("host '{host}' has no path alias '{alias}'. Run: qc ls")]
    UnknownAlias { host: String, alias: String },
}

/// Resolved destination ready for SSH/SCP
#[derive(Debug, Clone)]
pub struct ResolvedDest {
    pub user: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub identity_file: Option<std::path::PathBuf>,
}

impl ResolvedDest {
    /// Format as user@host:/path for scp
    pub fn scp_target(&self) -> String {
        format!("{}@{}:{}", self.user, self.host, self.path)
    }

    /// Format as user@host for ssh
    pub fn ssh_target(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }

    /// Get SSH args for port and identity
    pub fn ssh_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.port != 22 {
            args.push("-p".to_string());
            args.push(self.port.to_string());
        }
        if let Some(ref id) = self.identity_file {
            args.push("-i".to_string());
            args.push(id.display().to_string());
        }
        args
    }

    /// Get SCP args for port and identity
    pub fn scp_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.port != 22 {
            args.push("-P".to_string());
            args.push(self.port.to_string());
        }
        if let Some(ref id) = self.identity_file {
            args.push("-i".to_string());
            args.push(id.display().to_string());
        }
        args
    }
}

/// Parse destination string: "host", "host:alias", or "host:/absolute/path"
pub fn parse_destination(dest: &str) -> (String, Option<String>) {
    if let Some(idx) = dest.find(':') {
        let host = dest[..idx].to_string();
        let path_or_alias = dest[idx + 1..].to_string();
        (host, Some(path_or_alias))
    } else {
        (dest.to_string(), None)
    }
}

/// Resolve a destination string to full connection details
pub fn resolve(dest: &str, config: &Config) -> Result<ResolvedDest, ResolveError> {
    let (host_name, path_or_alias) = parse_destination(dest);

    let host = config.get_host(&host_name).ok_or_else(|| ResolveError::UnknownHost {
        name: host_name.clone(),
        suggestion: config.find_similar_host(&host_name).cloned(),
    })?;

    let path = resolve_path(host, path_or_alias.as_deref(), config)?;

    Ok(ResolvedDest {
        user: host.user.clone().unwrap_or_else(|| config.defaults.user.clone()),
        host: host.host.clone(),
        port: host.port.unwrap_or(config.defaults.port),
        path,
        identity_file: host.identity_file.clone(),
    })
}

fn resolve_path(host: &Host, path_or_alias: Option<&str>, config: &Config) -> Result<String, ResolveError> {
    match path_or_alias {
        // Explicit absolute path
        Some(p) if p.starts_with('/') => Ok(p.to_string()),
        // Named alias
        Some(alias) => {
            host.paths.get(alias).cloned().ok_or_else(|| ResolveError::UnknownAlias {
                host: host.host.clone(),
                alias: alias.to_string(),
            })
        }
        // Default alias
        None => {
            let default_alias = &config.defaults.default_path_alias;
            host.paths.get(default_alias).cloned().ok_or_else(|| ResolveError::UnknownAlias {
                host: host.host.clone(),
                alias: default_alias.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_destination() {
        assert_eq!(parse_destination("andromeda"), ("andromeda".to_string(), None));
        assert_eq!(
            parse_destination("andromeda:scratch"),
            ("andromeda".to_string(), Some("scratch".to_string()))
        );
        assert_eq!(
            parse_destination("andromeda:/home/chris"),
            ("andromeda".to_string(), Some("/home/chris".to_string()))
        );
    }
}
