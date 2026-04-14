Songbird — Ambient Sound Synthesis

Overview

Songbird synthesizes ambient soundscapes by mixing multiple "voices" backed by WAV samples. It supports two voice modes:
- Continuous: overlaps samples and crossfades between them.
- Discrete: event-driven, non-overlapping triggers ("birds").

Features

- Type-safe voice configuration (continuous vs. discrete modes)
- Hot-reloadable YAML config (glitch-free atomic voice replacement)
- Stereo panning and mixing
- WAV file I/O (both input samples and output audio)
- ALSA audio backend (Linux) with fallback to stub device
- Minimal dependencies (serde, serde_yaml, rand, notify, wav)

Quick start

Build:
  cargo build --release

Run with example config (plays to audio device):
  ./target/release/songbird examples/sine_demo.yaml -v

Write to WAV file:
  ./target/release/songbird examples/sine_demo.yaml -o out.wav -v

CLI flags

- -o, --output FILE : write to a WAV file instead of audio device
- -r, --sample-rate N : override sample rate (Hz) for testing
- -v, --verbose : verbose logging
- -h, --help : show usage

YAML config

Core structure:
  sample_rate: 44100
  sample_dir: ./samples
  voices:
    - id: my_voice
      mode: continuous  # or "discrete"
      pan: -0.5         # stereo position (-1.0 left to 1.0 right)
      samples:
        - sample1.wav
        - sample2.wav

Continuous mode (overlapping samples with crossfade):
  - overlap_ms: crossfade duration (default 500 ms)

Discrete mode (event-driven, non-overlapping):
  - probability: per-second trigger rate (0.0 to 1.0)
  - min_delay_ms: minimum delay between events
  - max_delay_ms: maximum delay between events

Discrete probability semantics

The discrete voice probability in YAML is a per-second rate (0.0–1.0), not per-sample. This allows intuitive config: 0.5 means ~2 events per second on average.

Internally it's converted to a per-sample trigger probability:
  p_sample = 1 - (1 - p_sec)^(1 / sample_rate)

This ensures the expected trigger rate matches the configured value regardless of sample rate.

Hot-reload

Songbird watches the config file and hot-reloads voice configs without audible glitches:

1. Detects config file changes via notify crate
2. Debounces rapid events (default 250ms) to avoid processing multiple editor save cycles
3. Preloads any new samples into the existing cache (no re-init required)
4. Atomically replaces the engine's voice list (running voices remain active, no clicks)
5. Ignores reloads when file mtime hasn't advanced (suppresses duplicates)

To test: edit examples/sine_demo.yaml while the process is running (e.g., change probability from 0.15 to 0.45). You should see one "Config file changed, reloading..." message and smooth audio transition.

Building and testing

Run all tests:
  cargo test

Format and lint:
  cargo fmt
  cargo clippy --fix

Example config walkthrough

See examples/sine_demo.yaml:
- Two simple sine-wave samples at different frequencies
- One continuous voice (panned left) plays overlapping sine samples
- One discrete voice (panned right) triggers randomly
- Good starting point for understanding voice modes

Troubleshooting

- Silence in WAV output: check that samples loaded correctly (use -v). Ensure engine started and voices activated.
- Audio device errors: if ALSA fails, the program falls back to stub device (no actual playback). Verify ALSA is installed or use -o to write WAV.
- Config parse errors: validate YAML syntax and ensure sample files exist at the configured sample_dir path.
- Sample rate mismatch: runtime sample-rate changes are ignored. To change sample rate, restart the process.

Architecture

- Audio: buffer primitives, stereo mixer with panning, backend abstraction (WAV writer, ALSA, stub)
- Config: YAML parser, validator, file watcher with debounce
- Samples: WAV loader, in-memory cache
- Voices: continuous and discrete schedulers, per-voice state, per-frame synthesis
- Synthesis: main orchestrator combining voices, mixing, and scheduling
- CLI: main binary with hot-reload loop and graceful shutdown

License

MIT — see LICENSE