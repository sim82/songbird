# Copilot Instructions for Songbird

## Project Overview

Songbird is an ambient sound synthesis framework written in Rust. It orchestrates multiple concurrent audio streams with probabilistic sample scheduling, stereo panning, and hot-reloadable YAML configuration.

### Core Architecture

The system is organized into modular layers:
- **Streams**: Two playback modes (continuous overlapping samples and event-driven "bird" mode)
- **Audio**: Buffers, stereo mixing with panning, and platform audio output abstraction
- **Samples**: WAV file loading and in-memory caching
- **Config**: YAML parsing, validation, and file watching for hot-reload
- **Engine**: Main orchestrator coordinating streams, mixing, and output

## Build, Test, and Lint Commands

### Building
```bash
cargo build                 # Debug build
cargo build --release      # Optimized release build
```

### Testing
```bash
cargo test                 # Run all tests
cargo test --lib          # Run library tests only
cargo test -- --nocapture # Run with output displayed
cargo test <test_name>    # Run a specific test by name
```

### Linting and Formatting
```bash
cargo clippy              # Lint with Clippy
cargo clippy --fix        # Auto-fix Clippy warnings
cargo fmt                 # Format code with Rustfmt
cargo fmt -- --check      # Check formatting without modifying
```

### Checking
```bash
cargo check               # Fast check without building
```

## Project Structure

```
src/
├── main.rs              # Binary entry point
├── lib.rs               # Library root, module exports
├── audio/
│   ├── mod.rs
│   ├── buffer.rs        # AudioBuffer primitives (stereo/mono)
│   ├── mixer.rs         # Stereo mixing with panning
│   └── output.rs        # OS audio layer abstraction (stub)
├── streams/
│   ├── mod.rs
│   ├── config.rs        # StreamConfig and StreamMode types
│   ├── continuous.rs    # Continuous-mode scheduler (overlapping)
│   ├── bird.rs          # Bird-mode scheduler (event-driven)
│   └── state.rs         # Per-stream playback state
├── samples/
│   ├── mod.rs
│   ├── loader.rs        # WAV file loading (stub, Phase 2)
│   └── cache.rs         # In-memory sample cache
├── config/
│   ├── mod.rs
│   ├── parser.rs        # YAML config parsing
│   ├── validator.rs     # Config validation
│   └── watcher.rs       # File watcher for hot-reload (stub, Phase 6)
└── engine.rs            # Main synthesis engine
```

## Key Conventions

- **Edition**: Rust 2024 edition
- **Code style**: Standard Rust conventions; enforced with `cargo fmt` and `cargo clippy`
- **Testing**: Unit tests co-located with implementation; run with `cargo test --lib`
- **Minimal dependencies**: Only essential crates used (serde, serde_yaml, rand, notify, wav)
- **Module structure**: Public API exported via mod.rs in each module; internal types use `pub use`

## Stream Concepts

- **Continuous Mode**: Probabilistically selects and overlaps multiple samples from a pool (e.g., water splattering)
- **Bird Mode**: Event-driven playback of individual, non-overlapping samples (e.g., rain drops, bird chirps)
- **Pan**: Stereo positioning from -1.0 (full left) to 1.0 (full right), 0.0 (center)

## Notes

- Phase 1 (Foundation) is complete: modules, types, and tests in place
- Phases 2-7 focus on audio infrastructure, scheduling logic, mixing, output, config hot-reload, and CLI
- Stub implementations exist for WAV loading (Phase 2) and file watching (Phase 6); integrate with actual crate APIs when reaching those phases
- OS audio output layer (Phase 5) uses minimal abstraction for future portability

