//! Songbird CLI: Ambient sound synthesis binary.
//!
//! Loads YAML configuration and orchestrates real-time audio synthesis with
//! support for hot-reload configuration changes.

use songbird::{
    audio::{AudioFormat, AudioOutput, StubAudioDevice, WavWriter},
    config::{ConfigParser, ConfigWatcher},
    synthesis::SynthesisEngine,
};
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

fn setup_signal_handler() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::Relaxed);
        SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");
}

fn create_audio_device(
    format: AudioFormat,
    output_file: Option<&str>,
    verbose: bool,
) -> Result<Box<dyn songbird::audio::AudioDevice>, Box<dyn std::error::Error>> {
    // If output file is specified, use WAV writer
    if let Some(file_path) = output_file {
        if verbose {
            println!("  Writing to WAV file: {}", file_path);
        }
        let device = WavWriter::new(file_path, format)?;
        return Ok(Box::new(device));
    }

    #[cfg(feature = "alsa")]
    {
        match songbird::audio::create_alsa_device(format) {
            Ok(device) => {
                if verbose {
                    println!("  Using ALSA audio backend");
                }
                return Ok(device);
            }
            Err(e) => {
                if verbose {
                    println!("  ⚠ ALSA initialization failed: {}", e);
                    println!("  Falling back to stub device");
                }
            }
        }
    }

    if verbose {
        println!("  Using stub audio device (no actual playback)");
    }
    Ok(Box::new(StubAudioDevice::new(format)?))
}

fn build_and_preload_voices(synthesis_engine: &mut SynthesisEngine, config: &songbird::config::parser::Config, voices_yaml: &[songbird::config::parser::VoiceConfigYaml], verbose: bool) -> Vec<songbird::voices::VoiceConfig> {
    let mut new_voice_configs = Vec::new();

    for voice in voices_yaml {
        match voice.to_voice_config() {
            Ok(mut vc) => {
                // Ensure sample paths are fully qualified and preload missing samples
                if let Some(samples) = &voice.samples {
                    let full_paths: Vec<String> = samples
                        .iter()
                        .map(|s| format!("{}/{}", config.sample_dir, s))
                        .collect();

                    for p in &full_paths {
                        if !synthesis_engine.sample_cache().contains(p) {
                            if let Err(e) = synthesis_engine.sample_cache_mut().load_and_cache(p.clone(), p.clone()) {
                                eprintln!("⚠ Warning: Failed to load sample {}: {}", p, e);
                            } else if verbose {
                                println!("  ✓ Loaded: {}", p);
                            }
                        }
                    }

                    vc.sample_pool = full_paths;
                }

                new_voice_configs.push(vc);
            }
            Err(e) => {
                eprintln!("⚠ Warning: Failed to convert voice config for {}: {}", voice.id, e);
            }
        }
    }

    new_voice_configs
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up signal handling for graceful shutdown
    setup_signal_handler();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage(&args[0]);
        return Ok(());
    }

    let config_file = &args[1];
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    // Parse sample rate override option (optional)
    let sample_rate_override = args
        .windows(2)
        .find(|w| w[0] == "-r" || w[0] == "--sample-rate")
        .and_then(|w| w[1].parse::<u32>().ok());

    // Parse output file option
    let output_file = args
        .windows(2)
        .find(|w| w[0] == "-o" || w[0] == "--output")
        .map(|w| w[1].as_str());

    if verbose {
        println!("🎵 Songbird Audio Synthesis Engine");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Config file: {}", config_file);
        if let Some(out) = output_file {
            println!("Output: {}", out);
        }
    }

    // Load and validate configuration
    if !Path::new(config_file).exists() {
        eprintln!("❌ Error: Config file not found: {}", config_file);
        return Err("Config file not found".into());
    }

    let mut config = ConfigParser::parse(config_file)?;

    if verbose {
        println!("✓ Configuration loaded");
        let effective_sample_rate = sample_rate_override.unwrap_or(config.sample_rate);
        println!(
            "  Sample rate: {} Hz (effective: {} Hz)",
            config.sample_rate, effective_sample_rate
        );
        let voice_count = config.voices.as_ref().map(|v| v.len()).unwrap_or(0);
        println!("  Voices: {}", voice_count);
    }

    // Convert YAML voices to VoiceConfig and load samples
    let voices_yaml = config.voices.clone().unwrap_or_default();

    if verbose {
        println!("✓ Loading samples...");
    }

    // Create synthesis engine
    let effective_sample_rate = sample_rate_override.unwrap_or(config.sample_rate);
    let mut synthesis_engine = SynthesisEngine::new(effective_sample_rate);

    // Build voice configs and preload samples using shared helper
    let new_voice_configs = build_and_preload_voices(&mut synthesis_engine, &config, &voices_yaml, verbose);

    // Atomically populate engine voices for startup
    synthesis_engine.replace_voices(new_voice_configs);

    // Verbose listing of voices
    for voice in &voices_yaml {
        if let Ok(_) = voice.to_voice_config() {
            if verbose {
                println!("  ✓ Voice: mode={}, pan={:.2}", &voice.mode, voice.pan.unwrap_or(0.0));
            }
        } else if verbose {
            eprintln!("⚠ Warning: Failed to convert voice config for {}", &voice.id);
        }
    }

    if verbose {
        println!("✓ Synthesis engine initialized");
    }

    // Create audio output with appropriate backend
    let format = AudioFormat::new(config.sample_rate);
    if verbose {
        println!("✓ Initializing audio output");
    }
    let device = create_audio_device(format, output_file, verbose)?;
    let mut audio_output = AudioOutput::with_device(device);
    audio_output.allocate_buffers(config.sample_rate as usize / 10); // 100ms buffer

    if verbose {
        println!("  Format: {} Hz, stereo", config.sample_rate);
        println!("  Latency: {}ms (approx)", audio_output.latency_ms());
    }

    // Start audio
    audio_output.start()?;
    synthesis_engine.start();

    if verbose {
        println!("\n🎵 Playback started");
        println!("Press Ctrl+C to stop...\n");
    }

    // Set up file watcher for hot-reload
    let mut _watcher = match ConfigWatcher::new(config_file) {
        Ok(w) => {
            if verbose {
                println!("✓ Config watcher enabled (hot-reload active)");
            }
            Some(w)
        }
        Err(_) => {
            if verbose {
                println!("⚠ Config watcher disabled");
            }
            None
        }
    };

    // Main synthesis loop
    let frames_per_chunk = config.sample_rate as usize / 100; // 10ms chunks
    let mut sample_buffer_left = vec![0.0; frames_per_chunk];
    let mut sample_buffer_right = vec![0.0; frames_per_chunk];

    // Hot-reload debounce handled by ConfigWatcher; set desired debounce here
    let reload_debounce = Duration::from_millis(250); // tunable debounce duration (250ms)
    // Track last applied config file modification time to avoid reloading multiple times for the same on-disk change
    let mut last_applied_mtime: Option<std::time::SystemTime> = None;

    loop {
        // Check for shutdown signal
        if SHUTDOWN_REQUESTED.load(Ordering::Relaxed) {
            if verbose {
                println!("\n🛑 Shutdown requested, finalizing...");
            }
            break;
        }

        // Synthesize audio
        for frame_idx in 0..frames_per_chunk {
            let (left, right) = synthesis_engine.process_frame();
            sample_buffer_left[frame_idx] = left;
            sample_buffer_right[frame_idx] = right;
        }

        // Write to output
        let res = audio_output.write(&sample_buffer_left, &sample_buffer_right);

        if res.is_err() {
            println!("write error")
        }

        // Check for config changes using debounced watcher API
        if let Some(ref mut watcher) = _watcher {
            if let Some(evt) = watcher.check_debounced_change(reload_debounce) {
                match evt {
                    songbird::config::ConfigChangeEvent::Modified(_) | songbird::config::ConfigChangeEvent::Created(_) => {
                        // Check file modification time and only reload if it changed since last applied reload
                        match std::fs::metadata(config_file).and_then(|m| m.modified()) {
                            Ok(mtime) => {
                                if let Some(last_mtime) = last_applied_mtime && mtime <= last_mtime {
                                    if verbose {
                                        println!("⚠ Reload suppressed (no new modification on disk)");
                                    }
                                } else {
                                    last_applied_mtime = Some(mtime);

                                    if verbose {
                                        println!("🔄 Config file changed, reloading...");
                                    }

                                    // Attempt to parse the new config
                                    match ConfigParser::parse(config_file) {
                                        Ok(new_config) => {
                                            // If sample rate changed, warn and ignore
                                            if new_config.sample_rate != config.sample_rate && verbose {
                                                println!("⚠ Sample rate change detected in config; ignoring at runtime. Restart required to change sample rate.");
                                            }

                                            // Use shared helper to build voice configs and preload samples
                                            let voices_yaml = new_config.voices.clone().unwrap_or_default();
                                            let new_voice_configs = build_and_preload_voices(&mut synthesis_engine, &new_config, &voices_yaml, verbose);

                                            // Atomically replace voices in the synthesis engine.
                                            synthesis_engine.replace_voices(new_voice_configs);

                                            // Update stored config
                                            config = new_config;

                                            if verbose {
                                                println!("✓ Hot-reload applied (voices updated)");
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("⚠ Failed to parse new config: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠ Could not stat config file before reload: {}", e);
                                // proceed with best-effort reload
                            }
                        }
                    }
                    songbird::config::ConfigChangeEvent::Error(err) => {
                        if verbose {
                            println!("⚠ Watcher error: {}", err);
                        }
                    }
                }
            }
        }

        // Simulate small sleep to prevent busy-waiting (in production, sleep duration
        // would be tuned to the buffer fill level)
        std::thread::sleep(Duration::from_millis(1));
    }

    // Gracefully stop audio output
    if verbose {
        println!("✓ Stopping audio output...");
    }
    audio_output.stop()?;

    if verbose {
        println!("✓ Shutdown complete");
    }

    Ok(())
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} <config.yaml> [OPTIONS]", program);
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -v, --verbose       Show detailed output");
    eprintln!("  -o, --output FILE   Write to WAV file instead of audio device");
    eprintln!("  -h, --help          Show this help message");
    eprintln!("  -r, --sample-rate N Override sample rate (Hz) for testing)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} examples/config.yaml --verbose", program);
    eprintln!(
        "  {} examples/sine_demo.yaml -o output.wav --verbose",
        program
    );
}
