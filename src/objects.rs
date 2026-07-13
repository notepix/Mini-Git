use std::fmt::Write as FmtWrite;

pub trait GitObject {
    fn object_type(&self) -> &'static str;
    fn serialize_data(&self) -> Vec<u8>;
    fn to_bytes(&self) -> Vec<u8> {
        let data = self.serialize_data();
        let header = format!("{} {}\0", self.object_type(), data.len());
        let mut full_data = header.into_bytes();
        full_data.extend(data);
        full_data
    }
}

pub struct Blob {
    pub content: Vec<u8>,
}

impl GitObject for Blob {
    fn object_type(&self) -> &'static str { "blob" }
    fn serialize_data(&self) -> Vec<u8> { self.content.clone() }
}

pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub oid: [u8; 20],
}

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl GitObject for Tree {
    fn object_type(&self) -> &'static str { "tree" }
    fn serialize_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for entry in &self.entries {
            data.extend(format!("{} {}\0", entry.mode, entry.name).into_bytes());
            data.extend_from_slice(&entry.oid);
        }
        data
    }
}

pub struct Commit {
    pub tree_oid: String,
    pub parent_oid: Option<String>,
    pub author: String,
    pub message: String,
}

impl GitObject for Commit {
    fn object_type(&self) -> &'static str { "commit" }
    fn serialize_data(&self) -> Vec<u8> {
        let mut data = String::new();
        writeln!(&mut data, "tree {}", self.tree_oid).unwrap();
        if let Some(parent) = &self.parent_oid {
            writeln!(&mut data, "parent {}", parent).unwrap();
        }
        writeln!(&mut data, "author {}", self.author).unwrap();
        writeln!(&mut data, "committer {}", self.author).unwrap();
        writeln!(&mut data, "").unwrap();
        data.push_str(&self.message);
        data.push('\n');
        data.into_bytes()
    }
}