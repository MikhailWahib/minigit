use super::reader::Reader;
use anyhow::{Result, anyhow, bail};
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::usize;
use std::{fmt, fs};
use std::{
    fs::File,
    io::{Read, Write},
};

const SUPPORTED_VERSIONS: [u32; 3] = [2, 3, 4];

#[derive(Debug)]
pub struct Index {
    entries: Vec<IndexEntry>,
    signature: [u8; 4],
    version: u32,
    entries_count: u32,
}

impl Index {
    pub fn read(path: &Path) -> Result<Self> {
        let mut idx_file = File::open(path)?;

        let mut index_buf = Vec::new();
        idx_file.read_to_end(&mut index_buf)?;

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

    pub fn write(&self, path: &str) -> Result<()> {
        let mut idx_file = File::create_new(path)?;

        idx_file.write_all(&self.signature)?;
        idx_file.write_all(&self.version.to_be_bytes())?;
        idx_file.write_all(&self.entries_count.to_be_bytes())?;

        self.write_entries(&mut idx_file)?;
        Ok(())
    }

    fn write_entries<W: Write>(&self, w: &mut W) -> Result<()> {
        for entry in &self.entries {
            entry.write_to(w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
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
    padding: u8,
}

impl IndexEntry {
    fn new(path: &Path, sha1: [u8; 20]) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        let ctime_secs = metadata.ctime() as u32;
        let ctime_nano = metadata.ctime_nsec() as u32;
        let mtime_secs = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as u32;
        let mtime_nano = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_nanos() as u32;
        let dev = metadata.dev() as u32;
        let ino = metadata.ino() as u32;
        let mode = metadata.mode();
        let uid = metadata.uid();
        let gid = metadata.gid();
        let name = path
            .to_str()
            .ok_or_else(|| anyhow!("Path is not valid UTF-8"))?;
        let file_size = metadata.size().try_into()?;
        let flags = name.len() as u16;
        let entry_len = 62 + name.len() + 1;
        let padding = ((8 - (entry_len % 8)) % 8) as u8;

        Ok(IndexEntry {
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
            name: name.into(),
            padding,
        })
    }

    fn write_to<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&self.ctime_secs.to_be_bytes())?;
        w.write_all(&self.ctime_nano.to_be_bytes())?;
        w.write_all(&self.mtime_secs.to_be_bytes())?;
        w.write_all(&self.mtime_nano.to_be_bytes())?;
        w.write_all(&self.dev.to_be_bytes())?;
        w.write_all(&self.ino.to_be_bytes())?;
        w.write_all(&self.mode.to_be_bytes())?;
        w.write_all(&self.uid.to_be_bytes())?;
        w.write_all(&self.gid.to_be_bytes())?;
        w.write_all(&self.file_size.to_be_bytes())?;
        w.write_all(&self.sha1)?;
        w.write_all(&self.flags.to_be_bytes())?;
        w.write_all(self.name.as_bytes())?;
        w.write_all(&[0])?;

        let padding = [0u8; 8];
        w.write_all(&padding[..self.padding as usize])?;

        Ok(())
    }
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
