use anyhow::{Context, Result, anyhow};
use hex;
use sha1::{Digest, Sha1};
use std::{
    fs::{File, create_dir_all},
    io::{self, Read, Write},
    path::Path,
};
use zlib_rs::{DeflateConfig, InflateConfig, compress_bound, compress_slice, decompress_slice};

use super::index::Index;

pub fn hash_object(object_path: &str, write: &bool) -> Result<String> {
    let mut file = File::open(&object_path)
        .with_context(|| format!("Failed to open object at {}", object_path))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let header = format!("blob {}\0", content.len());

    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(&content);
    let hash = hasher.finalize();
    let hex_hash = hex::encode(hash);

    if !write {
        return Ok(hex_hash);
    }

    let (dir, file_name) = hex_hash.split_at(2);
    let obj_dir = Path::new(".minigit/objects").join(dir);
    let obj_path = obj_dir.join(file_name);

    let mut object = Vec::new();
    object.extend_from_slice(header.as_bytes());
    object.extend_from_slice(&content);

    if !obj_path.exists() {
        create_dir_all(&obj_dir)?;

        let mut compressed_buf = vec![0u8; compress_bound(object.len())];
        let (compressed, _rc) =
            compress_slice(&mut compressed_buf, &mut object, DeflateConfig::default());

        let mut out = File::create(&obj_path)?;
        out.write_all(compressed)?;
    }

    println!("{hex_hash}");
    Ok(hex_hash)
}

pub fn cat_file(hash: &str, typ: &bool) -> Result<String> {
    let (dir, file_name) = hash.split_at(2);

    let mut compressed_buf: Vec<u8> = Vec::new();
    let file_dir = Path::new(".minigit/objects").join(dir);
    let file_path = file_dir.join(file_name);

    let mut file = File::open(&file_path)
        .with_context(|| format!("Could not find object with hash {}", hash))?;
    file.read_to_end(&mut compressed_buf)?;

    let mut decompressed_buf = vec![0u8; compress_bound(1024 * 16)];

    let (decompressed, _rc) = decompress_slice(
        &mut decompressed_buf,
        &mut compressed_buf,
        InflateConfig::default(),
    );

    if let Some(pos) = decompressed.iter().position(|&b| b == 0) {
        let header = &decompressed[..pos];
        let body = &decompressed[pos + 1..];

        let header_str = std::str::from_utf8(header)?;

        if *typ {
            let typ_name = header_str
                .split(' ')
                .next()
                .ok_or_else(|| anyhow!("Malformed object header"))?;
            println!("{typ_name}");
            return Ok(typ_name.to_string());
        }

        let body_str = std::str::from_utf8(body).unwrap_or("<binary>");
        println!("{body_str}");
        Ok(body_str.to_string())
    } else {
        Err(anyhow!("Invalid object data: no null separator found"))
    }
}

pub fn ls_files(index_path: impl AsRef<Path>) -> Result<()> {
    match Index::read(index_path.as_ref().to_str().unwrap()) {
        Ok(idx) => println!("{}", idx),
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            // ignore missing index
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

pub fn update_index(mode: &String, object: &String, file: &String) -> Result<()> {
    let mut idx = match Index::read(".minigit/index") {
        Ok(i) => i,
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            Index::new()
        }
        Err(e) => return Err(e),
    };

    let mode: u32 = mode.parse()?;
    let mut sha1 = [0u8; 20];
    hex::decode_to_slice(object, &mut sha1)?;

    idx.add(file, sha1, mode)?;
    idx.write(".minigit/index")?;

    Ok(())
}
