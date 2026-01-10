use crate::resolve::ResolvedDest;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("failed to execute {cmd}: {source}")]
    ExecError { cmd: String, source: std::io::Error },
    #[error("{cmd} failed with exit code {code}: {stderr}")]
    CommandFailed { cmd: String, code: i32, stderr: String },
    #[error("{cmd} was killed by signal")]
    Killed { cmd: String },
    #[error("local file not found: {0}")]
    LocalNotFound(String),
}

/// Ensure remote directory exists
pub fn ensure_remote_dir(dest: &ResolvedDest, path: &str, verbose: bool) -> Result<(), TransferError> {
    let dir = if path.ends_with('/') {
        path.to_string()
    } else {
        Path::new(path)
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "/".to_string())
    };

    let mut cmd = Command::new("ssh");
    cmd.args(dest.ssh_args());
    cmd.arg(dest.ssh_target());
    cmd.arg(format!("mkdir -p '{}'", dir));

    if verbose {
        eprintln!("+ ssh {} mkdir -p '{}'", dest.ssh_target(), dir);
    }

    let output = cmd
        .output()
        .map_err(|e| TransferError::ExecError {
            cmd: "ssh".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(TransferError::CommandFailed {
            cmd: "ssh mkdir".to_string(),
            code,
            stderr,
        });
    }

    Ok(())
}

/// Check if remote file exists
pub fn remote_file_exists(dest: &ResolvedDest, path: &str, verbose: bool) -> Result<bool, TransferError> {
    let mut cmd = Command::new("ssh");
    cmd.args(dest.ssh_args());
    cmd.arg(dest.ssh_target());
    cmd.arg(format!("test -e '{}'", path));

    if verbose {
        eprintln!("+ ssh {} test -e '{}'", dest.ssh_target(), path);
    }

    let status = cmd
        .status()
        .map_err(|e| TransferError::ExecError {
            cmd: "ssh".to_string(),
            source: e,
        })?;

    Ok(status.success())
}

/// Copy a file via scp
pub fn scp_file(
    local_path: &Path,
    dest: &ResolvedDest,
    remote_path: &str,
    verbose: bool,
) -> Result<(), TransferError> {
    if !local_path.exists() {
        return Err(TransferError::LocalNotFound(local_path.display().to_string()));
    }

    let full_target = format!("{}@{}:{}", dest.user, dest.host, remote_path);

    let mut cmd = Command::new("scp");
    cmd.args(dest.scp_args());
    cmd.arg(local_path);
    cmd.arg(&full_target);

    if verbose {
        eprintln!("+ scp {} {}", local_path.display(), full_target);
    }

    // Run with inherited stdout/stderr for progress display
    let status = cmd
        .status()
        .map_err(|e| TransferError::ExecError {
            cmd: "scp".to_string(),
            source: e,
        })?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(TransferError::CommandFailed {
            cmd: "scp".to_string(),
            code,
            stderr: "transfer failed".to_string(),
        });
    }

    Ok(())
}

/// Test SSH connectivity to a host
pub fn test_connectivity(dest: &ResolvedDest, verbose: bool) -> Result<(), TransferError> {
    let mut cmd = Command::new("ssh");
    cmd.args(dest.ssh_args());
    cmd.args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"]);
    cmd.arg(dest.ssh_target());
    cmd.arg("echo ok");

    if verbose {
        eprintln!("+ ssh -o BatchMode=yes {} echo ok", dest.ssh_target());
    }

    let output = cmd
        .output()
        .map_err(|e| TransferError::ExecError {
            cmd: "ssh".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(TransferError::CommandFailed {
            cmd: format!("ssh to {}", dest.host),
            code,
            stderr,
        });
    }

    Ok(())
}

/// Extract a zip file on the remote
pub fn remote_unzip(dest: &ResolvedDest, zip_path: &str, verbose: bool) -> Result<String, TransferError> {
    let extract_dir = zip_path.trim_end_matches(".zip");

    let mut cmd = Command::new("ssh");
    cmd.args(dest.ssh_args());
    cmd.arg(dest.ssh_target());
    cmd.arg(format!("unzip -o '{}' -d '{}'", zip_path, extract_dir));

    if verbose {
        eprintln!("+ ssh {} unzip -o '{}' -d '{}'", dest.ssh_target(), zip_path, extract_dir);
    }

    let output = cmd
        .output()
        .map_err(|e| TransferError::ExecError {
            cmd: "ssh unzip".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(TransferError::CommandFailed {
            cmd: "ssh unzip".to_string(),
            code,
            stderr,
        });
    }

    Ok(extract_dir.to_string())
}

/// Create a zip archive on the remote
pub fn remote_zip(
    dest: &ResolvedDest,
    dir_path: &str,
    excludes: &[String],
    staging_dir: &str,
    verbose: bool,
) -> Result<String, TransferError> {
    // Generate archive name
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dir_name = Path::new(dir_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");
    let archive_name = format!("{}-{}.zip", dir_name, timestamp);
    let archive_path = format!("{}/{}", staging_dir, archive_name);

    // Build exclude args
    let exclude_args: Vec<String> = excludes
        .iter()
        .flat_map(|e| vec!["-x".to_string(), format!("'{}'", e)])
        .collect();

    // Create zip on remote
    let zip_cmd = format!(
        "cd '{}' && zip -r '{}' . {}",
        dir_path,
        archive_path,
        exclude_args.join(" ")
    );

    let mut cmd = Command::new("ssh");
    cmd.args(dest.ssh_args());
    cmd.arg(dest.ssh_target());
    cmd.arg(&zip_cmd);

    if verbose {
        eprintln!("+ ssh {} {}", dest.ssh_target(), zip_cmd);
    }

    let output = cmd
        .output()
        .map_err(|e| TransferError::ExecError {
            cmd: "ssh zip".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(TransferError::CommandFailed {
            cmd: "ssh zip".to_string(),
            code,
            stderr,
        });
    }

    Ok(archive_path)
}

/// Copy a file from remote via scp
pub fn scp_from_remote(
    dest: &ResolvedDest,
    remote_path: &str,
    local_path: &Path,
    verbose: bool,
) -> Result<(), TransferError> {
    let full_source = format!("{}@{}:{}", dest.user, dest.host, remote_path);

    let mut cmd = Command::new("scp");
    cmd.args(dest.scp_args());
    cmd.arg(&full_source);
    cmd.arg(local_path);

    if verbose {
        eprintln!("+ scp {} {}", full_source, local_path.display());
    }

    let status = cmd
        .status()
        .map_err(|e| TransferError::ExecError {
            cmd: "scp".to_string(),
            source: e,
        })?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(TransferError::CommandFailed {
            cmd: "scp".to_string(),
            code,
            stderr: "transfer failed".to_string(),
        });
    }

    Ok(())
}

/// Remove a file on the remote
pub fn remote_rm(dest: &ResolvedDest, path: &str, verbose: bool) -> Result<(), TransferError> {
    let mut cmd = Command::new("ssh");
    cmd.args(dest.ssh_args());
    cmd.arg(dest.ssh_target());
    cmd.arg(format!("rm -f '{}'", path));

    if verbose {
        eprintln!("+ ssh {} rm -f '{}'", dest.ssh_target(), path);
    }

    let output = cmd
        .output()
        .map_err(|e| TransferError::ExecError {
            cmd: "ssh rm".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(TransferError::CommandFailed {
            cmd: "ssh rm".to_string(),
            code,
            stderr,
        });
    }

    Ok(())
}
