use crate::core::object::Object;
use crate::core::index::IndexEntry;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Tree 条目
pub struct TreeEntry {
    pub name: String,   // 文件名或目录名
    pub hash: String,   // SHA1 哈希
    pub mode: u32,      // 文件模式
    pub is_dir: bool,   // 是否为目录
}

/// Tree 对象处理器
pub struct TreeProcessor;

impl TreeProcessor {
    /// 生成空 Tree 对象（空提交）
    pub fn create_empty_tree(repo_path: &str) -> String {
        let tree_obj = Object::Tree(Vec::new());
        tree_obj.save(repo_path)
    }

    /// 根据 Index 条目生成 Tree 对象（递归）
    pub fn create_tree_from_index(
        repo_path: &str,
        index_entries: &HashMap<PathBuf, IndexEntry>,
    ) -> String {
        let mut dir_map: HashMap<PathBuf, Vec<&IndexEntry>> = HashMap::new();
        for (path, entry) in index_entries {
            let parent = path.parent().unwrap_or(Path::new(""));
            dir_map.entry(parent.to_path_buf()).or_default().push(entry);
        }
        Self::build_tree(repo_path, Path::new(""), &dir_map)
    }

    /// 内部递归生成 Tree
    fn build_tree(
        repo_path: &str,
        current_dir: &Path,
        dir_map: &HashMap<PathBuf, Vec<&IndexEntry>>,
    ) -> String {
        let mut entries = Vec::new();

        // 处理当前目录下的文件
        if let Some(files) = dir_map.get(current_dir) {
            for entry in files {
                if !entry.path.is_dir() {
                    let name = entry.path.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or_else(|| entry.path.to_str().unwrap())
                        .to_string();

                    entries.push(TreeEntry {
                        name,
                        hash: entry.sha.clone(),
                        mode: entry.mode,
                        is_dir: false,
                    });
                }
            }
        }

        // 处理子目录
        let mut subdirs = HashSet::new();
        for path in dir_map.keys() {
            if let Some(parent) = path.parent() {
                if parent == current_dir && path != current_dir {
                    subdirs.insert(path.clone());
                }
            }
        }

        for subdir in subdirs {
            let tree_hash = Self::build_tree(repo_path, &subdir, dir_map);

            let name = subdir.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")  // 根目录或特殊路径用空字符串
                .to_string();

            entries.push(TreeEntry {
                name,
                hash: tree_hash,
                mode: 0o40000, // 目录模式
                is_dir: true,
            });
        }

        Self::create_tree(repo_path, entries)
    }

    /// 创建 Tree 对象（二进制格式）
    fn create_tree(repo_path: &str, entries: Vec<TreeEntry>) -> String {
        let mut buf = Vec::new();
        for entry in entries {
            let mode_str = format!("{:o}", entry.mode);
            buf.extend(mode_str.as_bytes());
            buf.push(b' ');
            buf.extend(entry.name.as_bytes());
            buf.push(0); // null 分隔符
            buf.extend(hex::decode(&entry.hash).unwrap()); // 20 字节 SHA
        }
        let tree_obj = Object::Tree(buf);
        tree_obj.save(repo_path)
    }
}
