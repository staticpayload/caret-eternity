// Caret Record - Event storage
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Error, Result, SessionId, RecordingMetadata, Event};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Trait for event storage backends
pub trait EventStore: Send + Sync {
    /// Store an event
    fn store_event(&self, event: Event) -> Result<()>;

    /// Load events for a session
    fn load_events(&self, session_id: SessionId) -> Result<Vec<Event>>;

    /// Save recording metadata
    fn save_metadata(&self, metadata: &RecordingMetadata) -> Result<()>;

    /// Get recording metadata
    fn get_metadata(&self, session_id: SessionId) -> Result<Option<RecordingMetadata>>;

    /// List all recording sessions
    fn list_sessions(&self) -> Result<Vec<SessionId>>;

    /// Clear all data for a session
    fn clear_session(&self, session_id: SessionId) -> Result<()>;
}

/// In-memory event store
pub struct MemoryStore {
    /// Events by session ID
    events: Arc<Mutex<HashMap<u64, Vec<Event>>>>,
    /// Metadata by session ID
    metadata: Arc<Mutex<HashMap<u64, RecordingMetadata>>>,
}

impl MemoryStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(HashMap::new())),
            metadata: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore for MemoryStore {
    fn store_event(&self, event: Event) -> Result<()> {
        let mut events = self.events.lock();
        events.entry(event.session_id).or_default().push(event);
        Ok(())
    }

    fn load_events(&self, session_id: SessionId) -> Result<Vec<Event>> {
        let events = self.events.lock();
        Ok(events.get(&session_id.as_u64())
            .cloned()
            .unwrap_or_default())
    }

    fn save_metadata(&self, metadata: &RecordingMetadata) -> Result<()> {
        let mut store_metadata = self.metadata.lock();
        store_metadata.insert(metadata.session_id, metadata.clone());
        Ok(())
    }

    fn get_metadata(&self, session_id: SessionId) -> Result<Option<RecordingMetadata>> {
        let metadata = self.metadata.lock();
        Ok(metadata.get(&session_id.as_u64()).cloned())
    }

    fn list_sessions(&self) -> Result<Vec<SessionId>> {
        let metadata = self.metadata.lock();
        let ids: Vec<SessionId> = metadata.keys()
            .map(|&id| SessionId(id))
            .collect();
        Ok(ids)
    }

    fn clear_session(&self, session_id: SessionId) -> Result<()> {
        let id = session_id.as_u64();
        self.events.lock().remove(&id);
        self.metadata.lock().remove(&id);
        Ok(())
    }
}

/// File-based event store
pub struct FileStore {
    /// Base directory for storing recordings
    base_dir: PathBuf,
}

impl FileStore {
    /// Create a new file-based store
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    /// Get the directory for a session
    fn session_dir(&self, session_id: SessionId) -> PathBuf {
        self.base_dir.join(session_id.as_u64().to_string())
    }

    /// Get the events file for a session
    fn events_file(&self, session_id: SessionId) -> PathBuf {
        self.session_dir(session_id).join("events.jsonl")
    }

    /// Get the metadata file for a session
    fn metadata_file(&self, session_id: SessionId) -> PathBuf {
        self.session_dir(session_id).join("metadata.json")
    }

    /// Ensure session directory exists
    fn ensure_session_dir(&self, session_id: SessionId) -> Result<()> {
        fs::create_dir_all(self.session_dir(session_id))?;
        Ok(())
    }
}

impl EventStore for FileStore {
    fn store_event(&self, event: Event) -> Result<()> {
        self.ensure_session_dir(SessionId(event.session_id))?;

        let file = self.events_file(SessionId(event.session_id));
        let mut should_append = true;

        // Check if file exists
        if !file.exists() {
            // Create new file with header
            let f = File::create(&file)?;
            let mut writer = BufWriter::new(f);
            writeln!(writer, "[").map_err(|e| Error::StoreError(e.to_string()))?;
        } else {
            // Check if we need to close and reopen
            // For simplicity, we'll use append mode
            should_append = true;
        }

        // Append event to file (using JSONL format for simplicity)
        let f = fs::OpenOptions::new()
            .append(true)
            .open(&file)?;

        let mut writer = BufWriter::new(f);
        let json = serde_json::to_string(&event)
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        writeln!(writer, "{}", json).map_err(|e| Error::StoreError(e.to_string()))?;

        Ok(())
    }

    fn load_events(&self, session_id: SessionId) -> Result<Vec<Event>> {
        let file = self.events_file(session_id);

        if !file.exists() {
            return Ok(Vec::new());
        }

        let f = File::open(&file)?;
        let reader = BufReader::new(f);
        let mut events = Vec::new();

        for line in std::io::BufRead::lines(reader) {
            let line = line.map_err(|e| Error::StoreError(e.to_string()))?;
            if line.trim().is_empty() || line.starts_with('[') {
                continue;
            }
            // Remove trailing comma if present
            let line = line.trim_end_matches(',');
            let event: Event = serde_json::from_str(&line)
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }

    fn save_metadata(&self, metadata: &RecordingMetadata) -> Result<()> {
        self.ensure_session_dir(SessionId(metadata.session_id))?;

        let file = self.metadata_file(SessionId(metadata.session_id));
        let f = File::create(&file)?;
        let writer = BufWriter::new(f);

        serde_json::to_writer_pretty(writer, metadata)
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(())
    }

    fn get_metadata(&self, session_id: SessionId) -> Result<Option<RecordingMetadata>> {
        let file = self.metadata_file(session_id);

        if !file.exists() {
            return Ok(None);
        }

        let f = File::open(&file)?;
        let reader = BufReader::new(f);
        let metadata: RecordingMetadata = serde_json::from_reader(reader)
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(Some(metadata))
    }

    fn list_sessions(&self) -> Result<Vec<SessionId>> {
        let mut sessions = Vec::new();

        if !self.base_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&self.base_dir)
            .map_err(|e| Error::StoreError(e.to_string()))?
        {
            let entry = entry.map_err(|e| Error::StoreError(e.to_string()))?;
            let name = entry.file_name();
            if let Some(id_str) = name.to_str() {
                if let Ok(id) = id_str.parse::<u64>() {
                    sessions.push(SessionId(id));
                }
            }
        }

        sessions.sort();
        Ok(sessions)
    }

    fn clear_session(&self, session_id: SessionId) -> Result<()> {
        let dir = self.session_dir(session_id);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}

/// Store backend factory
pub enum StoreBackend {
    /// In-memory store
    Memory,
    /// File-based store
    File(PathBuf),
}

impl StoreBackend {
    /// Create the store instance
    pub fn create(self) -> Result<Arc<dyn EventStore>> {
        match self {
            StoreBackend::Memory => Ok(Arc::new(MemoryStore::new())),
            StoreBackend::File(path) => Ok(Arc::new(FileStore::new(path)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, EventType, EventData, StateChangeEvent, NodeId};

    #[test]
    fn test_memory_store() {
        let store = MemoryStore::new();
        let session_id = SessionId::new();

        let node = NodeId::new("node1".to_string());
        let event = Event::new(
            session_id,
            EventType::StateChange,
            0,
            EventData::StateChange(StateChangeEvent::new(node, "Idle", "Running")),
        );

        store.store_event(event.clone()).unwrap();

        let loaded = store.load_events(session_id).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].event_type, EventType::StateChange);
    }

    #[test]
    fn test_memory_store_metadata() {
        let store = MemoryStore::new();
        let session_id = SessionId::new();

        let mut metadata = RecordingMetadata::new(session_id);
        metadata = metadata.with_description("Test recording");

        store.save_metadata(&metadata).unwrap();

        let loaded = store.get_metadata(session_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().description, Some("Test recording".to_string()));
    }

    #[test]
    fn test_memory_store_list_sessions() {
        let store = MemoryStore::new();

        let id1 = SessionId::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = SessionId::new();

        let mut metadata = RecordingMetadata::new(id1);
        store.save_metadata(&metadata).unwrap();

        let mut metadata = RecordingMetadata::new(id2);
        store.save_metadata(&metadata).unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_memory_store_clear_session() {
        let store = MemoryStore::new();
        let session_id = SessionId::new();

        let node = NodeId::new("node1".to_string());
        let event = Event::new(
            session_id,
            EventType::StateChange,
            0,
            EventData::StateChange(StateChangeEvent::new(node, "Idle", "Running")),
        );

        store.store_event(event).unwrap();

        let mut metadata = RecordingMetadata::new(session_id);
        store.save_metadata(&metadata).unwrap();

        store.clear_session(session_id).unwrap();

        let events = store.load_events(session_id).unwrap();
        assert_eq!(events.len(), 0);

        let meta = store.get_metadata(session_id).unwrap();
        assert!(meta.is_none());
    }
}
