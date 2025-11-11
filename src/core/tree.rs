use crate::core::object::Object;
use crate::core::index::IndexEntry;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Tree 条目
#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub name: String,   // 文件名或目录名
    pub hash: String,   // 对应对象的 SHA1 哈希
    pub mode: u32,      // 文件模式（100644 普通文件，100755 可执行文件，40000 目录）
    pub is_dir: bool,   // 是否为目录
}

/// Tree 对象处理器
pub struct TreeProcessor;

impl TreeProcessor {
    // =====================================
    // ---------- 写操作（生成 Tree） ----------
    // =====================================

    /// 生成空 Tree 对象（用于空提交）
    pub fn create_empty_tree(repo_path: &str) -> String {
        let tree_obj = Object::Tree(Vec::new());
        tree_obj.save(repo_path)
    }

    /// 根据 Index 条目生成 Tree 对象（递归）
    /// - 输入：Index 哈希表（PathBuf -> IndexEntry）
    /// - 输出：Tree 对象 SHA1
    pub fn create_tree_from_index(
        repo_path: &str,
        index_entries: &HashMap<PathBuf, IndexEntry>,
    ) -> String {
        let mut dir_map: HashMap<PathBuf, Vec<&IndexEntry>> = HashMap::new();

        // 将每个文件按照父目录分组
        for (path, entry) in index_entries {
            let parent = path.parent().unwrap_or(Path::new(""));
            dir_map.entry(parent.to_path_buf()).or_default().push(entry);
        }

        // 从根目录开始递归生成 tree
        Self::build_tree(repo_path, Path::new(""), &dir_map)
    }

    /// 内部递归生成 Tree 对象
    fn build_tree(
        repo_path: &str,
        current_dir: &Path,
        dir_map: &HashMap<PathBuf, Vec<&IndexEntry>>,
    ) -> String {
        let mut entries = Vec::new();

        // 1️⃣ 处理当前目录下的文件
        if let Some(files) = dir_map.get(current_dir) {
            for entry in files {
                if !entry.path.is_dir() {
                    let name = entry
                        .path
                        .file_name()
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

        // 2️⃣ 处理子目录
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
            let name = subdir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            entries.push(TreeEntry {
                name,
                hash: tree_hash,
                mode: 0o40000, // 目录模式
                is_dir: true,
            });
        }

        // 3️⃣ 创建 tree 对象（二进制）
        Self::create_tree(repo_path, entries)
    }

    /// 将 TreeEntry 列表写入 Tree 对象（二进制格式）
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

    // =====================================
    // ---------- 读操作（解析 Tree） ----------
    // =====================================

    /// 解析 Tree 对象二进制数据
    /// - 输入：tree 对象二进制 Vec<u8>
    /// - 输出：Vec<TreeEntry>，包含目录和文件条目信息
    pub fn parse_tree(tree_data: &[u8]) -> Vec<TreeEntry> {
        let mut entries = Vec::new();
        let mut i = 0;

        while i < tree_data.len() {
            // 1️⃣ 读取 mode 字符串
            let start = i;
            while tree_data[i] != b' ' { i += 1; }
            let mode_str = std::str::from_utf8(&tree_data[start..i]).unwrap();
            let mode = u32::from_str_radix(mode_str, 8).unwrap(); // 8 = 八进制
            i += 1;

            // 2️⃣ 读取 name（以 null 结束）
            let start = i;
            while tree_data[i] != 0 { i += 1; }
            let name = String::from_utf8(tree_data[start..i].to_vec()).unwrap();
            i += 1;

            // 3️⃣ 读取 20 字节 SHA
            let sha_bytes = &tree_data[i..i+20];
            let hash = hex::encode(sha_bytes);
            i += 20;

            // 4️⃣ 判断是否目录
            let is_dir = mode == 0o40000;
            entries.push(TreeEntry {
                name,
                hash,
                mode,
                is_dir,
            });
        }

        entries
    }
}
