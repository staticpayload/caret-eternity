// Caret Core - Metadata types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::collections::HashMap;
use std::sync::Arc;

/// A typed value in metadata
#[derive(Clone, Debug, PartialEq)]
pub enum MetadataValue {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Unsigned integer value
    Uint(u64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Byte array value
    Bytes(Vec<u8>),
    /// Array of values
    Array(Vec<MetadataValue>),
    /// Map of values
    Map(HashMap<String, MetadataValue>),
}

impl MetadataValue {
    /// Create a null value
    pub fn null() -> Self {
        Self::Null
    }

    /// Create a boolean value
    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Create an integer value
    pub fn int(value: i64) -> Self {
        Self::Int(value)
    }

    /// Create an unsigned integer value
    pub fn uint(value: u64) -> Self {
        Self::Uint(value)
    }

    /// Create a float value
    pub fn float(value: f64) -> Self {
        Self::Float(value)
    }

    /// Create a string value
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Create a bytes value
    pub fn bytes(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }

    /// Create an array value
    pub fn array(values: Vec<MetadataValue>) -> Self {
        Self::Array(values)
    }

    /// Create a map value
    pub fn map(map: HashMap<String, MetadataValue>) -> Self {
        Self::Map(map)
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Try to get as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as an integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            Self::Uint(u) => i64::try_from(*u).ok(),
            _ => None,
        }
    }

    /// Try to get as an unsigned integer
    pub fn as_uint(&self) -> Option<u64> {
        match self {
            Self::Uint(u) => Some(*u),
            Self::Int(i) if *i >= 0 => Some(*i as u64),
            _ => None,
        }
    }

    /// Try to get as a float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Int(i) => Some(*i as f64),
            Self::Uint(u) => Some(*u as f64),
            _ => None,
        }
    }

    /// Try to get as a string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as bytes
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            _ => None,
        }
    }
}

impl From<bool> for MetadataValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for MetadataValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<u64> for MetadataValue {
    fn from(value: u64) -> Self {
        Self::Uint(value)
    }
}

impl From<f64> for MetadataValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<String> for MetadataValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MetadataValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Vec<u8>> for MetadataValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

/// Metadata associated with a packet
///
/// Each packet can carry arbitrary key-value metadata
/// for application-specific information.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Metadata {
    inner: HashMap<String, MetadataValue>,
}

impl Metadata {
    /// Create empty metadata
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Check if metadata is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.inner.get(key)
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<MetadataValue>) {
        self.inner.insert(key.into(), value.into());
    }

    /// Remove a value by key
    pub fn remove(&mut self, key: &str) -> Option<MetadataValue> {
        self.inner.remove(key)
    }

    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = (&str, &MetadataValue)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Merge another metadata into this one
    pub fn merge(&mut self, other: Metadata) {
        self.inner.extend(other.inner);
    }

    /// Clone into an Arc for cheap sharing
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

// Implement IntoIterator for metadata
impl IntoIterator for Metadata {
    type Item = (String, MetadataValue);
    type IntoIter = std::collections::hash_map::IntoIter<String, MetadataValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_value_conversions() {
        assert_eq!(MetadataValue::bool(true).as_bool(), Some(true));
        assert_eq!(MetadataValue::int(-42).as_int(), Some(-42));
        assert_eq!(MetadataValue::uint(42).as_uint(), Some(42));
        assert_eq!(MetadataValue::float(3.14).as_float(), Some(3.14));
        assert_eq!(MetadataValue::string("test").as_str(), Some("test"));
    }

    #[test]
    fn test_metadata_operations() {
        let mut meta = Metadata::new();
        assert!(meta.is_empty());

        meta.insert("key1", "value1");
        meta.insert("key2", 42i64);

        assert_eq!(meta.len(), 2);
        assert_eq!(meta.get("key1"), Some(&MetadataValue::string("value1")));
        assert_eq!(meta.get("key2"), Some(&MetadataValue::int(42)));
        assert_eq!(meta.get("key3"), None);

        meta.remove("key1");
        assert_eq!(meta.len(), 1);
    }

    #[test]
    fn test_metadata_merge() {
        let mut meta1 = Metadata::new();
        meta1.insert("a", 1i64);
        meta1.insert("b", 2i64);

        let mut meta2 = Metadata::new();
        meta2.insert("b", 3i64);
        meta2.insert("c", 4i64);

        meta1.merge(meta2);
        assert_eq!(meta1.len(), 3);
        assert_eq!(meta1.get("b"), Some(&MetadataValue::int(3)));
    }
}
