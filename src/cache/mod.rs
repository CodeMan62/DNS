use crate::protocol::DnsResourceRecord;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct DnsCache {
    // Key format: (domain_name, record_type)
    entries: HashMap<(String, u16), CacheEntry>,
}

struct CacheEntry {
    records: Vec<DnsResourceRecord>,
    expiry: Instant,
}

impl DnsCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn store(&mut self, name: String, record_type: u16, records: Vec<DnsResourceRecord>) {
        // Find the minimum TTL among all records
        let min_ttl = records.iter()
            .map(|r| r.ttl)
            .min()
            .unwrap_or(300); // Default to 5 minutes if no records
        
        let expiry = Instant::now() + Duration::from_secs(min_ttl as u64);
        
        self.entries.insert(
            (name, record_type),
            CacheEntry { records, expiry }
        );
    }

    pub fn get(&self, name: &str, record_type: u16) -> Option<Vec<DnsResourceRecord>> {
        let key = (name.to_string(), record_type);
        
        if let Some(entry) = self.entries.get(&key) {
            if entry.expiry > Instant::now() {
                return Some(entry.records.clone());
            }
        }
        
        None
    }

    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| entry.expiry > now);
    }
} 