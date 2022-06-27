#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]

mod bytes;
mod dns;
mod dtos;

use anyhow::{anyhow, Result};
use std::env;

const RESOLVER: &str = "1.1.1.1:53";

fn main() -> Result<()> {
    let domain = parse_args()?;

    println!("Resolver:\t{}\n", RESOLVER);

    let request = dtos::Message::new_request(dtos::Question {
        name: domain,
        qtype: dtos::Type::A,
        qclass: dtos::Class::IN,
    });
    println!("{}", request);

    let client = dns::Client::connect(RESOLVER)?;
    let response = client.resolve(&request)?;
    println!("{}", response);

    debug_assert_eq!(response.header.id, request.header.id);
    debug_assert!(response.header.is_response);
    debug_assert_eq!(response.header.response_code, dtos::ResponseCode::NoError);

    Ok(())
}

fn parse_args() -> Result<String> {
    let args: Vec<String> = env::args().collect();
    args.get(1)
        .map(std::convert::Into::into)
        .ok_or_else(|| anyhow!("Missing argument\n\nUsage: denis <domain>"))
}
