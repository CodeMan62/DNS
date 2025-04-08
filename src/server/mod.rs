use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::cache::DnsCache;
use crate::resolver::Resolver;
use crate::server::handler::QueryHandler;

pub mod handler;

pub struct DnsServer {
    socket: Arc<UdpSocket>,
    handler: QueryHandler,
}

impl DnsServer {
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        let socket = Arc::new(socket);
        
        // Initialize cache
        let cache = Arc::new(RwLock::new(DnsCache::new()));
        
        // Initialize resolver with cache
        let resolver = Resolver::new(cache);
        
        // Initialize query handler
        let handler = QueryHandler::new(resolver);
        
        Ok(Self { socket, handler })
    }

    pub async fn run(&self) -> Result<()> {
        let mut buf = [0; 512]; // Standard DNS message size
        
        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((size, src)) => {
                    // Process the query
                    log::info!("Received {} bytes from {}", size, src);
                    
                    // Copy the data before spawning the task
                    let query_data = buf[..size].to_vec();
                    let socket = self.socket.clone();
                    let handler = self.handler.clone();
                    
                    // Handle query in a separate task
                    tokio::spawn(async move {
                        if let Err(e) = handler.handle_query(&query_data, src, &socket).await {
                            log::error!("Error handling query: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Error receiving data: {}", e);
                }
            }
        }
    }
}

