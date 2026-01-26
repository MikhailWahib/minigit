use std::{error::Error, fs::File, io::Read, path::Path};
use zlib_rs::{InflateConfig, compress_bound, decompress_slice};

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
