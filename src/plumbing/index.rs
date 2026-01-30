use super::reader::Reader;
use anyhow::{Result, bail};
use std::usize;
use std::{fs::File, io::Read};

const SUPPORTED_VERSIONS: [u32; 3] = [2, 3, 4];

#[derive(Debug, Default)]
pub struct Index {
    entries: Vec<IndexEntry>,
    signature: [u8; 4],
    version: u32,
    entries_count: u32,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, index_path: &str) -> Result<()> {
        match File::open(index_path) {
            Ok(file) => self.read_index(file),
            Err(_) => self.new_and_write(index_path),
        }
    }

    fn read_index(&mut self, mut index_file: File) -> Result<()> {
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

        self.signature = signature;
        self.version = version;
        self.entries_count = entries_count;
        self.entries = entries;

        Ok(())
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
