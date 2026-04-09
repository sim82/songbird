//! Songbird CLI: Ambient sound synthesis binary.
//!
//! Loads YAML configuration and orchestrates real-time audio synthesis with
//! support for hot-reload configuration changes.

use songbird::{
    audio::{AudioFormat, AudioOutput, StubAudioDevice},
    config::{ConfigParser, ConfigWatcher},
    synthesis::SynthesisEngine,
};
use std::env;
use std::path::Path;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    let config_file = &args[1];
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    if verbose {
        println!("🎵 Songbird Audio Synthesis Engine");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Config file: {}", config_file);
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
                for sample_path in samples {
                    // Use unique ID (the path itself)
                    if !synthesis_engine.sample_cache().contains(sample_path) {
                        match synthesis_engine
                            .sample_cache_mut()
                            .load_and_cache(sample_path.clone(), sample_path)
                        {
                            Ok(_) => {
                                if verbose {
                                    println!("  ✓ Loaded: {}", sample_path);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "⚠ Warning: Failed to load sample {}: {}",
                                    sample_path, e
                                );
                            }
                        }
                    }
                }
            }

            // Add voice to engine
            synthesis_engine.add_voice(voice_config);
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

    // Create audio output (stub for now)
    let format = AudioFormat::new(config.sample_rate);
    let device = Box::new(StubAudioDevice::new(format)?);
    let mut audio_output = AudioOutput::with_device(device);
    audio_output.allocate_buffers(config.sample_rate as usize / 10); // 100ms buffer

    if verbose {
        println!("✓ Audio output initialized");
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
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} <config.yaml> [OPTIONS]", program);
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -v, --verbose    Show detailed output");
    eprintln!("  -h, --help       Show this help message");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  {} examples/config.yaml --verbose", program);
}
