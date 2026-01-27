use hex;
use sha1::{Digest, Sha1};
use std::{
    error::Error,
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::Path,
};
use zlib_rs::{DeflateConfig, InflateConfig, compress_bound, compress_slice, decompress_slice};

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
    let obj_dir = Path::new(".minigit/objects").join(dir);
    let obj_path = obj_dir.join(file);

    let mut object = Vec::new();
    object.extend_from_slice(header.as_bytes());
    object.extend_from_slice(&content);

    if !obj_path.exists() {
        create_dir_all(&obj_dir)?;

        let mut compressed_buf = vec![0u8; compress_bound(object.len())];
        let (compressed, _rc) =
            compress_slice(&mut compressed_buf, &mut object, DeflateConfig::default());

        let mut out = File::create(obj_path)?;
        out.write_all(&compressed)?;
    }

    println!("{hex_hash}");
    Ok(hex_hash)
}

pub fn cat_file(hash: String, typ: bool) -> Result<String, Box<dyn Error>> {
    let (dir, file) = hash.split_at(2);

    let mut compressed_buf: Vec<u8> = Vec::new();
    let file_dir = Path::new(".minigit/objects").join(dir);
    let file_path = file_dir.join(file);

    let mut file = File::open(file_path)?;
    file.read_to_end(&mut compressed_buf)?;

    let mut decompressed_buf = vec![0u8; compress_bound(compressed_buf.len())];

    let (decompressed, _rc) = decompress_slice(
        &mut decompressed_buf,
        &mut compressed_buf,
        InflateConfig::default(),
    );

    let body_str;

    if let Some(pos) = decompressed.iter().position(|&b| b == 0) {
        let header = &decompressed[..pos];
        let body = &decompressed[pos + 1..];

        let header_str = std::str::from_utf8(header)?;

        if typ {
            let typ: Vec<&str> = header_str.split(" ").collect();
            let typ = typ[0];
            println!("{typ}");
            return Ok(typ.to_string());
        }

        body_str = std::str::from_utf8(body).unwrap_or("<binary>");
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "no null separator",
        )));
    }

    println!("{body_str}");

    Ok(body_str.to_string())
}
