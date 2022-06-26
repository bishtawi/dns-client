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

    let resolver = format!("{}:{}", RESOLVER, PORT);
    println!("Resolver:\t{}\n", resolver);

    let question = dtos::Question {
        name: args[1].to_string(),
        qtype: dtos::Type::A,
        qclass: dtos::Class::IN,
    };
    let request = dtos::Message::new_request(question);
    println!("{}", request);

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| anyhow!("Unable to bind: {}", e))?;
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(TIMEOUT)))
        .map_err(|e| anyhow!("Unable to set read timeout: {}", e))?;
    socket
        .set_write_timeout(Some(std::time::Duration::from_secs(TIMEOUT)))
        .map_err(|e| anyhow!("Unable to set write timeout: {}", e))?;
    socket
        .connect(resolver)
        .map_err(|e| anyhow!("Unable to connect: {}", e))?;
    socket
        .send(&request.serialize())
        .map_err(|e| anyhow!("Unable to send: {}", e))?;

    let mut byte_reader = bytes::ByteReader::new();
    let size = socket
        .recv(byte_reader.get_mut_buf())
        .map_err(|e| anyhow!("Unable to receive: {}", e))?;
    byte_reader.set_size(size);

    let response = dtos::Message::deserialize(byte_reader)?;
    println!("{}", response);

    debug_assert_eq!(response.header.id, request.header.id);
    debug_assert!(response.header.is_response);
    debug_assert_eq!(response.header.response_code, dtos::ResponseCode::NoError);

    Ok(())
}
