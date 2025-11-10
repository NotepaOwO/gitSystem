use crate::core::object::Object;
use chrono::Local;

pub struct CommitBuilder;

impl CommitBuilder {
    /// 创建新提交对象
    pub fn create_commit(
        repo_path: &str,
        tree_hash: String,               // 关联的树对象哈希
        parent_commit: Option<String>,   // 父提交哈希
        author_info: String,             // 作者信息，如 "Tom <tom@example.com>"
        commit_message: String           // 提交信息
    ) -> String {
        // 获取当前时间戳（RFC 2822 格式）
        let timestamp = Local::now().to_rfc2822();

        // 构造提交内容
        let mut commit_content = format!("tree {}\n", tree_hash);
        if let Some(parent_hash) = parent_commit {
            commit_content.push_str(&format!("parent {}\n", parent_hash));
        }
        commit_content.push_str(&format!("author {} {}\n", author_info, timestamp));
        commit_content.push_str(&format!("committer {} {}\n", author_info, timestamp));
        commit_content.push_str("\n"); // 空行分隔头部和提交信息
        commit_content.push_str(&commit_message);

        // 转为二进制 Vec<u8> 保存
        let commit_obj = Object::Commit(commit_content.into_bytes());
        let commit_hash = commit_obj.save(repo_path);

        commit_hash
    }
}
