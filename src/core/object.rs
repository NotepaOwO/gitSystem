use crate::utils::fs::{create_dir, write_file_bytes};
use crate::utils::hash::sha1;
use std::path::Path;

/// Git 对象类型，全部使用二进制 Vec<u8>
pub enum Object {
    Commit(Vec<u8>),
    Tree(Vec<u8>),
    Blob(Vec<u8>),
    Tag(Vec<u8>),
}

impl Object {
    /// 保存对象到仓库，返回 SHA1 哈希
    pub fn save(&self, repo_path: &str) -> String {
        // 构造 header + 数据
        let raw_data: Vec<u8> = match self {
            Object::Commit(data) => [format!("commit {}\0", data.len()).as_bytes(), data].concat(),
            Object::Tree(data) => [format!("tree {}\0", data.len()).as_bytes(), data].concat(),
            Object::Blob(data) => [format!("blob {}\0", data.len()).as_bytes(), data].concat(),
            Object::Tag(data) => [format!("tag {}\0", data.len()).as_bytes(), data].concat(),
        };

        // 计算 SHA1
        let hash = sha1(&raw_data);

        // 创建对象目录
        let dir_path = Path::new(repo_path).join(".git").join("objects").join(&hash[0..2]);
        create_dir(dir_path.to_str().unwrap());

        // 保存对象文件
        let file_path = dir_path.join(&hash[2..]);
        write_file_bytes(file_path.to_str().unwrap(), &raw_data).unwrap();

        hash
    }

    /// 创建标签引用（refs/tags/<tag_name>）
    pub fn create_tag_ref(repo_path: &str, tag_name: &str, obj_hash: &str) {
        let ref_path = Path::new(repo_path).join(".git").join("refs").join("tags");
        create_dir(ref_path.to_str().unwrap());
        let tag_file = ref_path.join(tag_name);
        let _ = crate::utils::fs::write_file(tag_file.to_str().unwrap(), obj_hash);
    }
}
