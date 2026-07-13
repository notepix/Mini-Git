use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::objects::GitObject;

/// 核心写函数：接收 GitObject 对象，返回 SHA-1 字符串
pub fn write_object(obj: &dyn GitObject) -> Result<String> {
    // 1. 计算 SHA-1 哈希
    let bytes: Vec<u8> = obj.to_bytes();
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    let hex_hash: String = hex::encode(hash);

    // 2. 构造路径
    let dir: String = format!(".minigit/objects/{}", &hex_hash[..2]);
    let path: String = format!("{}/{}", dir, &hex_hash[2..]);

    // 3. 写入磁盘并压缩
    if !Path::new(&path).exists() {     // 避免重复写入
        fs::create_dir_all(&dir)?;
        let file: File = File::create(&path)?;
        // 创建一个 Zlib 压缩器包装这个文件
        let mut encoder: ZlibEncoder<File> = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(&bytes)?;
        encoder.finish()?;
    }
    
    Ok(hex_hash)// 返回计算出的 40 位哈希字符串
}

/// 核心读函数：通过哈希值，从磁盘里把原始对象数据读出来
pub fn read_object(oid: &str) -> Result<(String, Vec<u8>)> {
    let path: String = format!(".minigit/objects/{}/{}", &oid[..2], &oid[2..]);
    // 打开文件，如果文件不存在，附加上下文错误信息返回
    let file: File = File::open(&path).with_context(|| format!("Object {} not found", oid))?;

    // 1. 解压数据
    let mut decoder: ZlibDecoder<File> = ZlibDecoder::new(file);
    let mut buffer: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut buffer)?;// 解压到内存缓冲区 buffer 中

    // 2. 解析 Header ("类型 长度\0内容")
    let null_pos: usize = buffer.iter().position(|&b| b == 0).context("Invalid object format")?;
    let header = String::from_utf8_lossy(&buffer[..null_pos]);
    let obj_type: String = header.split_whitespace().next().unwrap().to_string();
    let content: Vec<u8> = buffer[null_pos + 1..].to_vec();

    Ok((obj_type, content))// 返回 (类型, 内容) 的元组
}