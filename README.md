# quick-copy

Fast SSH/SCP file transfer tool with named targets and path aliases.

**[Documentation](https://chris-alexiuk.github.io/quick-copy/)** | [GitHub](https://github.com/chris-alexiuk/quick-copy) | [crates.io](https://crates.io/crates/quick-copy)

## Installation

```bash
cargo install quick-copy
```

This installs both `quick-copy` and `qc` (short alias).

## Quick Start

1. Create config at `~/.config/quick-copy/config.yaml`:

```yaml
version: 1

defaults:
  user: myuser
  port: 22
  default_path_alias: scratch
  staging_dir: "/tmp"
  zip:
    exclude:
      - ".git/"
      - "node_modules/"

hosts:
  server:
    host: "192.168.1.100"
    paths:
      scratch: "/home/myuser/scratch"
      projects: "/home/myuser/projects"

  nas:
    host: "nas.local"
    paths:
      shared: "/mnt/shared"

shares:
  default: "nas:shared"
  layout: "{source}/{date}"
```

2. Copy files:

```bash
qc file model.pt server:scratch
qc dir server:projects
qc pull server:scratch
qc dump ./results
```

## Commands

### Copy a file

```bash
qc file report.pdf server:scratch
qc file data.csv server:/absolute/path
```

Copies a single file to a remote destination.

### Copy current directory

```bash
qc dir server:scratch
qc dir server:projects --name backup.zip
qc dir server:scratch --exclude "*.log" --exclude "data/"
```

Zips and copies the current directory. Excludes patterns from config are applied automatically.

### Pull remote directory

```bash
qc pull server:scratch
qc pull server:/remote/path
qc pull server:projects --no-extract
```

Downloads a remote directory to your current working directory. Extracts by default.

### Dump to shared storage

```bash
qc dump ./results
qc dump model.pt
qc dump
```

Copy files to shared storage (configured in `shares.default`). Organizes by source hostname and date.

### List targets

```bash
qc ls
```

Shows configured hosts and path aliases.

### Check setup

```bash
qc doctor
qc doctor --test server nas
```

Validates configuration and SSH connectivity.

## Destination Syntax

- `server` - Use host's default path alias
- `server:scratch` - Use named path alias
- `server:/absolute/path` - Use explicit path

## Configuration

Full configuration example:

```yaml
version: 1

defaults:
  user: myuser
  port: 22
  default_path_alias: scratch
  staging_dir: "/tmp"
  zip:
    exclude:
      - ".git/"
      - "node_modules/"
      - "__pycache__/"
      - ".venv/"
      - "*.tmp"
      - ".DS_Store"
      - "target/"
    follow_symlinks: false

hosts:
  workstation:
    host: "workstation.local"
    role: "development"
    paths:
      scratch: "/home/myuser/scratch"
      projects: "/home/myuser/projects"

  server:
    host: "192.168.1.100"
    role: "compute"
    paths:
      scratch: "/home/myuser/scratch"
      data: "/data"

  nas:
    host: "nas.local"
    role: "storage"
    paths:
      backups: "/mnt/backups"
      shared: "/mnt/shared"
      dumps: "/mnt/shared/quick-copy"

shares:
  default: "nas:dumps"
  layout: "{source}/{date}"
```

See `config.example.yaml` for a complete example.

## Global Options

- `-c, --config <path>` - Custom config file
- `-v, --verbose` - Show ssh/scp commands
- `--json` - Output in JSON format

## Use Cases

### Homelab file transfers
Configure your homelab servers once, then quickly move files between machines.

### Development workflows
Push code to remote build servers, pull results back.

### Backup automation
Script regular dumps to shared storage with automatic organization.

### Multi-machine projects
Work on projects across different machines with quick directory sync.

## License

MIT
