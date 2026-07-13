use anyhow::{Context, Result};
use chrono::Local;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

use crate::objects::{Blob, Commit, GitObject, Tree, TreeEntry};
use crate::storage::{read_object, write_object};

pub fn init() -> Result<()> {
    fs::create_dir_all(".minigit/objects")?;
    fs::create_dir_all(".minigit/refs/heads")?;
    fs::write(".minigit/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized empty Minigit repository in .minigit/");
    Ok(())
}

pub fn hash_object(file_path: &str, write: bool) -> Result<()> {
    let content = fs::read(file_path).with_context(|| format!("Failed to read {}", file_path))?;
    let blob = Blob { content };
    
    if write {
        let hash = write_object(&blob)?;
        println!("{}", hash);
    } else {
        let bytes = blob.to_bytes();
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        println!("{}", hex::encode(hasher.finalize()));
    }
    Ok(())
}

pub fn add(file_path: &str) -> Result<()> {
    let content = fs::read(file_path)?;
    let blob = Blob { content };
    let hash = write_object(&blob)?;

    let index_path = ".minigit/index";
    let mut index_entries = HashMap::new();
    
    if Path::new(index_path).exists() {
        let index_content = fs::read_to_string(index_path)?;
        for line in index_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                index_entries.insert(parts[2].to_string(), (parts[0].to_string(), parts[1].to_string()));
            }
        }
    }
    
    index_entries.insert(file_path.to_string(), ("100644".to_string(), hash));

    let mut new_index = String::new();
    for (path, (mode, oid)) in index_entries {
        core::fmt::write(&mut new_index, format_args!("{} {} {}\n", mode, oid, path))?;
    }
    fs::write(index_path, new_index)?;
    
    println!("Added {} to index", file_path);
    Ok(())
}

pub fn commit(message: &str) -> Result<()> {
    let index_path = ".minigit/index";
    if !Path::new(index_path).exists() {
        anyhow::bail!("Nothing to commit (index is empty)");
    }
    
    let index_content = fs::read_to_string(index_path)?;
    let mut entries = Vec::new();
    for line in index_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            let mut oid_bytes = [0u8; 20];
            hex::decode_to_slice(parts[1], &mut oid_bytes)?;
            entries.push(TreeEntry {
                mode: parts[0].to_string(),
                oid: oid_bytes,
                name: parts[2].to_string(),
            });
        }
    }
    
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let tree = Tree { entries };
    let tree_oid = write_object(&tree)?;

    let head_ref = fs::read_to_string(".minigit/HEAD")?;
    let ref_path = format!(".minigit/{}", head_ref.trim().strip_prefix("ref: ").unwrap_or(""));
    let parent_oid = if Path::new(&ref_path).exists() {
        Some(fs::read_to_string(&ref_path)?.trim().to_string())
    } else {
        None
    };

    let now = Local::now();
    let offset_secs = now.offset().local_minus_utc();
    let sign = if offset_secs < 0 { '-' } else { '+' };
    let offset_hours = (offset_secs.abs() / 3600) as i32;
    let offset_mins = ((offset_secs.abs() % 3600) / 60) as i32;
    let author = format!("Minigit User <user@minigit.local> {} {}{:02}{:02}", now.timestamp(), sign, offset_hours, offset_mins);

    let commit = Commit {
        tree_oid,
        parent_oid,
        author,
        message: message.to_string(),
    };
    let commit_oid = write_object(&commit)?;

    if !ref_path.is_empty() {
        fs::write(ref_path, format!("{}\n", commit_oid))?;
    }
    
    println!("[{}] {}", &commit_oid[..7], message);
    Ok(())
}

pub fn log() -> Result<()> {
    let head_ref = fs::read_to_string(".minigit/HEAD").context("Not a minigit repository")?;
    let ref_path = format!(".minigit/{}", head_ref.trim().strip_prefix("ref: ").unwrap_or(""));
    
    if !Path::new(&ref_path).exists() {
        anyhow::bail!("No commits yet");
    }
    
    let mut current_oid = fs::read_to_string(ref_path)?.trim().to_string();
    
    loop {
        let (obj_type, content) = read_object(&current_oid)?;
        if obj_type != "commit" {
            anyhow::bail!("Expected commit object, found {}", obj_type);
        }
        
        let content_str = String::from_utf8_lossy(&content);
        let mut parts = content_str.splitn(2, "\n\n");
        let headers = parts.next().unwrap_or("");
        let message = parts.next().unwrap_or("").trim();
        
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