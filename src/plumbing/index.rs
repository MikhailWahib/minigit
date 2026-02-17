use crate::repository::Repository;

use super::reader::Reader;
use anyhow::{Result, bail};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::{fmt, fs, io};

const SIGNATURE: [u8; 4] = *b"DIRC";
const DEFAULT_VERSION: u32 = 2;
const SUPPORTED_VERSIONS: [u32; 3] = [2, 3, 4];

#[derive(Debug, Default)]
pub struct Index {
    version: u32,
    entries: BTreeMap<String, IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            version: DEFAULT_VERSION,
        }
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let index_buf = fs::read(path)?;

        if index_buf.len() < 20 {
            bail!("Index file too short");
        }

        let (content, sha1) = index_buf.split_at(index_buf.len() - 20);

        let mut hasher = Sha1::new();
        hasher.update(content);
        let calculated_sha1: [u8; 20] = hasher.finalize().into();

        if sha1 != calculated_sha1 {
            bail!("Index file checksum mismatch");
        }

        let mut r = Reader::new(content);

        // read the header: first 12 bytes
        let signature: [u8; 4] = r.read_exact(4)?.try_into()?;
        if signature != SIGNATURE {
            bail!("invalid index signature")
        }

        let version = r.read_u32()?;
        if !SUPPORTED_VERSIONS.contains(&version) {
            bail!("unsupported version")
        }

        let entries_count = r.read_u32()?;

        // read next section: entries
        let entries = Self::read_entries(&mut r, entries_count)?;

        Ok(Index { version, entries })
    }

    pub fn read_optional(path: impl AsRef<Path>) -> Result<Option<Self>> {
        match Self::read(path) {
            Ok(idx) => Ok(Some(idx)),
            Err(e)
                if e.downcast_ref::<io::Error>()
                    .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
            {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    pub fn read_or_new(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::read_optional(path)?.unwrap_or_else(Self::new))
    }

    fn read_entries(r: &mut Reader, entries_count: u32) -> Result<BTreeMap<String, IndexEntry>> {
        let mut entries = BTreeMap::new();

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

            entries.insert(
                name.clone(),
                IndexEntry {
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
                },
            );
        }

        Ok(entries)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut content = Vec::new();

        content.write_all(&SIGNATURE)?;
        content.write_all(&self.version.to_be_bytes())?;
        content.write_all(&(self.entries.len() as u32).to_be_bytes())?;

        for entry in &self.entries {
            entry.1.write_to(&mut content)?;
        }

        let mut hasher = Sha1::new();
        hasher.update(&content);
        let sha1: [u8; 20] = hasher.finalize().into();

        content.extend_from_slice(&sha1);
        fs::write(path, content)?;

        Ok(())
    }

    pub fn add(&mut self, path: String, sha1: [u8; 20], mode: u32, work_tree: &Path) -> Result<()> {
        let new_entry = IndexEntry::new(path, sha1, mode, work_tree)?;

        self.entries.insert(new_entry.name.clone(), new_entry);

        Ok(())
    }

    pub fn remove(&mut self, file: String) {
        self.entries.remove(&file);
    }

    pub fn entries(&self) -> Vec<&IndexEntry> {
        self.entries.values().collect()
    }

    pub fn entries_map(&self) -> &BTreeMap<String, IndexEntry> {
        &self.entries
    }

    pub fn get(&self, name: &str) -> Option<&IndexEntry> {
        self.entries.get(name)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct IndexEntry {
    ctime_secs: u32,
    ctime_nano: u32,
    mtime_secs: u32,
    mtime_nano: u32,
    dev: u32,
    ino: u32,
    pub mode: u32,
    uid: u32,
    gid: u32,
    file_size: u32,
    pub sha1: [u8; 20],
    flags: u16,
    pub name: String,
    padding: u8,
}

impl IndexEntry {
    fn new(path: String, sha1: [u8; 20], mode: u32, work_tree: &Path) -> Result<Self> {
        // reject long names for now
        // TODO: handle long entry names
        if path.len() > 0x0FFF {
            bail!("path too long for index entry (max 4095 bytes)");
        }

        let metadata = fs::metadata(work_tree.join(&path))?;
        let ctime_secs = metadata.ctime() as u32;
        let ctime_nano = metadata.ctime_nsec() as u32;
        let mtime = metadata.modified()?.duration_since(UNIX_EPOCH)?;
        let mtime_secs = mtime.as_secs() as u32;
        let mtime_nano = mtime.subsec_nanos();
        let dev = metadata.dev() as u32;
        let ino = metadata.ino() as u32;
        let uid = metadata.uid();
        let gid = metadata.gid();
        let file_size = metadata.size().try_into()?;
        let flags = path.len() as u16;
        let entry_len = 62 + path.len() + 1;
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
            name: path,
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

    pub fn is_modified(&self, repo: &Repository) -> Result<bool> {
        let abs_path = repo.work_tree().join(&self.name);
        let metadata = fs::metadata(&abs_path)?;
        let modified_dur = metadata.modified()?.duration_since(UNIX_EPOCH)?;
        let cur_mtime_secs = modified_dur.as_secs() as u32;
        let cur_mtime_nano = modified_dur.subsec_nanos() as u32;
        let cur_size = metadata.size() as u32;

        // check mtimes
        if self.mtime_secs != cur_mtime_secs || self.mtime_nano != cur_mtime_nano {
            return Ok(true);
        }

        // check size
        if self.file_size != cur_size {
            return Ok(true);
        }

        // check file hash
        let file_content = fs::read(&abs_path)?;
        let header = format!("blob {}\0", file_content.len());
        let mut hasher = Sha1::new();
        hasher.update(header.as_bytes());
        hasher.update(&file_content);
        let cur_sha1: [u8; 20] = hasher.finalize().into();

        if self.sha1 != cur_sha1 {
            return Ok(true);
        }

        Ok(false)
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            writeln!(f, "{}", entry.1)?;
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

#[cfg(test)]
mod tests {
    use super::Index;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir_in;

    #[test]
    fn write_read_roundtrip_preserves_entries() {
        let tmp = tempdir_in(".").expect("tempdir");
        let base = tmp
            .path()
            .file_name()
            .expect("tmp dir name")
            .to_string_lossy();

        let file_a = format!("{base}/a.txt");
        let file_b = format!("{base}/b.txt");
        fs::write(&file_a, "alpha").expect("write file a");
        fs::write(&file_b, "beta").expect("write file b");

        let mut index = Index::new();
        index
            .add(file_b.clone(), [0xBB; 20], 0o100644, Path::new("."))
            .expect("add b");
        index
            .add(file_a.clone(), [0xAA; 20], 0o100644, Path::new("."))
            .expect("add a");

        let index_path = tmp.path().join("index");
        index.write(&index_path).expect("write index");

        let loaded = Index::read(&index_path).expect("read index");
        let entries = loaded.entries();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, file_a);
        assert_eq!(entries[0].sha1, [0xAA; 20]);
        assert_eq!(entries[1].name, file_b);
        assert_eq!(entries[1].sha1, [0xBB; 20]);
    }

    #[test]
    fn rejects_checksum_mismatch() {
        let tmp = tempdir_in(".").expect("tempdir");
        let base = tmp
            .path()
            .file_name()
            .expect("tmp dir name")
            .to_string_lossy();
        let file = format!("{base}/tracked.txt");
        fs::write(&file, "content").expect("write tracked file");

        let mut index = Index::new();
        index
            .add(file, [0x11; 20], 0o100644, Path::new("."))
            .expect("add index entry");

        let path = tmp.path().join("index");
        index.write(&path).expect("write index");

        let mut bytes = fs::read(&path).expect("read index");
        bytes[0] ^= 0xFF;
        fs::write(&path, bytes).expect("rewrite corrupted index");

        let err = Index::read(&path).expect_err("checksum mismatch should fail");
        assert!(format!("{err}").contains("checksum mismatch"));
    }
}
