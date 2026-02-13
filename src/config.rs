// This uses git config for now.

use std::process::Command;

use anyhow::{Result, bail};

pub fn get_user_name() -> Result<String> {
    if let Some(name) = get("user.name") {
        Ok(name)
    } else {
        bail!("no user name found")
    }
}

pub fn get_user_email() -> Result<String> {
    if let Some(email) = get("user.email") {
        Ok(email)
    } else {
        bail!("no user email found")
    }
}

fn get(key: &str) -> Option<String> {
    let output = Command::new("git").args(["config", key]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    Some(value.trim().to_string())
}
