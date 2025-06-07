# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a Rust workspace project for ChatGPT History Explorer. The project uses a workspace structure with:

- **Workspace root**: Contains the main `Cargo.toml` with workspace configuration
- **Core crate**: Located at `crates/core/` - contains the main library functionality

The project is in early development with minimal functionality currently implemented.

## Development Commands

### Building
```bash
cargo build                 # Build all workspace members
cargo build -p core         # Build only the core crate
```

### Testing
```bash
cargo test                  # Run all tests in workspace
cargo test -p core          # Run tests for core crate only
```

### Other Cargo Commands
```bash
cargo check                 # Fast compilation check
cargo clippy                # Rust linter
cargo fmt                   # Format code
```

## Project Structure

- `/crates/core/` - Main library crate with core functionality
- Root workspace manages dependencies and build configuration across all crates