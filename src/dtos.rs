use crate::bytes;

use anyhow::{bail, Result};

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1
pub struct Message {
    pub header: Header, // 12 bytes
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additional: Vec<Record>,
}

impl Message {
    pub fn new_request(question: Question) -> Message {
        Message {
            header: Header {
                id: rand::random(),
                is_response: false,
                opcode: Opcode::StandardQuery,
                authoritative_answer: false,
                truncation: false,
                recursion_desired: true,
                recursion_available: false,
                reserved: 0,
                response_code: ResponseCode::NoError,
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                additional_count: 0,
            },
            questions: vec![question],
            answers: Vec::new(),
            authorities: Vec::new(),
            additional: Vec::new(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);

        self.header.serialize(&mut buf);
        for q in &self.questions {
            q.serialize(&mut buf);
        }
        for r in &self.answers {
            r.serialize(&mut buf);
        }
        for r in &self.authorities {
            r.serialize(&mut buf);
        }
        for r in &self.additional {
            r.serialize(&mut buf);
        }

        buf
    }

    pub fn deserialize(mut byte_reader: bytes::ByteReader) -> Result<Message> {
        let header = Header::deserialize(&mut byte_reader)?;

        let mut questions = Vec::new();
        while questions.len() < header.question_count.into() {
            questions.push(Question::deserialize(&mut byte_reader)?);
        }

        let mut answers = Vec::new();
        while answers.len() < header.answer_count.into() {
            answers.push(Record::deserialize(&mut byte_reader)?);
        }

        let mut authorities = Vec::new();
        while authorities.len() < header.authority_count.into() {
            authorities.push(Record::deserialize(&mut byte_reader)?);
        }

        let mut additional = Vec::new();
        while additional.len() < header.additional_count.into() {
            additional.push(Record::deserialize(&mut byte_reader)?);
        }

        Ok(Message {
            header,
            questions,
            answers,
            authorities,
            additional,
        })
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.header)?;
        for q in &self.questions {
            writeln!(f, "Question:\t{}", q)?;
        }
        for r in &self.answers {
            writeln!(f, "Answer:\t{}", r)?;
        }
        for r in &self.authorities {
            writeln!(f, "Authority:\t{}", r)?;
        }
        for r in &self.additional {
            writeln!(f, "Additional:\t{}", r)?;
        }
        Ok(())
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
// 12 bytes
#[allow(clippy::struct_excessive_bools)] // DNS Header spec has a lot of booleans
pub struct Header {
    pub id: u16,                     // 16 bits
    pub is_response: bool,           // 1 bit
    pub opcode: Opcode,              // 4 bits
    pub authoritative_answer: bool,  // 1 bit
    pub truncation: bool,            // 1 bit
    pub recursion_desired: bool,     // 1 bit
    pub recursion_available: bool,   // 1 bit
    pub reserved: u8,                // 3 bits (reserved)
    pub response_code: ResponseCode, // 4 bits
    pub question_count: u16,         // 16 bits
    pub answer_count: u16,           // 16 bits
    pub authority_count: u16,        // 16 bits
    pub additional_count: u16,       // 16 bits
}

impl Header {
    fn serialize(&self, buf: &mut Vec<u8>) {
        bytes::write_u16(buf, self.id);

        let mut flags = 0_u16;

        if self.is_response {
            flags |= 1 << 15;
        }

        flags |= (self.opcode as u16 & bytes::FOUR_BITS) << 11;

        if self.authoritative_answer {
            flags |= 1 << 10;
        }

        if self.truncation {
            flags |= 1 << 9;
        }

        if self.recursion_desired {
            flags |= 1 << 8;
        }

        if self.recursion_available {
            flags |= 1 << 7;
        }

        flags |= (u16::from(self.reserved) & bytes::THREE_BITS) << 4;

        flags |= self.response_code as u16 & bytes::FOUR_BITS;

        bytes::write_u16(buf, flags);
        bytes::write_u16(buf, self.question_count);
        bytes::write_u16(buf, self.answer_count);
        bytes::write_u16(buf, self.authority_count);
        bytes::write_u16(buf, self.additional_count);
    }

    fn deserialize(byte_reader: &mut bytes::ByteReader) -> Result<Header> {
        let id = byte_reader.read_u16()?;

        let flags = byte_reader.read_u16()?;
        let is_response = bytes::read_bit(flags, 15);
        let opcode = Opcode::from((flags >> 11 & bytes::FOUR_BITS).try_into()?)?;
        let authoritative_answer = bytes::read_bit(flags, 10);
        let truncation = bytes::read_bit(flags, 9);
        let recursion_desired = bytes::read_bit(flags, 8);
        let recursion_available = bytes::read_bit(flags, 7);
        let reserved = (flags >> 4 & bytes::THREE_BITS).try_into()?;
        let response_code = ResponseCode::from((flags & bytes::FOUR_BITS).try_into()?)?;

        let question_count = byte_reader.read_u16()?;
        let answer_count = byte_reader.read_u16()?;
        let authority_count = byte_reader.read_u16()?;
        let additional_count = byte_reader.read_u16()?;

        Ok(Header {
            id,
            is_response,
            opcode,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            reserved,
            response_code,
            question_count,
            answer_count,
            authority_count,
            additional_count,
        })
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_response {
            write!(
                f,
                "Response:\tid: {}, status: {:?}, authoritative answer: {}, truncated: {}",
                self.id, self.response_code, self.authoritative_answer, self.truncation
            )
        } else {
            write!(f, "Request:\tid: {}, opcode: {:?}", self.id, self.opcode)
        }
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2
pub struct Question {
    pub name: String,  // variable bytes
    pub qtype: Type,   // 16 bits
    pub qclass: Class, // 16 bits
}

impl Question {
    fn serialize(&self, buf: &mut Vec<u8>) {
        bytes::write_string(buf, &self.name);
        bytes::write_u16(buf, self.qtype as u16);
        bytes::write_u16(buf, self.qclass as u16);
    }

    fn deserialize(byte_reader: &mut bytes::ByteReader) -> Result<Question> {
        let name = byte_reader.read_string()?;
        let qtype = Type::from(byte_reader.read_u16()?)?;
        let qclass = Class::from(byte_reader.read_u16()?)?;

        Ok(Question {
            name,
            qtype,
            qclass,
        })
    }
}

impl std::fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t({:?} {:?})", self.name, self.qclass, self.qtype)
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3
pub struct Record {
    pub name: String,      // variable bytes
    pub rtype: Type,       // 16 bits
    pub class: Class,      // 16 bits
    pub ttl: u32,          // 32 bits
    pub rdata_length: u16, // 16 bits
    pub rdata: Vec<u8>,    // variable bytes
}

impl Record {
    fn serialize(&self, buf: &mut Vec<u8>) {
        bytes::write_string(buf, &self.name);
        bytes::write_u16(buf, self.rtype as u16);
        bytes::write_u16(buf, self.class as u16);
        bytes::write_u32(buf, self.ttl);
        bytes::write_u16(buf, self.rdata_length);
        buf.extend(&self.rdata);
    }

    fn deserialize(byte_reader: &mut bytes::ByteReader) -> Result<Record> {
        let name = byte_reader.read_string()?;
        let rtype = Type::from(byte_reader.read_u16()?)?;
        let class = Class::from(byte_reader.read_u16()?)?;
        let ttl = byte_reader.read_u32()?;
        let rdata_length = byte_reader.read_u16()?;
        let rdata = byte_reader.read_bytes(rdata_length.into())?;

        Ok(Record {
            name,
            rtype,
            class,
            ttl,
            rdata_length,
            rdata,
        })
    }

    pub fn display_data(&self) -> String {
        if self.rtype == Type::A {
            format!(
                "{}.{}.{}.{}",
                self.rdata[0], self.rdata[1], self.rdata[2], self.rdata[3]
            )
        } else if self.rtype == Type::CName {
            bytes::read_string(&self.rdata)
        } else {
            format!(
                "{}  (Warning: have not implemented proper decoding of type {:?})",
                String::from_utf8_lossy(&self.rdata),
                self.rtype
            )
        }
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}\t({:?} {:?}, ttl: {})",
            self.name,
            self.display_data(),
            self.class,
            self.rtype,
            self.ttl
        )
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
// 4 bits
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Opcode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,
}

impl Opcode {
    fn from(val: u8) -> Result<Opcode> {
        Ok(match val {
            0 => Opcode::StandardQuery,
            1 => Opcode::InverseQuery,
            2 => Opcode::ServerStatusRequest,
            _ => bail!("Unexpected opcode val {}", val),
        })
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
// 4 bits
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
}

impl ResponseCode {
    fn from(val: u8) -> Result<ResponseCode> {
        Ok(match val {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => bail!("Unexpected response code val {}", val),
        })
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2
// 16 bits
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Type {
    A = 1,
    NS = 2,
    MD = 3, // Obsolete - use MX
    MF = 4, // Obsolete - use MX
    CName = 5,
    Soa = 6,
    MB = 7,    // EXPERIMENTAL
    MG = 8,    // EXPERIMENTAL
    MR = 9,    // EXPERIMENTAL
    Null = 10, // EXPERIMENTAL
    Wks = 11,
    Ptr = 12,
    Hinfo = 13,
    Minfo = 14,
    MX = 15,
    Txt = 16,
    Axfr = 252,
    MailB = 253,
    MailA = 254, // Obsolete - see MX
    _ANY_ = 255,
}

impl Type {
    fn from(val: u16) -> Result<Type> {
        Ok(match val {
            1 => Type::A,
            2 => Type::NS,
            3 => Type::MD,
            4 => Type::MF,
            5 => Type::CName,
            6 => Type::Soa,
            7 => Type::MB,
            8 => Type::MG,
            9 => Type::MR,
            10 => Type::Null,
            11 => Type::Wks,
            12 => Type::Ptr,
            13 => Type::Hinfo,
            14 => Type::Minfo,
            15 => Type::MX,
            16 => Type::Txt,
            252 => Type::Axfr,
            253 => Type::MailB,
            254 => Type::MailA,
            255 => Type::_ANY_,
            _ => bail!("Unexpected type val {}", val),
        })
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.4
// 16 bits
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Class {
    IN = 1,
    CS = 2, // Obsolete
    CH = 3,
    HS = 4,
    _ANY_ = 255,
}

impl Class {
    fn from(val: u16) -> Result<Class> {
        Ok(match val {
            1 => Class::IN,
            2 => Class::CS,
            3 => Class::CH,
            4 => Class::HS,
            255 => Class::_ANY_,
            _ => bail!("Unexpected class val {}", val),
        })
    }
}
