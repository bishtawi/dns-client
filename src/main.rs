#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]

mod bytes;
mod dtos;

use anyhow::{anyhow, bail, Result};
use std::env;
use std::net::UdpSocket;

const RESOLVER: &str = "1.1.1.1";
const PORT: u16 = 53;
const TIMEOUT: u64 = 5; // seconds

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        bail!("Missing argument\n\nUsage: denis <domain>")
    }

    let query = &args[1];
    let request = dtos::Message::new_request(query);
    let data = request.serialize();

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| anyhow!("Unable to bind: {}", e))?;
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(TIMEOUT)))
        .map_err(|e| anyhow!("Unable to set read timeout: {}", e))?;
    socket
        .set_write_timeout(Some(std::time::Duration::from_secs(TIMEOUT)))
        .map_err(|e| anyhow!("Unable to set write timeout: {}", e))?;
    socket
        .connect(format!("{}:{}", RESOLVER, PORT))
        .map_err(|e| anyhow!("Unable to connect: {}", e))?;
    socket
        .send(&data)
        .map_err(|e| anyhow!("Unable to send: {}", e))?;

    let mut byte_reader = bytes::ByteReader::new();
    let size = socket
        .recv(byte_reader.get_mut_buf())
        .map_err(|e| anyhow!("Unable to receive: {}", e))?;
    byte_reader.set_size(size);

    let response = dtos::Message::deserialize(byte_reader)?;

    response.answers.iter().for_each(|r| {
        println!("{}: {}", r.name, r.display_data());
    });

    assert_eq!(response.header.id, request.header.id);
    assert!(response.header.is_response);
    assert_eq!(response.header.response_code, dtos::ResponseCode::NoError);

    Ok(())
}
