use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::objects::GitObject;

/// 将对象压缩并写入 .minigit/objects 目录
pub fn write_object(obj: &dyn GitObject) -> Result<String> {
    let bytes = obj.to_bytes();
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    let hex_hash = hex::encode(hash);

    let dir = format!(".minigit/objects/{}", &hex_hash[..2]);
    let path = format!("{}/{}", dir, &hex_hash[2..]);

    if !Path::new(&path).exists() {
        fs::create_dir_all(&dir)?;
        let file = File::create(&path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(&bytes)?;
        encoder.finish()?;
    }
    
    Ok(hex_hash)
}

/// 读取并解压对象文件
pub fn read_object(oid: &str) -> Result<(String, Vec<u8>)> {
    let path = format!(".minigit/objects/{}/{}", &oid[..2], &oid[2..]);
    let file = File::open(&path).with_context(|| format!("Object {} not found", oid))?;
    let mut decoder = ZlibDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    let null_pos = buffer.iter().position(|&b| b == 0).context("Invalid object format")?;
    let header = String::from_utf8_lossy(&buffer[..null_pos]);
    let obj_type = header.split_whitespace().next().unwrap().to_string();
    let content = buffer[null_pos + 1..].to_vec();

    Ok((obj_type, content))
}