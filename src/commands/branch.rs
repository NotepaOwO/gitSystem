use crate::core::reference::Reference;
use crate::utils::fs;
use std::path::Path;

/// ======================================
/// 🌿 Git branch 命令实现（增强版）
///
/// ✅ 支持功能：
/// 1. `git branch`               —— 列出所有分支并标明当前分支  
/// 2. `git branch <name>`        —— 创建新分支（基于当前 commit）  
/// 3. `git branch -d <name>`     —— 删除分支（禁止删除当前分支）
///
/// ⚙️ 实现原理：
/// - 每个分支对应 `.git/refs/heads/<branch>` 文件，
///   文件内容为该分支当前指向的 commit 哈希。
/// - `.git/HEAD` 文件保存当前检出分支引用（例如：`ref: refs/heads/master`）
/// ======================================
pub fn git_branch(repo_path: &Path, branch_name: Option<&str>, delete: bool) {
    let refs_heads_path = repo_path.join(".git/refs/heads");

    // ==============================================================
    // 1️⃣ 无参数：列出所有分支
    // ==============================================================
    if branch_name.is_none() {
        match fs::list_dir(&refs_heads_path) {
            Ok(branches) => {
                // 尝试读取 HEAD 文件内容（可能为空或损坏）
                let head_ref_path = repo_path.join(".git/HEAD");
                let head_ref = fs::read_file(&head_ref_path.to_str().unwrap())
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                // 若 HEAD 文件为空，提示用户初始化提交
                if head_ref.is_empty() {
                    println!("⚠️  No HEAD reference found. Repository may not have an initial commit.");
                    println!("💡 Run `git commit -m \"msg\"` before creating branches.");
                    return;
                }

                // 提取当前分支名（去掉前缀 ref: refs/heads/）
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

        // 读取 HEAD 以判断当前分支
        let head_ref_path = repo_path.join(".git/HEAD");
        let head_ref = fs::read_file(&head_ref_path.to_str().unwrap()).unwrap_or_default();
        let current_branch = head_ref
            .strip_prefix("ref: refs/heads/")
            .unwrap_or("master")
            .trim();

        // 禁止删除当前分支（与真实 Git 一致）
        if branch_name == current_branch {
            println!("❌ Cannot delete the current checked-out branch '{}'", branch_name);
            println!("💡 Switch to another branch first: `git checkout <other>`");
            return;
        }

        // 检查分支是否存在
        if branch_path.exists() {
            fs::remove_file(&branch_path).expect("Failed to delete branch");
            println!("🗑️  Deleted branch '{}'", branch_name);
        } else {
            println!("⚠️  Branch '{}' does not exist", branch_name);
        }
    } else {
        // --------------------------------------------------------------
        // 🌱 创建新分支逻辑
        // --------------------------------------------------------------

        // 尝试读取 HEAD 文件
        let head_ref_path = repo_path.join(".git/HEAD");
        let head_ref = fs::read_file(&head_ref_path.to_str().unwrap())
            .unwrap_or_default()
            .trim()
            .to_string();

        if head_ref.is_empty() {
            println!("⚠️ Cannot create branch '{}': HEAD is missing or empty.", branch_name);
            println!("💡 Make an initial commit first.");
            return;
        }

        // 提取当前分支路径，例如 "refs/heads/master"
        let current_branch_ref = head_ref.strip_prefix("ref: ").unwrap_or("refs/heads/master");

        // 获取当前分支指向的最新 commit
        let current_commit =
            Reference::resolve(repo_path.to_str().unwrap(), current_branch_ref);

        // ❌ 当前分支尚无提交
        if current_commit.is_none() {
            println!(
                "⚠️ Cannot create branch '{}': current branch has no commits.",
                branch_name
            );
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

        // 统一路径格式（Windows '\' → '/'）
        println!(
            "🌿 Created branch '{}' at {}",
            branch_name.replace('\\', "/"),
            commit_hash
        );
    }
}
