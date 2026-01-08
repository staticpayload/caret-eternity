// Caret IO - File IO nodes
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::nodes::{SinkNode, SourceNode};
use caret_core::{Error, Packet, Result};
use std::path::{Path, PathBuf};

/// Configuration for a file source
#[derive(Clone, Debug)]
pub struct FileSourceConfig {
    /// Path to the file to read
    pub path: PathBuf,
    /// Buffer size for reading
    pub buffer_size: usize,
    /// Whether to loop the file
    pub loop_file: bool,
}

impl Default for FileSourceConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            buffer_size: 8192,
            loop_file: false,
        }
    }
}

impl FileSourceConfig {
    /// Create a new file source config
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set whether to loop the file
    pub fn with_loop(mut self, loop_file: bool) -> Self {
        self.loop_file = loop_file;
        self
    }
}

/// A source node that reads from a file
pub struct FileSource {
    config: FileSourceConfig,
    position: usize,
    data: Vec<u8>,
    exhausted: bool,
}

impl FileSource {
    /// Create a new file source
    pub fn new(config: FileSourceConfig) -> Result<Self> {
        // Read the file into memory
        let data = std::fs::read(&config.path)
            .map_err(|e| Error::io(format!("failed to read file: {}", e)))?;

        if data.is_empty() {
            return Err(Error::invalid_input("file is empty"));
        }

        Ok(Self {
            config,
            position: 0,
            data,
            exhausted: false,
        })
    }

    /// Create from a path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::new(FileSourceConfig::new(path))
    }

    /// Reset to the beginning of the file
    pub fn reset(&mut self) {
        self.position = 0;
        self.exhausted = false;
    }
}

impl SourceNode for FileSource {
    fn generate(&mut self) -> Result<Option<Packet>> {
        if self.exhausted {
            if self.config.loop_file {
                self.reset();
            } else {
                return Ok(None);
            }
        }

        let remaining = self.data.len() - self.position;
        let chunk_size = self.config.buffer_size.min(remaining);

        if chunk_size == 0 {
            self.exhausted = true;
            return Ok(None);
        }

        let end = self.position + chunk_size;
        let chunk = self.data[self.position..end].to_vec();
        self.position = end;

        Ok(Some(Packet::bytes(chunk)))
    }
}

/// Configuration for a file sink
#[derive(Clone, Debug)]
pub struct FileSinkConfig {
    /// Path to the output file
    pub path: PathBuf,
    /// Buffer size for writing
    pub buffer_size: usize,
    /// Whether to append to existing file
    pub append: bool,
}

impl Default for FileSinkConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            buffer_size: 8192,
            append: false,
        }
    }
}

impl FileSinkConfig {
    /// Create a new file sink config
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Set whether to append
    pub fn with_append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }
}

/// A sink node that writes to a file
pub struct FileSink {
    config: FileSinkConfig,
    buffer: Vec<u8>,
    bytes_written: u64,
}

impl FileSink {
    /// Create a new file sink
    pub fn new(config: FileSinkConfig) -> Self {
        Self {
            config,
            buffer: Vec::new(),
            bytes_written: 0,
        }
    }

    /// Create from a path
    pub fn to_path(path: impl AsRef<Path>) -> Self {
        Self::new(FileSinkConfig::new(path))
    }

    /// Get the number of bytes written
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Flush the buffer to disk
    pub fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        use std::io::Write;
        if self.config.append {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&self.config.path)
                .map_err(|e| Error::io(format!("failed to open file: {}", e)))?;
            file
                .write_all(&self.buffer)
                .map_err(|e| Error::io(format!("failed to write file: {}", e)))?;
        } else {
            let mut file = std::fs::File::create(&self.config.path)
                .map_err(|e| Error::io(format!("failed to create file: {}", e)))?;
            file
                .write_all(&self.buffer)
                .map_err(|e| Error::io(format!("failed to write file: {}", e)))?;
        }

        self.bytes_written += self.buffer.len() as u64;
        self.buffer.clear();
        Ok(())
    }
}

impl Drop for FileSink {
    fn drop(&mut self) {
        // Best effort flush on drop
        let _ = self.flush();
    }
}

impl SinkNode for FileSink {
    fn consume(&mut self, packet: Packet) -> Result<()> {
        self.buffer.extend_from_slice(packet.data());
        if self.buffer.len() >= self.config.buffer_size {
            self.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_file_source_config() {
        let config = FileSourceConfig::new("/tmp/test.txt")
            .with_buffer_size(4096)
            .with_loop(true);
        assert_eq!(config.path, PathBuf::from("/tmp/test.txt"));
        assert_eq!(config.buffer_size, 4096);
        assert!(config.loop_file);
    }

    #[test]
    fn test_file_source_empty_file() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("caret_empty_test.txt");

        // Create empty file
        std::fs::write(&file_path, b"").unwrap();

        let result = FileSource::from_path(&file_path);
        assert!(result.is_err());

        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_source_read() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("caret_source_test.txt");

        // Create file with content
        std::fs::write(&file_path, b"hello world").unwrap();

        let mut source = FileSource::from_path(&file_path).unwrap();
        assert_eq!(source.data, b"hello world");

        let packet = source.generate().unwrap().unwrap();
        assert_eq!(packet.data().as_ref(), b"hello world");

        // File is now exhausted
        assert!(source.generate().unwrap().is_none());

        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_source_loop() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("caret_loop_test.txt");

        std::fs::write(&file_path, b"test").unwrap();

        let mut source = FileSource::new(FileSourceConfig::new(&file_path).with_loop(true)).unwrap();

        // First read
        let p1 = source.generate().unwrap().unwrap();
        assert_eq!(p1.data().as_ref(), b"test");

        // Exhausted
        assert!(source.generate().unwrap().is_none());

        // But loops, so next read gives data again
        let p2 = source.generate().unwrap().unwrap();
        assert_eq!(p2.data().as_ref(), b"test");

        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_sink() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("caret_sink_test.txt");

        let mut sink = FileSink::to_path(&file_path);

        // Consume some packets
        sink.consume(Packet::bytes(&b"hello "[..])).unwrap();
        sink.consume(Packet::bytes(&b"world"[..])).unwrap();

        // Flush to disk
        sink.flush().unwrap();

        // Verify content
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world");
        assert_eq!(sink.bytes_written(), 11);

        std::fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_sink_append() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("caret_sink_append_test.txt");

        // Initial write
        {
            let mut sink = FileSink::new(FileSinkConfig::new(&file_path));
            sink.consume(Packet::bytes(&b"hello "[..])).unwrap();
            sink.flush().unwrap();
        }

        // Append
        {
            let mut sink = FileSink::new(FileSinkConfig::new(&file_path).with_append(true));
            sink.consume(Packet::bytes(&b"world"[..])).unwrap();
            sink.flush().unwrap();
        }

        // Verify
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world");

        std::fs::remove_file(&file_path).ok();
    }
}
