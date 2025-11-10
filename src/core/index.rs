use crate::utils::fs::{read_file_bytes, write_file_bytes};
// use crate::utils::hash::sha1;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::metadata;

/// Index 条目
#[derive(Clone, Debug)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub sha: String,
    pub mode: u32,
    pub mtime: u64,
    pub ctime: u64,
    pub size: u64,
}

/// Git Index 暂存区
#[derive(Debug)]
pub struct Index {
    pub repo_path: PathBuf,
    pub entries: HashMap<PathBuf, IndexEntry>,
}

impl Index {
    /// 加载仓库的 index 文件（二进制）
    pub fn load(repo_path: &Path) -> Self {
        let index_file = repo_path.join(".git").join("index");
        let mut entries = HashMap::new();

        if index_file.exists() {
            let content = read_file_bytes(index_file.to_str().unwrap()).unwrap_or_default();
            let mut i = 0;
            while i + 20 + 4 + 8*3 <= content.len() {
                let sha = hex::encode(&content[i..i+20]); i+=20;
                let mode = u32::from_be_bytes(content[i..i+4].try_into().unwrap()); i+=4;
                let mtime = u64::from_be_bytes(content[i..i+8].try_into().unwrap()); i+=8;
                let ctime = u64::from_be_bytes(content[i..i+8].try_into().unwrap()); i+=8;
                let size = u64::from_be_bytes(content[i..i+8].try_into().unwrap()); i+=8;
                let path_len = content[i] as usize; i+=1;
                let path = PathBuf::from(String::from_utf8(content[i..i+path_len].to_vec()).unwrap());
                i += path_len;

                entries.insert(path.clone(), IndexEntry { path, sha, mode, mtime, ctime, size });
            }
        }

        Index { repo_path: repo_path.to_path_buf(), entries }
    }

    /// 保存 Index（二进制）
    pub fn save(&self) {
        let index_file = self.repo_path.join(".git").join("index");
        let mut buf = Vec::new();
        for entry in self.entries.values() {
            buf.extend(hex::decode(&entry.sha).unwrap());
            buf.extend(&entry.mode.to_be_bytes());
            buf.extend(&entry.mtime.to_be_bytes());
            buf.extend(&entry.ctime.to_be_bytes());
            buf.extend(&entry.size.to_be_bytes());
            let path_bytes = entry.path.to_str().unwrap().as_bytes();
            buf.push(path_bytes.len() as u8);
            buf.extend(path_bytes);
        }
        write_file_bytes(index_file.to_str().unwrap(), &buf).unwrap();
    }

    /// 将文件加入暂存区（存相对路径）
    pub fn stage_file(&mut self, file_path: &Path, obj_sha: &String) {
        let metadata = metadata(file_path).unwrap();
        let sha = obj_sha.clone();
        let mode = if metadata.permissions().readonly() { 0o100644 } else { 0o100755 };
        let mtime = metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let ctime = mtime;
        let size = metadata.len();

        // ✅ 使用相对仓库根路径
        let relative_path = file_path.strip_prefix(&self.repo_path)
            .unwrap_or(file_path)
            .to_path_buf();

        let entry = IndexEntry {
            path: relative_path.clone(),
            sha,
            mode,
            mtime,
            ctime,
            size,
        };

        self.entries.insert(relative_path, entry);
        self.save();
    }

    /// 从暂存区移除文件
    pub fn unstage_file(&mut self, file_path: &Path) {
        self.entries.remove(file_path);
        self.save();
    }

    /// 清空暂存区
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }
}
