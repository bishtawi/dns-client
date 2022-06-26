use anyhow::{bail, Result};

pub const THREE_BITS: u16 = 0b111;
pub const FOUR_BITS: u16 = 0b1111;

const MAX_PACKET_SIZE: usize = 512; // bytes
const POINTER_LIMIT: u8 = 5;

pub struct ByteReader {
    buf: [u8; MAX_PACKET_SIZE],
    size: usize,
    index: usize,
}

impl ByteReader {
    pub fn new() -> ByteReader {
        ByteReader {
            buf: [0; MAX_PACKET_SIZE],
            size: 0,
            index: 0,
        }
    }

    pub fn get_mut_buf(&mut self) -> &mut [u8] {
        &mut self.buf
    }

    pub fn set_size(&mut self, size: usize) {
        assert!(size <= self.buf.len());
        self.size = size;
    }

    pub fn set_index(&mut self, index: usize) {
        assert!(index <= self.size);
        self.index = index;
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.index >= self.size {
            bail!("No more bytes left to read");
        }
        let val = self.buf[self.index];
        self.index += 1;
        Ok(val)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from(self.read_u8()?) << 8 | u16::from(self.read_u8()?))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from(self.read_u8()?) << 24
            | u32::from(self.read_u8()?) << 16
            | u32::from(self.read_u8()?) << 8
            | u32::from(self.read_u8()?))
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(count);

        for _ in 0..count {
            bytes.push(self.read_u8()?);
        }

        Ok(bytes)
    }

    pub fn read_string(&mut self) -> Result<String> {
        self.read_string_helper(0)
    }

    fn read_string_helper(&mut self, count: u8) -> Result<String> {
        if count > POINTER_LIMIT {
            bail!("pointer count exeeded limit");
        }

        let mut string = String::new();
        let mut size = self.read_u8()?;
        let ptr_designator: u8 = 0xC0;
        loop {
            let bits = size & ptr_designator;
            if bits == ptr_designator {
                // Follow pointer
                let pointer = u16::from(size ^ ptr_designator) << 8 | u16::from(self.read_u8()?);
                let saved_index = self.index;
                self.set_index(pointer.into());
                string.push_str(&self.read_string_helper(count + 1)?);
                self.set_index(saved_index);
                return Ok(string);
            } else if bits != 0 {
                bail!("Invalid name bits: {:#b}", bits);
            }

            for _ in 0..size {
                string.push(self.read_u8()? as char);
            }
            size = self.read_u8()?;
            if size == 0 {
                return Ok(string);
            }
            string.push('.');
        }
    }
}

pub fn write_u16(buf: &mut Vec<u8>, val: u16) {
    buf.push((val >> 8).try_into().unwrap());

    buf.push((val & 0xFF).try_into().unwrap());
}

pub fn write_u32(buf: &mut Vec<u8>, val: u32) {
    buf.push((val >> 24).try_into().unwrap());

    buf.push(((val) >> 16 & 0xFF).try_into().unwrap());

    buf.push(((val) >> 8 & 0xFF).try_into().unwrap());

    buf.push((val & 0xFF).try_into().unwrap());
}

pub fn write_string(buf: &mut Vec<u8>, val: &str) {
    val.split('.').for_each(|x| {
        buf.push(x.len().try_into().unwrap());
        x.chars().for_each(|x| {
            buf.push(x as u8);
        });
    });
    buf.push(0);
}

pub fn read_string(buf: &[u8]) -> String {
    let mut string = String::new();
    let mut index = 0;
    let mut size = buf[index];
    index += 1;
    loop {
        for _ in 0..size {
            string.push(buf[index] as char);
            index += 1;
        }
        size = buf[index];
        index += 1;
        if size == 0 {
            return string;
        }
        string.push('.');
    }
}

pub fn read_bit(val: u16, index: u8) -> bool {
    (val >> index) & 1 == 1
}
