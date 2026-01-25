use std::{
    error::Error,
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::Path,
};

use hex;
use sha1::{Digest, Sha1};
use zlib_rs::{DeflateConfig, compress_bound, compress_slice};

pub fn hash_object(object_path: String, write: bool) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(object_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let header = format!("blob {}\0", content.len());

    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(&content);
    let hash = hasher.finalize();
    let hex_hash = hex::encode(&hash);

    if !write {
        return Ok(hex_hash);
    }

    let (dir, file) = hex_hash.split_at(2);
    let obj_dir = Path::new(".git/objects").join(dir);
    let obj_path = obj_dir.join(file);

    let mut object = Vec::new();
    object.extend_from_slice(header.as_bytes());
    object.extend_from_slice(&content);

    if !obj_path.exists() {
        create_dir_all(&obj_dir)?;

        let mut compressed_buf = vec![0u8; compress_bound(object.len())];
        let (compressed, _rc) = compress_slice(
            &mut compressed_buf,
            object.as_ref(),
            DeflateConfig::default(),
        );

        let mut out = File::create(obj_path)?;
        out.write_all(&compressed)?;
    }

    println!("{hex_hash}");
    Ok(hex_hash)
}
