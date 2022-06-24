// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1
pub struct Message {
    header: Header, // 12 bytes
    questions: Vec<Question>,
    answers: Vec<Record>,
    authorities: Vec<Record>,
    additional: Vec<Record>,
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
// 12 bytes
#[allow(clippy::struct_excessive_bools)]
pub struct Header {
    id: u16,                     // 16 bits
    is_response: bool,           // 1 bit
    opcode: Opcode,              // 4 bits
    authoritative_answer: bool,  // 1 bit
    truncation: bool,            // 1 bit
    recursion_desired: bool,     // 1 bit
    recursion_available: bool,   // 1 bit
    reserved: u8,                // 3 bits (reserved)
    response_code: ResponseCode, // 4 bits
    question_count: u16,         // 16 bits
    answer_count: u16,           // 16 bits
    authority_count: u16,        // 16 bits
    additional_count: u16,       // 16 bits
}

// 4 bit
pub enum Opcode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,
}

// 4 bit
pub enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2
pub struct Question {
    name: Vec<u8>,  // variable bytes
    qtype: QType,   // 16 bits
    qclass: QClass, // 16 bits
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2
// 16 bits
pub enum QType {
    A = 1,
    Ns = 2,
    Md = 3, // Obsolete - use MX
    Mf = 4, // Obsolete - use MX
    Cname = 5,
    Soa = 6,
    MB = 7,    // EXPERIMENTAL
    MG = 8,    // EXPERIMENTAL
    MR = 9,    // EXPERIMENTAL
    Null = 10, // EXPERIMENTAL
    Wks = 11,
    Ptr = 12,
    Hinfo = 13,
    Minfo = 14,
    Mx = 15,
    Txt = 16,
    Axfr = 252,
    Mailb = 253,
    Maila = 254, // Obsolete - see MX
    _All_ = 255,
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.4
// 16 bits
pub enum QClass {
    IN = 1,
    CS = 2, // Obsolete
    CH = 3,
    HS = 4,
    _ALL_ = 255,
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3
// TODO: Probably change record into an enum so we can better represent the rdata contents?
pub struct Record {
    name: Vec<u8>,  // variable bytes
    rtype: QType,   // 16 bits
    class: QClass,  // 16 bits
    ttl: u32,       // 32 bits
    rdlength: u16,  // 16 bits
    rdata: Vec<u8>, // variable bytes
}
