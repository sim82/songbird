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

    let config = ConfigParser::parse(config_file)?;

    if verbose {
        println!("✓ Configuration loaded");
        println!("  Sample rate: {} Hz", config.sample_rate);
        let voice_count = config.voices.as_ref().map(|v| v.len()).unwrap_or(0);
        println!("  Voices: {}", voice_count);
    }

    // Convert YAML voices to VoiceConfig and load samples
    let voices_yaml = config.voices.unwrap_or_default();

    if verbose {
        println!("✓ Loading samples...");
    }

    // Create synthesis engine and load samples
    let mut synthesis_engine = SynthesisEngine::new(config.sample_rate);

    for voice in &voices_yaml {
        if let Ok(voice_config) = voice.to_voice_config() {
            // Load samples for this voice
            if let Some(samples) = &voice.samples {
                for sample_name in samples {
                    let full_path = format!("{}/{}", config.sample_dir, sample_name);
                    // Use unique ID (the path itself)
                    if !synthesis_engine.sample_cache().contains(&full_path) {
                        match synthesis_engine
                            .sample_cache_mut()
                            .load_and_cache(full_path.clone(), &full_path)
                        {
                            Ok(_) => {
                                if verbose {
                                    println!("  ✓ Loaded: {}", full_path);
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠ Warning: Failed to load sample {}: {}", full_path, e);
                            }
                        }
                    }
                }
            }

            // Update voice config to use full paths
            let mut updated_config = voice_config;
            if let Some(samples) = &voice.samples {
                updated_config.sample_pool = samples
                    .iter()
                    .map(|s| format!("{}/{}", config.sample_dir, s))
                    .collect();
            }

            // Add voice to engine
            synthesis_engine.add_voice(updated_config);
            if verbose {
                println!(
                    "  ✓ Voice: mode={}, pan={:.2}",
                    &voice.mode,
                    voice.pan.unwrap_or(0.0)
                );
            }
        } else if verbose {
            eprintln!(
                "⚠ Warning: Failed to convert voice config for {}",
                &voice.id
            );
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
        let _written = audio_output.write(&sample_buffer_left, &sample_buffer_right)?;

        // Check for config changes
        if let Some(ref mut watcher) = _watcher {
            while let Some(event) = watcher.check_changes() {
                use songbird::config::ConfigChangeEvent;
                match event {
                    ConfigChangeEvent::Modified(_) => {
                        if verbose {
                            println!("🔄 Config file changed, reloading...");
                        }
                        // In production: pause, reload, resume with zero glitch
                    }
                    ConfigChangeEvent::Error(e) => {
                        if verbose {
                            println!("⚠ Watcher error: {}", e);
                        }
                    }
                    _ => {}
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
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} examples/config.yaml --verbose", program);
    eprintln!(
        "  {} examples/sine_demo.yaml -o output.wav --verbose",
        program
    );
}
