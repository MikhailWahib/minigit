use anyhow::{Result, anyhow};
use chrono::{DateTime, Local};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{get_user_email, get_user_name};
use crate::plumbing::object::ObjectId;

#[derive(Debug)]
pub struct Commit {
    tree: ObjectId,
    parent: Option<ObjectId>,
    author: Signature,
    committer: Signature,
    message: String,
}

impl Commit {
    pub fn new(tree: ObjectId, parent: Option<ObjectId>, message: String) -> Result<Self> {
        let name = get_user_name()?;
        let email = get_user_email()?;
        let author = Signature::new(name.clone(), email.clone());
        let committer = Signature::new(name, email);

        Ok(Commit {
            tree,
            parent,
            author,
            committer,
            message,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut content = Vec::new();
        content.extend_from_slice(format!("tree {}\n", self.tree).as_bytes());
        if let Some(parent) = &self.parent {
            content.extend_from_slice(format!("parent {}\n", parent).as_bytes());
        }
        content.extend_from_slice(self.author.to_bytes("author").as_slice());
        content.extend_from_slice(self.committer.to_bytes("committer").as_slice());
        content.extend_from_slice(b"\n");
        content.extend_from_slice(self.message.as_bytes());
        content
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(bytes)?;
        let (header, message) = text
            .split_once("\n\n")
            .ok_or_else(|| anyhow!("Malformed commit: missing header/message separator"))?;

        let mut tree = None;
        let mut parent = None;
        let mut author = None;
        let mut committer = None;

        for line in header.lines() {
            if let Some(value) = line.strip_prefix("tree ") {
                tree = Some(ObjectId::from_hex(value)?);
            } else if let Some(value) = line.strip_prefix("parent ") {
                parent = Some(ObjectId::from_hex(value)?);
            } else if let Some(value) = line.strip_prefix("author ") {
                author = Some(Signature::from_str(value)?);
            } else if let Some(value) = line.strip_prefix("committer ") {
                committer = Some(Signature::from_str(value)?);
            }
        }

        let tree = tree.ok_or_else(|| anyhow!("Malformed commit: missing tree header"))?;
        let author = author.ok_or_else(|| anyhow!("Malformed commit: missing author header"))?;
        let committer =
            committer.ok_or_else(|| anyhow!("Malformed commit: missing committer header"))?;

        Ok(Self {
            tree,
            parent,
            author,
            committer,
            message: message.to_string(),
        })
    }

    pub fn parent(&self) -> Option<ObjectId> {
        self.parent
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Author: {}", self.author)?;
        writeln!(f, "Date:   {}", self.committer.format_git_date())?;
        writeln!(f)?;
        for line in self.message.lines() {
            writeln!(f, "    {line}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Signature {
    name: String,
    email: String,
    timestamp: i64,
    timezone: String,
}

impl Signature {
    fn new(name: String, email: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let timezone = Local::now().format("%z").to_string();

        Signature {
            name,
            email,
            timestamp,
            timezone,
        }
    }

    fn to_bytes(&self, typ: &str) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.extend_from_slice(
            format!(
                "{typ} {} <{}> {} {}\n",
                self.name, self.email, self.timestamp, self.timezone
            )
            .as_bytes(),
        );

        buf
    }

    fn from_str(value: &str) -> Result<Self> {
        let lt_pos = value
            .find('<')
            .ok_or_else(|| anyhow!("Malformed signature: missing '<'"))?;
        let gt_pos = value[lt_pos..]
            .find('>')
            .map(|idx| lt_pos + idx)
            .ok_or_else(|| anyhow!("Malformed signature: missing '>'"))?;

        let name = value[..lt_pos].trim_end().to_string();
        let email = value[lt_pos + 1..gt_pos].to_string();
        let tail = value[gt_pos + 1..].trim_start();
        let mut parts = tail.split_whitespace();

        let timestamp = parts
            .next()
            .ok_or_else(|| anyhow!("Malformed signature: missing timestamp"))?
            .parse::<i64>()?;
        let timezone = parts
            .next()
            .ok_or_else(|| anyhow!("Malformed signature: missing timezone"))?
            .to_string();

        Ok(Self {
            name,
            email,
            timestamp,
            timezone,
        })
    }

    pub fn format_git_date(&self) -> String {
        let offset = self.timezone_offset_seconds().unwrap_or(0);
        let dt = DateTime::from_timestamp(self.timestamp, 0)
            .map(|utc_dt| utc_dt + chrono::Duration::seconds(offset as i64))
            .map(|dt| dt.format("%a %b %-d %H:%M:%S %Y").to_string())
            .unwrap_or_else(|| self.timestamp.to_string());
        format!("{dt} {}", self.timezone)
    }

    fn timezone_offset_seconds(&self) -> Option<i32> {
        if self.timezone.len() != 5 {
            return None;
        }
        let sign = match &self.timezone[0..1] {
            "+" => 1,
            "-" => -1,
            _ => return None,
        };
        let hours = self.timezone[1..3].parse::<i32>().ok()?;
        let minutes = self.timezone[3..5].parse::<i32>().ok()?;
        Some(sign * (hours * 3600 + minutes * 60))
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

#[cfg(test)]
mod tests {
    use super::Commit;

    #[test]
    fn parses_commit_from_raw_bytes() {
        let raw = b"tree aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nparent bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\nauthor Jane Doe <jane@example.com> 1700000000 +0200\ncommitter Jane Doe <jane@example.com> 1700000000 +0200\n\ninitial commit";
        let commit = Commit::from_bytes(raw).expect("parse commit");

        assert_eq!(
            commit.parent().expect("parent").to_hex(),
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        let printed = format!("{commit}");
        assert!(printed.contains("Author: Jane Doe <jane@example.com>"));
        assert!(printed.contains("initial commit"));
    }
}
