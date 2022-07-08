# Denis: a simple DNS to IP resolver

Reference: https://datatracker.ietf.org/doc/html/rfc1035

## Development

Written in Rust, development follows standard Rust practices:

- Build: `cargo build`
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Usage

```
$ denis --help
denis 0.1.0
Simple program to resolve domain name to IP

USAGE:
    denis [OPTIONS] --domain <DOMAIN>

OPTIONS:
    -c, --class <CLASS>              DNS resource record class [default: in] [possible values: in,
                                     cs, ch, hs, any]
    -d, --domain <DOMAIN>            Domain name to resolve [required]
    -h, --help                       Print help information
    -q, --query-type <QUERY_TYPE>    Query DNS resource record type [default: a] [possible values:
                                     a, ns, md, mf, c-name, soa, mb, mg, mr, null, wks, ptr, hinfo,
                                     minfo, mx, txt, axfr, mail-b, mail-a, any]
    -r, --resolver <RESOLVER>        Upstream DNS resolver [default: 1.1.1.1:53]
    -V, --version                    Print version information
```

## Example

```
$ denis --domain www.reddit.com
Resolver:       1.1.1.1:53

Request:        id: 17644, opcode: StandardQuery
Question:       www.reddit.com  (IN A)

Response:       id: 17644, status: NoError, authoritative answer: false, truncated: false
Question:       www.reddit.com  (IN A)
Answer: www.reddit.com  reddit.map.fastly.net   (IN CName, ttl: 5069)
Answer: reddit.map.fastly.net   151.101.65.140  (IN A, ttl: 4)
Answer: reddit.map.fastly.net   151.101.129.140 (IN A, ttl: 4)
Answer: reddit.map.fastly.net   151.101.193.140 (IN A, ttl: 4)
Answer: reddit.map.fastly.net   151.101.1.140   (IN A, ttl: 4)
```
