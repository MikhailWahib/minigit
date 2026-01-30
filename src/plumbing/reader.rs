use anyhow::{Result, bail};
use std::io::ErrorKind;

pub struct Reader<'a> {
    pub buf: &'a [u8],
    pub offset: usize,
}

impl<'a> Reader<'a> {
    pub fn read_u8(&mut self) -> Result<u8> {
        let bytes: [u8; 1] = self.buf[self.offset..self.offset + 1].try_into()?;
        self.offset += 1;
        Ok(u8::from_be_bytes(bytes))
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes: [u8; 2] = self.buf[self.offset..self.offset + 2].try_into()?;
        self.offset += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes: [u8; 4] = self.buf[self.offset..self.offset + 4].try_into()?;
        self.offset += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        let bytes: [u8; N] = self.buf[self.offset..self.offset + N].try_into()?;
        self.offset += N;
        Ok(bytes)
    }

    pub fn read_exact(&mut self, n: usize) -> Result<Vec<u8>> {
        let end = self.offset + n;
        if end > self.buf.len() {
            bail!(ErrorKind::UnexpectedEof);
        }

        let bytes = self.buf[self.offset..end].to_vec();
        self.offset = end;
        Ok(bytes)
    }

    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.read_exact(n).map(|_| ())
    }
}
