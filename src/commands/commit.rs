use crate::core::object::Object;
use crate::core::index::Index;
use crate::core::tree::TreeProcessor;
use crate::core::reference::Reference;
use crate::utils::fs::write_file;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// git commit 命令（提交当前暂存区）
///
/// # 功能
/// - 从 index 构建 tree 对象
/// - 生成 commit 对象并保存至 .git/objects
/// - 更新当前分支（refs/heads/xxx）
/// - 更新 HEAD 指向新的 commit
///
/// # 参数
/// - `repo_path`: 仓库根路径
/// - `message`: 提交信息
pub fn git_commit(repo_path: &Path, message: &str) {
    // 1️⃣ 加载 index，准备构造 tree
    let index = Index::load(repo_path);

    // 2️⃣ 构造 tree 对象内容
    // 3️⃣ 写入 tree 对象
    let tree_sha = TreeProcessor::create_tree_from_index(&repo_path.to_str().unwrap(), &index.entries);

    // 4️⃣ 获取当前分支
    let head_path = repo_path.join(".git/HEAD");
    let head_content = std::fs::read_to_string(&head_path).unwrap_or_default();
    let (is_branch, branch_name) = if head_content.starts_with("ref:") {
        (true, head_content.trim_start_matches("ref: ").trim().to_string())
    } else {
        (false, String::new())
    };

    // 5️⃣ 获取 parent commit（如果存在）
    let parent = if is_branch {
        Reference::resolve(repo_path.to_str().unwrap(), &branch_name)
    } else {
        None
    };

    // 6️⃣ 构造 commit 对象
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut commit_content = format!("tree {}\n", tree_sha);
    if let Some(parent_sha) = &parent {
        commit_content.push_str(&format!("parent {}\n", parent_sha));
    }
    commit_content.push_str(&format!("author You <you@example.com> {}\n\n{}", timestamp, message));

    // 7️⃣ 保存 commit 对象
    let commit_sha = Object::Commit(commit_content.as_bytes().to_vec()).save(repo_path.to_str().unwrap());

    // 8️⃣ 更新分支引用（若 HEAD 是分支）
    if is_branch {
        let ref_path = repo_path.join(".git").join(&branch_name);
        write_file(ref_path.to_str().unwrap(), &commit_sha).expect("Failed to update branch ref");
        println!("✅ Commit saved to branch '{}': {}", branch_name, commit_sha);
    } else {
        // Detached HEAD
        write_file(head_path.to_str().unwrap(), &commit_sha).expect("Failed to update HEAD");
        println!("⚠️ Detached HEAD now at {}", commit_sha);
    }
}
