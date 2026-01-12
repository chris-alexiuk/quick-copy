# Roadmap

Development plan for quick-copy.

## Current Version: 0.1.0

Core functionality:
- Single file transfers (`file`)
- Directory zipping and transfer (`dir`)
- Remote directory retrieval (`pull`)
- Shared storage dumps (`dump`)
- Host/alias listing (`ls`)
- Configuration validation (`doctor`)

## Version 0.2.0 - Polish & Reliability

Focus: Improve user experience and handle edge cases.

- [ ] Progress indicators for large transfers
- [ ] Better error messages with actionable suggestions
- [ ] Overwrite confirmation for existing files (unless `--overwrite`)
- [ ] Handle SSH key authentication failures gracefully
- [ ] Validate remote paths before transfer
- [x] Add `--dry-run` flag to preview operations
- [ ] Cleanup temporary archives on failure
- [ ] Transfer size estimation before upload

## Version 0.3.0 - Transfer Management

Focus: Make transfers more robust and controllable.

- [ ] Resume interrupted transfers (using rsync fallback)
- [ ] Parallel file transfers (multiple files in `dir`)
- [ ] Bandwidth limiting (`--limit` flag)
- [ ] Compression level control (`--compression 1-9`)
- [ ] Cancel transfers gracefully (SIGINT handling)
- [ ] Transfer history log (`~/.local/share/quick-copy/history.jsonl`)
- [ ] Retry failed transfers automatically

## Version 0.4.0 - Sync & Mirroring

Focus: Bidirectional sync and advanced transfer modes.

- [ ] Sync command for bidirectional updates
  - `qc sync server:projects` - sync local dir with remote
  - Detect changed/new/deleted files
  - Conflict resolution strategies
- [ ] Mirror mode (one-way sync with deletes)
- [ ] Incremental backups
- [ ] `.qcignore` file support (like `.gitignore`)
- [ ] Checksum verification (MD5/SHA256)

## Version 0.5.0 - Remote Operations

Focus: Work with remote files without local copies.

- [ ] Remote-to-remote transfers (no local hop)
  - `qc copy server1:data server2:backup`
- [ ] Remote file listing
  - `qc ls server:scratch --remote`
- [ ] Remote file operations
  - `qc rm server:scratch/old-data.zip`
  - `qc mv server:scratch/file.txt server:archive/`
- [ ] Disk usage on remote paths
  - `qc du server:scratch`

## Version 0.6.0 - Monitoring & Observability

Focus: Integration with homelab monitoring infrastructure.

- [ ] Prometheus metrics endpoint
  - Transfer counts, sizes, duration
  - Success/failure rates
  - Active transfers gauge
- [ ] Structured logging (JSON output mode)
- [ ] Transfer notifications (webhook support)
- [ ] Grafana dashboard template
- [ ] Integration with Mercury monitoring stack

## Version 1.0.0 - Production Ready

Focus: Stability, documentation, and ecosystem.

- [ ] Shell completion (bash, zsh, fish)
- [ ] Man pages
- [ ] Comprehensive test suite
- [ ] CI/CD pipeline
- [ ] Binary releases for Linux/macOS
- [ ] Migration guide from scp/rsync
- [ ] Performance benchmarks
- [ ] Security audit

## Future Considerations

Ideas for post-1.0 development (no timeline):

### Advanced Features
- Watch mode for automatic syncing on file changes
- Transfer queuing and scheduling
- Multiple archive formats (tar.gz, tar.zst, etc.)
- Encryption at rest (GPG integration)

### Platform Support
- Windows support (via WSL or native)
- Container image for isolated usage
- Plugin system for custom transfer handlers

### Integration
- Git integration for project-aware transfers
- Docker volume synchronization
- Cloud storage backends (S3, B2)
- Tailscale mesh VPN integration
- K8s pod file transfers

## Non-Goals

Features explicitly out of scope:

- GUI application (CLI-first tool)
- Built-in file browser/editor
- Version control (use git)
- File sharing server
- Real-time collaboration features
- Chat/messaging features

## Contributing

Roadmap items are prioritized but not set in stone. Contributions are welcome. See issues tagged with roadmap version labels.
