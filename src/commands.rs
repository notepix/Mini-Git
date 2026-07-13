use anyhow::{Context, Result};
use chrono::Local;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::objects::{Blob, Commit, GitObject, Tree, TreeEntry};
use crate::storage::{read_object, write_object};

// minigit init
pub fn init() -> Result<()> {
    fs::create_dir_all(".minigit/objects")?;// 存放所有哈希对象的目录
    fs::create_dir_all(".minigit/refs/heads")?;// 存放分支指针的目录
    fs::write(".minigit/HEAD", "ref: refs/heads/main\n")?;// 指明当前在 main 分支
    println!("Initialized empty Minigit repository in .minigit/");
    Ok(())
}

pub fn hash_object(file_path: &str, write: bool) -> Result<()> {
    let content: Vec<u8> = fs::read(file_path).with_context(|| format!("Failed to read {}", file_path))?;
    let blob: Blob = Blob { content };
    
    if write {
        let hash: String = write_object(&blob)?;
        println!("{}", hash);
    } else {
        let bytes: Vec<u8> = blob.to_bytes();
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        println!("{}", hex::encode(hasher.finalize()));
    }
    Ok(())
}

// minigit add <file>
pub fn add(file_path: &str) -> Result<()> {
    // 1. 读取文件内容，转成 Blob，计算哈希并存入 objects 库
    let content: Vec<u8> = fs::read(file_path)?;
    let blob: Blob = Blob { content };
    let hash: String = write_object(&blob)?;

    // 2. 更新暂存区 (Index)
    let index_path: &str = ".minigit/index";
    let mut index_entries = HashMap::new();
    
    if Path::new(index_path).exists() {
        let index_content: String = fs::read_to_string(index_path)?;
        for line in index_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                index_entries.insert(parts[2].to_string(), (parts[0].to_string(), parts[1].to_string()));
            }
        }
    }
    
    // 把这次新添加的文件覆盖或者插入进去
    index_entries.insert(file_path.to_string(), ("100644".to_string(), hash));

    // 把 HashMap 的内容重新拼接成字符串，覆盖写入 index 文件
    let mut new_index: String = String::new();
    for (path, (mode, oid)) in index_entries {
        core::fmt::write(&mut new_index, format_args!("{} {} {}\n", mode, oid, path))?;
    }
    fs::write(index_path, new_index)?;
    
    println!("Added {} to index", file_path);
    Ok(())
}

// minigit commit -m "msg"
pub fn commit(message: &str) -> Result<()> {
    // 1. 读取 index 暂存区的内容
    let index_path: &str = ".minigit/index";
    if !Path::new(index_path).exists() {
        anyhow::bail!("Nothing to commit (index is empty)");
    }
    
    let index_content: String = fs::read_to_string(index_path)?;

    // 把暂存区里的每一个文件记录，转换成 TreeEntry 结构
    let mut entries: Vec<TreeEntry> = Vec::new();
    for line in index_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            let mut oid_bytes: [u8; 20] = [0u8; 20];
            hex::decode_to_slice(parts[1], &mut oid_bytes)?;
            entries.push(TreeEntry {
                mode: parts[0].to_string(),
                oid: oid_bytes,
                name: parts[2].to_string(),
            });
        }
    }
    // 按文件名排序
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let tree: Tree = Tree { entries };
    let tree_oid: String = write_object(&tree)?;

    // 2. 寻找当前分支的"上一次提交"作为父节点
    let head_ref: String = fs::read_to_string(".minigit/HEAD")?;
    let ref_path: String = format!(".minigit/{}", head_ref.trim().strip_prefix("ref: ").unwrap_or(""));
    let parent_oid: Option<String> = if Path::new(&ref_path).exists() {
        Some(fs::read_to_string(&ref_path)?.trim().to_string())
    } else {
        None
    };

    // 3. 组装作者和时间信息
    let now: chrono::prelude::DateTime<Local> = Local::now();
    let offset_secs = now.offset().local_minus_utc();
    let sign: char = if offset_secs < 0 { '-' } else { '+' };
    let offset_hours: i32 = (offset_secs.abs() / 3600) as i32;
    let offset_mins: i32 = ((offset_secs.abs() % 3600) / 60) as i32;
    let author: String = format!("Minigit User <user@minigit.local> {} {}{:02}{:02}", now.timestamp(), sign, offset_hours, offset_mins);

    // 4. 生成 Commit 对象并存入对象库
    let commit: Commit = Commit {
        tree_oid,
        parent_oid,
        author,
        message: message.to_string(),
    };
    let commit_oid: String = write_object(&commit)?;

    // 5. 移动 HEAD 指针，把当前分支的文件内容替换为这次新提交的哈希值
    if !ref_path.is_empty() {
        fs::write(ref_path, format!("{}\n", commit_oid))?;
    }
    
    println!("[{}] {}", &commit_oid[..7], message);
    Ok(())
}

// minigit log
pub fn log() -> Result<()> {
    // 1. 顺着 HEAD 找到当前分支指向的最新 Commit 哈希
    let head_ref: String = fs::read_to_string(".minigit/HEAD").context("Not a minigit repository")?;
    let ref_path: String = format!(".minigit/{}", head_ref.trim().strip_prefix("ref: ").unwrap_or(""));
    
    if !Path::new(&ref_path).exists() {
        anyhow::bail!("No commits yet");
    }
    
    let mut current_oid: String = fs::read_to_string(ref_path)?.trim().to_string();
    
    // 2. 递归查找历史
    loop {
        let (obj_type, content) = read_object(&current_oid)?;
        if obj_type != "commit" {
            anyhow::bail!("Expected commit object, found {}", obj_type);
        }
        
        let content_str = String::from_utf8_lossy(&content);
        let mut parts = content_str.splitn(2, "\n\n");
        let headers: &str = parts.next().unwrap_or("");
        let message: &str = parts.next().unwrap_or("").trim();
        
        let mut author = "";
        let mut parent = None;
        
        for line in headers.lines() {
            if let Some(a) = line.strip_prefix("author ") {
                author = a;
            } else if let Some(p) = line.strip_prefix("parent ") {
                parent = Some(p.to_string());
            }
        }
        
        println!("commit {}", current_oid);
        println!("Author: {}", author);
        println!("\n    {}\n", message);
        
        match parent {
            Some(p) => current_oid = p,
            None => break,
        }
    }
    Ok(())
}