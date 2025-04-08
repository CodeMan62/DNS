use crate::protocol::{DnsQuestion, DnsResourceRecord};
use crate::cache::DnsCache;
use anyhow::Result;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Resolver {
    cache: Arc<RwLock<DnsCache>>,
}

impl Resolver {
    pub fn new(cache: Arc<RwLock<DnsCache>>) -> Self {
        Self { cache }
    }

    pub async fn resolve(&self, question: &DnsQuestion) -> Result<Vec<DnsResourceRecord>> {
        // Check cache first
        if let Some(records) = self.check_cache(question).await {
            return Ok(records);
        }

        // Perform resolution based on query type
        match question.qtype {
            1 => self.resolve_a_record(question).await,
            28 => self.resolve_aaaa_record(question).await,
            5 => self.resolve_cname_record(question).await,
            _ => Ok(Vec::new()), // Unsupported record type
        }
    }

    async fn check_cache(&self, question: &DnsQuestion) -> Option<Vec<DnsResourceRecord>> {
        let cache = self.cache.read().await;
        cache.get(&question.name, question.qtype)
    }

    async fn resolve_a_record(&self, question: &DnsQuestion) -> Result<Vec<DnsResourceRecord>> {
        // In a real implementation, you would look up the A record from zone files
        // or forward to another DNS server. For simplicity, we'll return a dummy record.
        
        let record = DnsResourceRecord {
            name: question.name.clone(),
            rtype: 1, // A record
            rclass: 1, // IN (Internet)
            ttl: 3600,
            rdlength: 4,
            rdata: Ipv4Addr::new(192, 168, 1, 1).octets().to_vec(),
        };

        // Store in cache
        let mut cache = self.cache.write().await;
        cache.store(question.name.clone(), 1, vec![record.clone()]);

        Ok(vec![record])
    }

    async fn resolve_aaaa_record(&self, question: &DnsQuestion) -> Result<Vec<DnsResourceRecord>> {
        // Similar implementation for AAAA records
        let ipv6_addr = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        let record = DnsResourceRecord {
            name: question.name.clone(),
            rtype: 28, // AAAA record
            rclass: 1, // IN (Internet)
            ttl: 3600,
            rdlength: 16,
            rdata: ipv6_addr.octets().to_vec(),
        };

        let mut cache = self.cache.write().await;
        cache.store(question.name.clone(), 28, vec![record.clone()]);

        Ok(vec![record])
    }

    async fn resolve_cname_record(&self, question: &DnsQuestion) -> Result<Vec<DnsResourceRecord>> {
        // CNAME implementation
        let cname = "example.com";
        let cname_bytes = serialize_domain_name(cname);
        
        let record = DnsResourceRecord {
            name: question.name.clone(),
            rtype: 5, // CNAME record
            rclass: 1, // IN (Internet)
            ttl: 3600,
            rdlength: cname_bytes.len() as u16,
            rdata: cname_bytes,
        };

        let mut cache = self.cache.write().await;
        cache.store(question.name.clone(), 5, vec![record.clone()]);

        Ok(vec![record])
    }
}

fn serialize_domain_name(name: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for label in name.split('.') {
        result.push(label.len() as u8);
        result.extend_from_slice(label.as_bytes());
    }
    result.push(0);
    result
} 