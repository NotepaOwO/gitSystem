use crate::core::reference::Reference;
use crate::utils::fs;
use std::path::Path;

/// ======================================
/// 🌿 Git branch 命令实现（更接近真实 Git）
///
/// ✅ 支持：
/// 1. `git branch`               —— 列出所有分支并标明当前分支  
/// 2. `git branch <name>`        —— 创建新分支（必须基于现有提交）  
/// 3. `git branch -d <name>`     —— 删除分支
///
/// ⚙️ 分支原理：
/// 每个分支对应 `.git/refs/heads/<branch>` 文件，
/// 文件内容是该分支当前指向的 commit 哈希。
///
/// 📌 注意区别于之前版本：
/// 现在不会再为“空仓库”自动创建空 tree / 空 commit，
/// 因为在真实 Git 中，未提交的仓库不能创建分支。
/// ======================================
pub fn git_branch(repo_path: &Path, branch_name: Option<&str>, delete: bool) {
    let refs_heads_path = repo_path.join(".git/refs/heads");

    // ==============================================================
    // 1️⃣ 无参数情况 —— 列出所有分支
    // ==============================================================
    if branch_name.is_none() {
        match fs::list_dir(&refs_heads_path) {
            Ok(branches) => {
                // 读取 HEAD 文件，获取当前分支引用
                let head_ref_path = repo_path.join(".git/HEAD");
                let head_ref = fs::read_file(&head_ref_path.to_str().unwrap())
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                // 提取当前分支名
                let current_branch = head_ref
                    .strip_prefix("ref: refs/heads/")
                    .unwrap_or("master");

                println!("📂 Branches:");
                for path in branches {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // 当前分支前加星号
                        if name == current_branch {
                            println!("* {}", name);
                        } else {
                            println!("  {}", name);
                        }
                    }
                }
            }
            Err(err) => println!("❌ Failed to list branches: {}", err),
        }
        return;
    }

    // ==============================================================
    // 2️⃣ 创建或删除分支
    // ==============================================================
    let branch_name = branch_name.unwrap();

    if delete {
        // --------------------------------------------------------------
        // 🗑️ 删除分支逻辑
        // --------------------------------------------------------------
        let branch_path = refs_heads_path.join(branch_name);
        if branch_path.exists() {
            fs::remove_file(&branch_path).expect("Failed to delete branch");
            println!("🗑️ Deleted branch '{}'", branch_name);
        } else {
            println!("⚠️ Branch '{}' does not exist", branch_name);
        }
    } else {
        // --------------------------------------------------------------
        // 🌱 创建新分支逻辑
        // --------------------------------------------------------------

        // 读取 HEAD 文件，确定当前分支引用
        let head_ref_path = repo_path.join(".git/HEAD");
        let head_ref = fs::read_file(&head_ref_path.to_str().unwrap())
            .expect("Failed to read HEAD")
            .trim()
            .to_string();

        // 提取当前分支路径，例如 "refs/heads/master"
        let current_branch_ref = head_ref.strip_prefix("ref: ").unwrap_or("refs/heads/master");

        // 获取当前分支指向的最新 commit
        let current_commit =
            Reference::resolve(repo_path.to_str().unwrap(), current_branch_ref);

        // ❌ 如果当前分支没有 commit，拒绝创建新分支（符合真实 Git 逻辑）
        if current_commit.is_none() {
            println!("⚠️ Cannot create branch '{}': current branch has no commits.", branch_name);
            println!("💡 Hint: make an initial commit first.");
            return;
        }

        // ✅ 创建新分支引用文件并指向当前 commit
        let commit_hash = current_commit.unwrap();
        Reference::create(
            repo_path.to_str().unwrap(),
            &format!("refs/heads/{}", branch_name),
            &commit_hash,
        );

        println!("🌿 Created branch '{}' at {}", branch_name, commit_hash);
    }
}
