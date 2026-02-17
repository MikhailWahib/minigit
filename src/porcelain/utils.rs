use anyhow::Result;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

pub fn collect_files(path: &Path, ignore: &[PathBuf], files: &mut Vec<PathBuf>) -> Result<()> {
    if is_ignored(path, ignore) {
        return Ok(());
    }

    if path.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }

    if path.is_dir() {
        let entries: Vec<DirEntry> = path.read_dir()?.flatten().collect();
        for entry in entries {
            collect_files(&entry.path(), ignore, files)?;
        }
    }

    Ok(())
}

fn is_ignored(path: &Path, ignore: &[PathBuf]) -> bool {
    ignore
        .iter()
        .any(|ignored| path == ignored || (ignored.is_dir() && path.starts_with(ignored)))
}

#[cfg(test)]
mod tests {
    use super::collect_files;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn collects_nested_files() {
        let tmp = tempdir().expect("tempdir");
        let root = tmp.path();
        let sub = root.join("src");
        fs::create_dir_all(&sub).expect("create subdir");
        fs::write(root.join("a.txt"), "a").expect("write a");
        fs::write(sub.join("b.txt"), "b").expect("write b");

        let mut files = Vec::new();
        collect_files(root, &[], &mut files).expect("collect files");
        files.sort();

        assert_eq!(files, vec![root.join("a.txt"), sub.join("b.txt")]);
    }

    #[test]
    fn skips_ignored_paths() {
        let tmp = tempdir().expect("tempdir");
        let root = tmp.path();
        let ignored_dir = root.join("target");
        fs::create_dir_all(&ignored_dir).expect("create ignored dir");
        fs::write(root.join("keep.txt"), "keep").expect("write keep");
        fs::write(ignored_dir.join("drop.txt"), "drop").expect("write drop");

        let mut files = Vec::new();
        collect_files(root, &[ignored_dir], &mut files).expect("collect files");
        files.sort();

        assert_eq!(files, vec![root.join("keep.txt")]);
    }
}
