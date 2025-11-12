use crate::core::object::Object;

/// Blob 对象处理器
pub struct BlobProcessor;

impl BlobProcessor {
    /// 创建 Blob 对象（支持二进制文件）
    /// # 参数
    /// - `repo_path`: 仓库路径
    /// - `content`: 文件内容（可二进制）
    /// # 返回值
    /// - Blob 对象 SHA1 哈希
    pub fn create_blob(repo_path: &str, content: &[u8]) -> String {
        let blob_obj = Object::Blob(content.to_vec()); // 创建 Blob 对象
        blob_obj.save(repo_path) // 保存到 .git/objects 并返回哈希
    }
}
