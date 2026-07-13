use std::fmt::Write as FmtWrite;

pub trait GitObject {
    // 返回对象类型
    fn object_type(&self) -> &'static str;
    // 返回对象纯内容的字节数组 (Vec<u8>)
    fn serialize_data(&self) -> Vec<u8>;
    // Git 规范： "{类型} {内容字节数}\0{真实内容}"
    fn to_bytes(&self) -> Vec<u8> {
        let data = self.serialize_data();
        let header = format!("{} {}\0", self.object_type(), data.len());
        let mut full_data = header.into_bytes();
        full_data.extend(data);// 把内容追加到头部后面
        full_data
    }
}

// === Blob 对象：代表纯粹的文件内容 ===
pub struct Blob {
    pub content: Vec<u8>,
}

impl GitObject for Blob {
    fn object_type(&self) -> &'static str { "blob" }
    fn serialize_data(&self) -> Vec<u8> { self.content.clone() }
}

// === Tree 对象：代表一个文件夹/目录结构 ===
pub struct TreeEntry {
    pub mode: String,    // 文件权限
    pub name: String,    // 文件名
    pub oid: [u8; 20],   // 文件内容对应的 SHA-1 哈希值
}

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl GitObject for Tree {
    fn object_type(&self) -> &'static str { "tree" }
    fn serialize_data(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        for entry in &self.entries {
            // Tree 格式："{权限} {文件名}\0{20位原始哈希字节}"
            data.extend(format!("{} {}\0", entry.mode, entry.name).into_bytes());
            data.extend_from_slice(&entry.oid);
        }
        data
    }
}

// === Commit 对象：代表一次提交快照 ===
pub struct Commit {
    pub tree_oid: String,              // 指向当前快照根目录(Tree)的哈希值
    pub parent_oid: Option<String>,    // 父提交的哈希
    pub author: String,                // 作者信息和时间戳
    pub message: String,               // 提交信息
}

impl GitObject for Commit {
    fn object_type(&self) -> &'static str { "commit" }
    fn serialize_data(&self) -> Vec<u8> {
        let mut data: String = String::new();
        // 使用 writeln! 宏向字符串中追加带换行的文本
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