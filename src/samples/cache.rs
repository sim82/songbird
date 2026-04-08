//! Sample cache for memory-efficient playback.

use crate::audio::AudioBuffer;
use std::collections::HashMap;
use std::sync::Arc;

/// Statistics about the sample cache.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of samples in cache.
    pub sample_count: usize,
    /// Total memory used (approximate, in bytes).
    pub memory_bytes: usize,
}

/// In-memory cache for audio samples.
#[derive(Debug, Clone)]
pub struct SampleCache {
    /// Cached samples indexed by identifier.
    samples: Arc<HashMap<String, AudioBuffer>>,
}

impl SampleCache {
    /// Create a new empty sample cache.
    pub fn new() -> Self {
        Self {
            samples: Arc::new(HashMap::new()),
        }
    }

    /// Add a sample to the cache.
    pub fn add(&mut self, id: String, buffer: AudioBuffer) {
        Arc::get_mut(&mut self.samples)
            .expect("SampleCache should be uniquely owned")
            .insert(id, buffer);
    }

    /// Load a sample from a WAV file and add to cache.
    pub fn load_and_cache<P: AsRef<std::path::Path>>(
        &mut self,
        id: String,
        path: P,
    ) -> Result<(), String> {
        let buffer = crate::samples::loader::SampleLoader::load(path)?;
        self.add(id, buffer);
        Ok(())
    }

    /// Retrieve a sample from the cache.
    pub fn get(&self, id: &str) -> Option<AudioBuffer> {
        self.samples.get(id).cloned()
    }

    /// Check if a sample exists in the cache.
    pub fn contains(&self, id: &str) -> bool {
        self.samples.contains_key(id)
    }

    /// Get the number of samples in the cache.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Clear all samples from the cache.
    pub fn clear(&mut self) {
        Arc::get_mut(&mut self.samples)
            .expect("SampleCache should be uniquely owned")
            .clear();
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let mut total_memory = 0;
        let mut total_samples = 0;

        for buffer in self.samples.values() {
            total_samples += 1;
            // Approximate memory: 2 channels * 4 bytes per f32
            total_memory += buffer.length * 2 * 4;
        }

        CacheStats {
            sample_count: total_samples,
            memory_bytes: total_memory,
        }
    }

    /// Get list of all sample IDs in the cache.
    pub fn list_samples(&self) -> Vec<String> {
        self.samples.keys().cloned().collect()
    }
}

impl Default for SampleCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let mut cache = SampleCache::new();
        let buffer = AudioBuffer::new_mono(vec![0.1, 0.2, 0.3], 44100);

        cache.add("sample1".to_string(), buffer.clone());
        assert!(cache.contains("sample1"));
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get("sample1").unwrap();
        assert_eq!(retrieved.length, 3);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = SampleCache::new();
        let buffer = AudioBuffer::new_mono(vec![0.0; 1000], 44100);
        cache.add("sample1".to_string(), buffer);

        let stats = cache.stats();
        assert_eq!(stats.sample_count, 1);
        assert_eq!(stats.memory_bytes, 1000 * 2 * 4);
    }

    #[test]
    fn test_list_samples() {
        let mut cache = SampleCache::new();
        cache.add("s1".to_string(), AudioBuffer::new_mono(vec![0.0], 44100));
        cache.add("s2".to_string(), AudioBuffer::new_mono(vec![0.0], 44100));

        let samples = cache.list_samples();
        assert_eq!(samples.len(), 2);
        assert!(samples.contains(&"s1".to_string()));
        assert!(samples.contains(&"s2".to_string()));
    }
}
