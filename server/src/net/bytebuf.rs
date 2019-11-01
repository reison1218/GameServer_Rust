use super::*;

pub enum ReadError {
    None,
    NotEnough,
    Zero,
}

#[derive(Clone)]
pub struct ByteBuf {
    bytes: Vec<u8>,
    index: usize,
}

impl From<&[u8]> for ByteBuf {
    fn from(bytes: &[u8]) -> Self {
        let mut byte = ByteBuf::new();
        for i in bytes {
            byte.push(*i);
        }
        byte
    }
}

impl ByteBuf {
    pub fn new() -> ByteBuf {
        ByteBuf {
            bytes: Vec::new(),
            index: 0,
        }
    }

    pub fn to_string(&self) -> String {
        let v = self.bytes.clone();
        let s = String::from_utf8(v);
        s.unwrap()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) -> usize {
        self.index = index;
        self.index
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..]
    }

    pub fn push(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    pub fn push_array(&mut self, bytes: &[u8]) {
        for i in bytes {
            self.bytes.push(*i);
        }
    }

    pub fn push_str(&mut self, _str: &str) {
        for i in _str.as_bytes() {
            self.bytes.push(*i);
        }
    }

    pub fn push_u32(&mut self, i: u32) {
        unsafe {
            let mut byte = transmute::<u32, [u8; 4]>(i);

            for i in &byte {
                self.bytes.push(*i);
            }
        }
    }

    pub fn push_u16(&mut self, i: u16) {
        unsafe {
            let byte = transmute::<u16, [u8; 2]>(i);

            for i in &byte {
                self.bytes.push(*i);
            }
        }
    }

    pub fn push_u64(&mut self, i: u64) {
        unsafe {
            let byte = transmute::<u64, [u8; 8]>(i);

            for i in &byte {
                self.bytes.push(*i);
            }
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.bytes.push(c as u8);
    }

    pub fn push_string(&mut self, s: String) {
        for i in s.as_bytes() {
            self.bytes.push(*i);
        }
    }

    pub fn read_u32(&mut self) -> ByteBufResult<u32, &str> {
        if self.bytes.len() - self.index < 4 {
            return Err("NotEnough");
        }
        let b = &self.bytes[self.index..=self.index + 3];
        self.index += 4;
        let mut int = 0;
        unsafe {
            let mut byte: [u8; 4] = [0; 4];

            for i in 0..3 {
                byte[i] = b[i];
            }
            int = transmute::<[u8; 4], u32>(byte);
        }
        Ok(int)
    }

    pub fn read_u16(&mut self) -> ByteBufResult<u16, &str> {
        if self.bytes.len() - self.index < 2 {
            return Err("NotEnough");
        }

        let b = &self.bytes[self.index..=self.index + 1];
        self.index += 2;
        let mut short = 0;
        unsafe {
            let mut byte: [u8; 2] = [0; 2];

            for i in 0..1 {
                byte[i] = b[i];
            }
            short = transmute::<[u8; 2], u16>(byte);
        }
        Ok(short)
    }

    pub fn read_u64(&mut self) -> ByteBufResult<u64, &str> {
        if self.bytes.len() - self.index < 8 {
            return Err("NotEnough");
        }

        let b = &self.bytes[self.index..=self.index + 7];
        self.index += 8;
        let mut long = 0;
        unsafe {
            let mut byte: [u8; 8] = [0; 8];

            for i in 0..7 {
                byte[i] = b[i];
            }
            long = transmute::<[u8; 8], u64>(byte);
        }
        Ok(long)
    }

    pub fn read_u8(&mut self) -> ByteBufResult<u8, &str> {
        if self.bytes.len() - self.index < 1 {
            return Err("NotEnough");
        }

        let b = self.bytes.get(self.index).unwrap();
        self.index += 1;
        Ok(*b)
    }

    pub fn read_bytes(&mut self) -> ByteBufResult<&[u8], &str> {
        let v = &self.bytes[self.index..];
        self.index = self.bytes.len() - 1;
        Ok(v)
    }
}
