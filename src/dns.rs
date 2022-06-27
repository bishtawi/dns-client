use anyhow::{anyhow, Result};
use std::net::UdpSocket;

use crate::{bytes, dtos};

const TIMEOUT: u64 = 5; // seconds

pub struct Client {
    conn: UdpSocket,
}

impl Client {
    pub fn connect(resolver: &str) -> Result<Client> {
        let conn = UdpSocket::bind("0.0.0.0:0").map_err(|e| anyhow!("Unable to bind: {}", e))?;
        conn.set_read_timeout(Some(std::time::Duration::from_secs(TIMEOUT)))
            .map_err(|e| anyhow!("Unable to set read timeout: {}", e))?;
        conn.set_write_timeout(Some(std::time::Duration::from_secs(TIMEOUT)))
            .map_err(|e| anyhow!("Unable to set write timeout: {}", e))?;
        conn.connect(resolver)
            .map_err(|e| anyhow!("Unable to connect: {}", e))?;

        Ok(Client { conn })
    }

    pub fn resolve(&self, request: &dtos::Message) -> Result<dtos::Message> {
        self.conn
            .send(&request.serialize())
            .map_err(|e| anyhow!("Unable to send: {}", e))?;

        let mut byte_reader = bytes::ByteReader::new();
        let size = self
            .conn
            .recv(byte_reader.get_mut_buf())
            .map_err(|e| anyhow!("Unable to receive: {}", e))?;
        byte_reader.set_size(size);

        dtos::Message::deserialize(byte_reader)
    }
}
