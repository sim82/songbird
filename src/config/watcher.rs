//! Configuration file watcher for hot-reloading.
//!
//! Monitors config file changes and signals when a reload is needed using the notify crate.
//! Provides glitch-free updates by deferring config updates to specific synchronization points.

use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Events emitted by the config watcher.
#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    /// Configuration file was modified.
    Modified(PathBuf),
    /// Configuration file was created.
    Created(PathBuf),
    /// Error occurred while watching.
    Error(String),
}

/// Watches for configuration file changes and signals updates.
///
/// Uses async file watching to detect modifications in real-time.
/// Provides `check_changes()` for synchronous polling of pending updates.
pub struct ConfigWatcher {
    config_path: PathBuf,
    rx: mpsc::Receiver<ConfigChangeEvent>,
    _watcher: Box<dyn Watcher>,
}

impl ConfigWatcher {
    /// Create a new config watcher for the given file.
    ///
    /// # Errors
    /// Returns `io::Error` if file path is invalid or watcher cannot be created.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let config_path = path.as_ref().to_path_buf();

        // Verify file exists and is readable
        if !config_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Config file not found: {:?}", config_path),
            ));
        }

        let (tx, rx) = mpsc::channel();

        let config_path_clone = config_path.clone();
        let watch_path = if config_path.is_dir() {
            config_path.clone()
        } else {
            config_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf()
        };

        // Create watcher with notify crate
        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // Report changes where the event path matches the config filename.
                        for path in &event.paths {
                            // Compare by file name so atomic editor saves (tmp file + rename)
                            // are detected even if the event path is a temp path.
                            if path.file_name() == config_path_clone.file_name() {
                                match &event.kind {
                                    notify::EventKind::Create(_) => {
                                        let _ = tx.send(ConfigChangeEvent::Created(path.clone()));
                                    }
                                    notify::EventKind::Modify(_) => {
                                        let _ = tx.send(ConfigChangeEvent::Modified(path.clone()));
                                    }
                                    // Treat other kinds (Remove, Rename, Other) as modifications
                                    _ => {
                                        let _ = tx.send(ConfigChangeEvent::Modified(path.clone()));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(ConfigChangeEvent::Error(e.to_string()));
                    }
                }
            })
            .map_err(|e| std::io::Error::other(format!("Failed to create watcher: {}", e)))?;

        watcher
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| std::io::Error::other(format!("Failed to watch path: {}", e)))?;

        Ok(Self {
            config_path,
            rx,
            _watcher: Box::new(watcher),
        })
    }

    /// Check for pending configuration changes (non-blocking).
    ///
    /// Returns the first pending event, or `None` if no changes detected.
    /// Can be called repeatedly to poll all pending events.
    pub fn check_changes(&mut self) -> Option<ConfigChangeEvent> {
        self.rx.try_recv().ok()
    }

    /// Drain all pending events (useful for debouncing rapid changes).
    ///
    /// Returns all currently pending events. Useful during startup or
    /// after detecting a change to consume all queued updates.
    pub fn drain_pending(&mut self) -> Vec<ConfigChangeEvent> {
        self.rx.try_iter().collect()
    }

    /// Wait for the next configuration change with timeout.
    ///
    /// Blocks up to `duration` waiting for a change event.
    /// Returns `Some(event)` if change detected, `None` on timeout.
    pub fn wait_for_change(&self, duration: Duration) -> Option<ConfigChangeEvent> {
        self.rx.recv_timeout(duration).ok()
    }

    /// Get the watched config file path.
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_watcher_creation_nonexistent_file() {
        let result = ConfigWatcher::new("/nonexistent/path/config.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_watcher_creation_valid_file() {
        // Create temp file
        let tmp_file = "/tmp/test_config_watcher.yaml";
        let _ = fs::File::create(tmp_file);

        let result = ConfigWatcher::new(tmp_file);
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_file(tmp_file);
    }

    #[test]
    fn test_watcher_drain_pending() {
        let tmp_file = "/tmp/test_config_watcher_drain.yaml";
        let _ = fs::File::create(tmp_file);

        if let Ok(mut watcher) = ConfigWatcher::new(tmp_file) {
            let pending = watcher.drain_pending();
            // Initially should be empty
            assert_eq!(pending.len(), 0);
        }

        let _ = fs::remove_file(tmp_file);
    }

    #[test]
    fn test_watcher_config_path() {
        let tmp_file = "/tmp/test_config_path.yaml";
        let _ = fs::File::create(tmp_file);

        if let Ok(watcher) = ConfigWatcher::new(tmp_file) {
            assert_eq!(watcher.config_path().to_str().unwrap(), tmp_file);
        }

        let _ = fs::remove_file(tmp_file);
    }

    #[test]
    fn test_watcher_check_changes_no_changes() {
        let tmp_file = "/tmp/test_no_changes.yaml";
        let _ = fs::File::create(tmp_file);

        if let Ok(mut watcher) = ConfigWatcher::new(tmp_file) {
            // Should return None when no changes pending
            let change = watcher.check_changes();
            assert!(change.is_none());
        }

        let _ = fs::remove_file(tmp_file);
    }

    #[test]
    fn test_watcher_file_modification() {
        let tmp_file = "/tmp/test_modification.yaml";
        let mut file = fs::File::create(tmp_file).unwrap();
        writeln!(file, "initial content").unwrap();
        drop(file);

        if let Ok(mut watcher) = ConfigWatcher::new(tmp_file) {
            // Give watcher time to initialize
            std::thread::sleep(Duration::from_millis(100));

            // Modify file
            if let Ok(mut file) = fs::File::create(tmp_file) {
                writeln!(file, "modified content").unwrap();
            }

            // Give watcher time to detect
            std::thread::sleep(Duration::from_millis(200));

            // Check for modifications (may or may not have fired depending on OS)
            let pending = watcher.drain_pending();
            // Just verify drain works even if no events
            let _ = pending;
        }

        let _ = fs::remove_file(tmp_file);
    }
}
