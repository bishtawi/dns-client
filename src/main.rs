#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]

mod bytes;
mod dns;
mod dtos;

use anyhow::Result;
use clap::Parser;

const DEFAULT_RESOLVER: &str = "1.1.1.1:53";

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Resolver:\t{}\n", args.resolver);

    let request = dtos::Message::new_request(dtos::Question {
        name: args.domain,
        qtype: args.query_type,
        qclass: args.class,
    });
    println!("{}", request);

    let client = dns::Client::connect(&args.resolver)?;
    let response = client.resolve(&request)?;
    println!("{}", response);

    debug_assert_eq!(response.header.id, request.header.id);
    debug_assert!(response.header.is_response);
    debug_assert_eq!(response.header.response_code, dtos::ResponseCode::NoError);

    Ok(())
}

/// Simple program to resolve domain name to IP
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Domain name to resolve [required]
    #[clap(short, long, value_parser)]
    domain: String,

    /// DNS resource record class
    #[clap(short, long, value_enum, default_value_t = dtos::Class::IN)]
    class: dtos::Class,

    /// Query DNS resource record type
    #[clap(short, long, value_enum, default_value_t = dtos::Type::A)]
    query_type: dtos::Type,

    /// Upstream DNS resolver
    #[clap(short, long, value_parser, default_value_t = DEFAULT_RESOLVER.to_string())]
    resolver: String,
}
