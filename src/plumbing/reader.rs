use anyhow::{Result, bail};
use std::io::ErrorKind;

pub struct Reader<'a> {
    pub buf: &'a [u8],
    pub offset: usize,
}

impl<'a> Reader<'a> {
    fn get_slice(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.offset + n;
        if end > self.buf.len() {
            bail!(ErrorKind::UnexpectedEof);
        }
        let slice = &self.buf[self.offset..end];
        self.offset = end;
        Ok(slice)
    }
    pub fn read_u32(&mut self) -> Result<u32> {
        let slice = self.get_slice(4)?;
        Ok(u32::from_be_bytes(slice.try_into()?))
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let slice = self.get_slice(2)?;
        Ok(u16::from_be_bytes(slice.try_into()?))
    }

    pub fn read_exact(&mut self, n: usize) -> Result<&'a [u8]> {
        self.get_slice(n)
    }

    pub fn skip(&mut self, n: usize) -> Result<()> {
        let end = self.offset + n;
        if end > self.buf.len() {
            bail!(ErrorKind::UnexpectedEof);
        }
        self.offset = end;
        Ok(())
    }
}
