use super::reader::Reader;
use anyhow::{Result, bail};
use std::fmt;
use std::usize;
use std::{fs::File, io::Read};

const SUPPORTED_VERSIONS: [u32; 3] = [2, 3, 4];

#[derive(Debug)]
pub struct Index {
    entries: Vec<IndexEntry>,
    signature: [u8; 4],
    version: u32,
    entries_count: u32,
}

impl Index {
    pub fn open(index_path: &str) -> Result<Self> {
        let file = File::open(index_path)?;
        Self::read_index(file)
    }

    fn read_index(mut index_file: File) -> Result<Self> {
        let mut index_buf = Vec::new();
        index_file.read_to_end(&mut index_buf)?;

        let mut r = Reader {
            buf: &index_buf,
            offset: 0,
        };

        // read the header: first 12 bytes
        let signature = r.read_exact(4)?.try_into()?;
        if signature != *b"DIRC" {
            bail!("invalid index signature")
        }

        let version = r.read_u32()?;
        if !SUPPORTED_VERSIONS.contains(&version) {
            bail!("unsupported version")
        }

        let entries_count = r.read_u32()?;

        // read next section: entries
        let entries = Self::read_entries(&mut r, entries_count)?;

        Ok(Index {
            entries,
            signature,
            version,
            entries_count,
        })
    }

    fn read_entries(r: &mut Reader, entries_count: u32) -> Result<Vec<IndexEntry>> {
        let mut entries = Vec::with_capacity(entries_count as usize);

        for _ in 0..entries_count as usize {
            let ctime_secs = r.read_u32()?;
            let ctime_nano = r.read_u32()?;
            let mtime_secs = r.read_u32()?;
            let mtime_nano = r.read_u32()?;
            let dev = r.read_u32()?;
            let ino = r.read_u32()?;
            let mode = r.read_u32()?;
            let uid = r.read_u32()?;
            let gid = r.read_u32()?;
            let file_size = r.read_u32()?;

            let sha1 = r.read_exact(20)?.try_into()?;
            let flags = r.read_u16()?;

            // TODO: handle long names
            let name_len = (flags & 0x0FFF) as usize;

            let name_bytes = r.read_exact(name_len)?;
            r.skip(1)?; // skip the NUL

            let name = String::from_utf8(name_bytes.into())?;

            let entry_len = 62 + name.len() + 1;
            let padding = (8 - (entry_len % 8)) % 8;
            r.skip(padding)?;

            entries.push(IndexEntry {
                ctime_secs,
                ctime_nano,
                mtime_secs,
                mtime_nano,
                dev,
                ino,
                mode,
                uid,
                gid,
                file_size,
                sha1,
                flags,
                name,
                padding: padding.try_into()?,
            });
        }

        Ok(entries)
    }

    fn new_and_write(&self, path: &str) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default)]
struct IndexEntry {
    ctime_secs: u32,
    ctime_nano: u32,
    mtime_secs: u32,
    mtime_nano: u32,
    dev: u32,
    ino: u32,
    mode: u32,
    uid: u32,
    gid: u32,
    file_size: u32,
    sha1: [u8; 20],
    flags: u16,
    name: String,
    padding: u64,
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            writeln!(f, "{}", entry)?;
        }
        Ok(())
    }
}

impl fmt::Display for IndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_sha1 = self
            .sha1
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        let stage = (self.flags >> 12) & 0x3;

        writeln!(f, "{}", self.name)?;
        writeln!(f, "  ctime: {}:{}", self.ctime_secs, self.ctime_nano)?;
        writeln!(f, "  mtime: {}:{}", self.mtime_secs, self.mtime_nano)?;
        writeln!(f, "  dev: {}\tino: {}", self.dev, self.ino)?;
        writeln!(f, "  uid: {}\tgid: {}", self.uid, self.gid)?;
        writeln!(f, "  size: {}\tflags: {:x}", self.file_size, stage)?;
        write!(f, "  mode: {:o}\tsha1: {}", self.mode, hex_sha1)
    }
}
