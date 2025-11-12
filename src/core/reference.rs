use crate::utils::fs::{create_dir, write_file, read_file, check_path_exists};
use std::path::Path;

/// 引用管理器（不存储状态，纯操作类）
pub struct Reference;

impl Reference {
    /// 创建引用文件（分支或标签）
    pub fn create(repo_path: &str, ref_name: &str, target_hash: &str) {
        // 构建完整路径：.git/refs/... 
        let ref_path = Path::new(repo_path).join(".git").join(ref_name);
        if let Some(parent) = ref_path.parent() {
            create_dir(parent.to_str().unwrap()); // 确保目录存在
        }
        write_file(ref_path.to_str().unwrap(), target_hash)
            .expect("Failed to write reference file");
    }

    /// 删除引用文件
    pub fn delete(repo_path: &str, ref_name: &str) {
        let ref_path = Path::new(repo_path).join(".git").join(ref_name);
        if check_path_exists(ref_path.to_str().unwrap()) {
            std::fs::remove_file(ref_path).expect("Failed to delete reference file");
        }
    }

    /// 解析引用内容，返回对应的哈希
    pub fn resolve(repo_path: &str, ref_name: &str) -> Option<String> {
        let ref_path = Path::new(repo_path).join(".git").join(ref_name);
        if check_path_exists(ref_path.to_str().unwrap()) {
            let content = read_file(ref_path.to_str().unwrap())
                .expect("Failed to read reference file");
            Some(content.trim().to_string())
        } else {
            None
        }
    }
}
