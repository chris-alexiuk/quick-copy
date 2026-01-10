use chrono::Local;
use glob::Pattern;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("failed to create archive: {0}")]
    IoError(#[from] std::io::Error),
    #[error("zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("directory not found: {0}")]
    DirNotFound(String),
    #[error("failed to get current directory")]
    NoCwd,
}

/// Create a zip archive of a directory
pub fn create_archive(
    source_dir: &Path,
    excludes: &[String],
    staging_dir: &str,
    custom_name: Option<&str>,
) -> Result<PathBuf, ArchiveError> {
    if !source_dir.is_dir() {
        return Err(ArchiveError::DirNotFound(source_dir.display().to_string()));
    }

    let dir_name = source_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");

    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let archive_name = custom_name
        .map(|n| n.to_string())
        .unwrap_or_else(|| format!("{}-{}.zip", dir_name, timestamp));

    let archive_path = PathBuf::from(staging_dir).join(&archive_name);
    let file = File::create(&archive_path)?;
    let mut zip = ZipWriter::new(file);

    let exclude_patterns: Vec<Pattern> = excludes
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect();

    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for entry in WalkDir::new(source_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .unwrap_or(path);

        // Skip if matches any exclude pattern
        let relative_str = relative.to_string_lossy();
        if exclude_patterns.iter().any(|p| p.matches(&relative_str)) {
            continue;
        }

        // Also check path components for directory excludes
        let should_skip = relative.components().any(|c| {
            let comp = c.as_os_str().to_string_lossy();
            exclude_patterns.iter().any(|p| {
                let pattern_str = p.as_str();
                // Handle directory patterns like ".git/" or "node_modules/"
                if pattern_str.ends_with('/') {
                    comp == pattern_str.trim_end_matches('/')
                } else {
                    p.matches(&comp)
                }
            })
        });

        if should_skip {
            continue;
        }

        if path.is_file() {
            zip.start_file(relative.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if path.is_dir() && !relative.as_os_str().is_empty() {
            zip.add_directory(relative.to_string_lossy(), options)?;
        }
    }

    zip.finish()?;
    Ok(archive_path)
}

/// Get current working directory name
pub fn cwd_name() -> Result<String, ArchiveError> {
    std::env::current_dir()
        .map_err(|_| ArchiveError::NoCwd)?
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or(ArchiveError::NoCwd)
}

/// Generate timestamp for archive naming
pub fn timestamp() -> String {
    Local::now().format("%Y%m%d-%H%M%S").to_string()
}

/// Generate date string for dump layout
pub fn date_string() -> String {
    Local::now().format("%Y%m%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_format() {
        let ts = timestamp();
        assert!(ts.len() == 15); // YYYYMMDD-HHMMSS
        assert!(ts.contains('-'));
    }
}
