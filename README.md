# omara-cli

The official command-line interface for the Omara project family. Clean, fast, and built in Rust.

## Commands

```bash
omara update          # System update (dnf + flatpak)
omara doctor          # Run health checks
omara app             # Install, remove, search, or reset applications
omara theme           # List, set, or show themes
omara wallpaper       # List, set, or cycle wallpapers
omara config          # Manage CLI configuration values
omara log             # View and manage system logs
omara info            # Show system and component version information
omara help            # AI-powered help
```

## Compilation

Build the release binary:
```bash
cargo build --release
```

The compiled binary will be located under `target/release/omara`.

## Development

The CLI dynamically resolves paths (manifests, system configuration templates) using fallback resolution:
1. Environment variables (`OMARA_WORKSPACE_DIR`, `OMARA_APPS_DIR`, etc.) for local development.
2. Standard system locations (e.g., `/usr/share/omara/`).
3. Standard local fallback (`~/Projects/omara-labs/`).
