use anyhow::Result;
use chrono::Local;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{get_user_email, get_user_name};

pub struct Commit {
    tree: String,
    parent: Option<String>,
    author: Signature,
    committer: Signature,
    message: String,
}

impl Commit {
    pub fn new(tree: String, parent: Option<String>, message: String) -> Result<Self> {
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
}

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
}
