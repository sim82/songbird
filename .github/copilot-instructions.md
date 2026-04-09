# Copilot Instructions for Songbird

## Project Overview

Songbird is an ambient sound synthesis framework written in Rust. It orchestrates multiple concurrent audio voices with probabilistic sample scheduling, stereo panning, and hot-reloadable YAML configuration.

### Core Architecture

The system is organized into modular layers:
- **Voices**: Two playback modes (continuous overlapping samples and discrete event-driven mode)
- **Audio**: Buffers, stereo mixing with panning, and platform audio output abstraction
- **Samples**: WAV file loading and in-memory caching
- **Config**: YAML parsing, validation, and file watching for hot-reload
- **Engine**: Main orchestrator coordinating voices, mixing, and output

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
├── voices/
│   ├── mod.rs
│   ├── config.rs        # VoiceConfig and VoiceMode types
│   ├── continuous.rs    # Continuous-mode scheduler (overlapping)
│   ├── discrete.rs      # Discrete-mode scheduler (event-driven)
│   ├── state.rs         # Per-voice playback state
│   └── manager.rs       # VoiceManager orchestration
├── samples/
│   ├── mod.rs
│   ├── loader.rs        # WAV file loading
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

## Voice Concepts

- **Continuous Mode**: Plays samples completely, always selecting new random samples when current finishes. Transitions use fixed-duration crossfading for smooth blending. Deterministic sample succession (no gaps).
- **Discrete Mode**: Triggers samples probabilistically at random intervals. Non-overlapping, event-driven playback. Individual sounds separated in time.
- **Pan**: Stereo positioning from -1.0 (full left) to 1.0 (full right), 0.0 (center)
- **Mode-Specific Config**: VoiceMode enum contains configuration specific to each mode, preventing invalid configs

## Notes

- Phases 1-3 complete: modules, types, and tests in place
- Phases 4-7 focus on mixing/panning, audio output, config hot-reload, and CLI
- Stub implementations exist for file watching (Phase 6); integrate with notify crate when reached
- OS audio output layer (Phase 5) uses minimal abstraction for future portability

