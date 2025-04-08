use std::net::SocketAddr;
use anyhow::Result;
use dns_server::server::DnsServer;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with more detailed configuration
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    // Create and run DNS server
    let addr: SocketAddr = "0.0.0.0:53".parse()?;
    let server = DnsServer::new(addr).await?;
    
    log::info!("DNS server listening on {}", addr);
    server.run().await?;

    Ok(())
}