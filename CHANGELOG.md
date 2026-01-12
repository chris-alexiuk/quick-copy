# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-12

### Added
- `--dry-run` flag to preview operations without executing them
- Dry-run support for all transfer commands (file, dir, pull, dump)
- ROADMAP.md documenting planned features and development direction

### Changed
- All command functions now accept `dry_run` parameter

## [0.1.0] - 2026-01-12

### Added
- Initial release
- Copy single files to remote destinations (`file` command)
- Zip and copy directories (`dir` command)
- Pull remote directories to local machine (`pull` command)
- Dump files to shared storage with layout organization (`dump` command)
- List configured hosts and path aliases (`ls` command)
- Configuration validation and connectivity testing (`doctor` command)
- YAML-based configuration with host aliases and path shortcuts
- Support for custom SSH ports and identity files
- Exclude patterns for directory archiving
- JSON output mode for scripting
- Verbose mode showing SSH/SCP commands

[0.2.0]: https://github.com/yourusername/quick-copy/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/quick-copy/releases/tag/v0.1.0
