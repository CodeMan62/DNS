use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct DnsMessage {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsResourceRecord>,
}

#[derive(Debug, Clone)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

#[derive(Debug, Clone)]
pub struct DnsQuestion {
    pub name: String,
    pub qtype: u16,
    pub qclass: u16,
}

#[derive(Debug, Clone)]
pub struct DnsResourceRecord {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

impl DnsMessage {
    pub fn parse(buf: &[u8]) -> Result<Self, &'static str> {
        let mut buf = Bytes::copy_from_slice(buf);
        
        let header = DnsHeader::parse(&mut buf)?;
        let mut questions = Vec::new();
        let mut answers = Vec::new();

        for _ in 0..header.qdcount {
            questions.push(DnsQuestion::parse(&mut buf)?);
        }

        for _ in 0..header.ancount {
            answers.push(DnsResourceRecord::parse(&mut buf)?);
        }

        Ok(Self {
            header,
            questions,
            answers,
        })
    }

    pub fn serialize(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(512);
        
        self.header.serialize(&mut buf);
        for question in &self.questions {
            question.serialize(&mut buf);
        }
        for answer in &self.answers {
            answer.serialize(&mut buf);
        }

        buf.freeze()
    }
}

impl DnsHeader {
    fn parse(buf: &mut Bytes) -> Result<Self, &'static str> {
        if buf.remaining() < 12 {
            return Err("Buffer too short for DNS header");
        }

        Ok(Self {
            id: buf.get_u16(),
            flags: buf.get_u16(),
            qdcount: buf.get_u16(),
            ancount: buf.get_u16(),
            nscount: buf.get_u16(),
            arcount: buf.get_u16(),
        })
    }

    fn serialize(&self, buf: &mut BytesMut) {
        buf.put_u16(self.id);
        buf.put_u16(self.flags);
        buf.put_u16(self.qdcount);
        buf.put_u16(self.ancount);
        buf.put_u16(self.nscount);
        buf.put_u16(self.arcount);
    }
}

impl DnsQuestion {
    fn parse(buf: &mut Bytes) -> Result<Self, &'static str> {
        let name = parse_domain_name(buf)?;
        if buf.remaining() < 4 {
            return Err("Buffer too short for DNS question");
        }

        Ok(Self {
            name,
            qtype: buf.get_u16(),
            qclass: buf.get_u16(),
        })
    }

    fn serialize(&self, buf: &mut BytesMut) {
        serialize_domain_name(buf, &self.name);
        buf.put_u16(self.qtype);
        buf.put_u16(self.qclass);
    }
}

impl DnsResourceRecord {
    fn parse(buf: &mut Bytes) -> Result<Self, &'static str> {
        let name = parse_domain_name(buf)?;
        if buf.remaining() < 10 {
            return Err("Buffer too short for DNS resource record");
        }

        let rtype = buf.get_u16();
        let rclass = buf.get_u16();
        let ttl = buf.get_u32();
        let rdlength = buf.get_u16();

        if buf.remaining() < rdlength as usize {
            return Err("Buffer too short for DNS resource record data");
        }

        let mut rdata = vec![0; rdlength as usize];
        buf.copy_to_slice(&mut rdata);

        Ok(Self {
            name,
            rtype,
            rclass,
            ttl,
            rdlength,
            rdata,
        })
    }

    fn serialize(&self, buf: &mut BytesMut) {
        serialize_domain_name(buf, &self.name);
        buf.put_u16(self.rtype);
        buf.put_u16(self.rclass);
        buf.put_u32(self.ttl);
        buf.put_u16(self.rdlength);
        buf.extend_from_slice(&self.rdata);
    }
}

fn parse_domain_name(buf: &mut Bytes) -> Result<String, &'static str> {
    let mut labels = Vec::new();
    
    loop {
        if buf.remaining() == 0 {
            return Err("Unexpected end of buffer while parsing domain name");
        }

        let length = buf.get_u8() as usize;
        if length == 0 {
            break;
        }

        if length > 63 {
            return Err("Label too long");
        }

        if buf.remaining() < length {
            return Err("Buffer too short for domain name label");
        }

        let mut label = vec![0; length];
        buf.copy_to_slice(&mut label);
        labels.push(String::from_utf8(label).map_err(|_| "Invalid UTF-8 in domain name")?);
    }

    Ok(labels.join("."))
}

fn serialize_domain_name(buf: &mut BytesMut, name: &str) {
    for label in name.split('.') {
        buf.put_u8(label.len() as u8);
        buf.extend_from_slice(label.as_bytes());
    }
    buf.put_u8(0);
} 