use anyhow::{Result, bail};
use std::io::ErrorKind;

pub struct Reader<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    #[cfg(test)]
    fn position(&self) -> usize {
        self.offset
    }

    pub fn is_eof(&self) -> bool {
        self.offset >= self.buf.len()
    }

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

    pub fn read_until_nul(&mut self) -> Result<&'a [u8]> {
        let start = self.offset;
        let rel_nul_pos = self.buf[start..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| anyhow::anyhow!("Malformed data: missing NUL terminator"))?;
        let end = start + rel_nul_pos;
        self.offset = end + 1;
        Ok(&self.buf[start..end])
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

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn reads_numbers_and_skips_bytes() {
        let buf = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut reader = Reader::new(&buf);

        assert_eq!(reader.read_u32().expect("read_u32"), 0x12345678);
        assert_eq!(reader.read_u16().expect("read_u16"), 0x9ABC);
        reader.skip(1).expect("skip");
        assert_eq!(reader.read_exact(1).expect("read_exact"), &[0xF0]);
        assert_eq!(reader.position(), buf.len());
    }

    #[test]
    fn fails_on_unexpected_eof() {
        let buf = [0x00, 0x01, 0x02];
        let mut reader = Reader::new(&buf);

        let err = reader.read_u32().expect_err("should fail on EOF");
        assert!(format!("{err}").contains("unexpected end of file"));
    }
}
