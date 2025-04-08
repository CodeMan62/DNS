use crate::protocol::{DnsMessage, DnsHeader};
use crate::resolver::Resolver;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Clone)]
pub struct QueryHandler {
    resolver: Resolver,
}

impl QueryHandler {
    pub fn new(resolver: Resolver) -> Self {
        Self { resolver }
    }

    pub async fn handle_query(&self, query: &[u8], src: SocketAddr, socket: &UdpSocket) -> Result<()> {
        // Parse the incoming DNS query
        let dns_query = match DnsMessage::parse(query) {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Failed to parse DNS query: {}", e);
                return Ok(());
            }
        };

        // Create response message
        let response = self.create_response(dns_query).await?;
        
        // Serialize and send response
        let response_bytes = response.serialize();
        socket.send_to(&response_bytes, src).await?;
        
        Ok(())
    }

    async fn create_response(&self, query: DnsMessage) -> Result<DnsMessage> {
        let mut response = DnsMessage {
            header: DnsHeader {
                id: query.header.id,
                flags: 0x8180, // Standard response, no error
                qdcount: query.header.qdcount,
                ancount: 0,
                nscount: 0,
                arcount: 0,
            },
            questions: query.questions.clone(),
            answers: Vec::new(),
        };

        // Process each question and resolve answers
        for question in &query.questions {
            let answers = self.resolver.resolve(question).await?;
            response.answers.extend(answers);
        }

        response.header.ancount = response.answers.len() as u16;
        
        Ok(response)
    }
} 