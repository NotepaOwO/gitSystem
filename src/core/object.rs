use crate::utils::fs::{create_dir, read_file_bytes, write_file_bytes};
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
    /// 从仓库加载 Git 对象内容（Blob / Tree / Commit / Tag）
    ///
    /// # 参数
    /// - `repo_path`: 仓库根路径（包含 .git 文件夹）
    /// - `sha`: 对象 SHA1 哈希值（40 位十六进制字符串）
    ///
    /// # 返回
    /// - `Some(Vec<u8>)`：对象的原始二进制内容（不含 header）  
    /// - `None`：对象不存在或读取失败
    ///
    /// # 功能说明
    /// - Git 对象存储在 `.git/objects/xx/yyyy...`，xx 是 SHA 前两位，yyyy... 是剩余 38 位。
    /// - 文件内容包含 header + 数据，例如：
    ///     - Blob:  `blob 12\0<file content>`  
    ///     - Tree:  `tree 45\0<tree content>`  
    ///     - Commit: `commit 123\0<commit content>`
    /// - 本方法会去掉 header，返回纯数据部分。
    pub fn load(repo_path: &str, sha: &str) -> Option<Vec<u8>> {
        // ✅ 校验 SHA 长度
        if sha.len() < 40 {
            return None;
        }

        // 1️⃣ 构造对象路径
        // SHA: 0123456789abcdef...
        // 文件路径: .git/objects/01/23456789abcdef...
        let dir = &sha[0..2];
        let file = &sha[2..];
        let obj_path = Path::new(repo_path)
            .join(".git")
            .join("objects")
            .join(dir)
            .join(file);

        // 2️⃣ 文件不存在直接返回 None
        if !obj_path.exists() {
            return None;
        }

        // 3️⃣ 读取对象文件
        let data = match read_file_bytes(obj_path.to_str().unwrap()) {
            Ok(d) => d,
            Err(_) => return None,
        };

        // 4️⃣ 查找 header 结束位置（\0 分隔符）
        //    header 示例: "blob 123\0" -> 返回 \0 的位置
        let pos = match data.iter().position(|&b| b == 0) {
            Some(p) => p,
            None => 0, // 若没有找到 header，直接返回整个文件
        };

        // 5️⃣ 返回 header 之后的数据部分
        Some(data[pos + 1..].to_vec())
    }

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
