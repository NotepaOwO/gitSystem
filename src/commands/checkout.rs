use crate::core::{index::Index, object::Object, reference::Reference, tree::TreeProcessor};
use crate::utils::fs;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// git checkout 命令实现
///
/// ✅ 功能：
/// 1. 切换到已有分支或 commit  
/// 2. 支持 `-b <branch>` 创建新分支  
/// 3. 检查工作区干净  
/// 4. 同步 HEAD、index、工作区，删除 commit 中没有的文件和空目录
pub fn git_checkout(repo_path: &Path, target: &str, create_new: bool) {
    // ------------------ 1️⃣ 检查工作区是否干净 ------------------
    if !is_workdir_clean(repo_path) {
        panic!("⚠️ Cannot checkout: working directory has uncommitted changes");
    }

    // ------------------ 2️⃣ 获取当前 HEAD ------------------
    let head_path = repo_path.join(".git/HEAD");
    let head_ref = fs::read_file(&head_path.to_str().unwrap())
        .unwrap_or_default()
        .trim()
        .to_string();
    let current_branch_ref = head_ref.strip_prefix("ref: ").unwrap_or("");

    // ------------------ 3️⃣ 计算目标引用 ------------------
    let target_branch_ref = format!("refs/heads/{}", target);

    // ------------------ 4️⃣ 获取目标 commit SHA ------------------
    let target_commit_sha = if create_new {
        // 新分支基于当前分支最新 commit
        let base_commit = Reference::resolve(repo_path.to_str().unwrap(), current_branch_ref)
            .expect("Cannot create branch: current branch has no commits");
        Reference::create(repo_path.to_str().unwrap(), &target_branch_ref, &base_commit);
        base_commit
    } else {
        let branch_path = repo_path.join(".git").join(&target_branch_ref);
        if branch_path.exists() {
            // 目标是分支
            Reference::resolve(repo_path.to_str().unwrap(), &target_branch_ref)
                .expect("Target branch has no commit")
        } else {
            // 目标是 commit SHA
            target.to_string()
        }
    };
    println!("Target commit SHA: {}", target_commit_sha);

    // ------------------ 5️⃣ 移动 HEAD ------------------
    let new_head_content = if create_new || target_branch_ref.starts_with("refs/heads/") {
        format!("ref: {}", target_branch_ref)
    } else {
        target_commit_sha.clone() // detached HEAD
    };
    fs::write_file_bytes(&head_path.to_str().unwrap(), new_head_content.as_bytes())
        .expect("Failed to update HEAD");

    // ------------------ 6️⃣ 更新 index 和工作区 ------------------
    restore_index_and_workdir(repo_path, &target_commit_sha);

    println!("✅ Checked out {}", target);
}

/// 检查工作区是否干净（工作区与 index 比对）
fn is_workdir_clean(repo_path: &Path) -> bool {
    let index = Index::load(repo_path);
    for entry in index.entries.values() {
        if let Ok(content) = fs::read_file_bytes(&entry.path.to_str().unwrap()) {
            let sha = Object::Blob(content).save(repo_path.to_str().unwrap());
            if sha != entry.sha {
                return false;
            }
        }
    }
    true
}

/// 更新 index 和工作区，使其与目标 commit 对齐，同时删除多余文件和空目录
fn restore_index_and_workdir(repo_path: &Path, commit_sha: &str) {
    // 1️⃣ 加载 commit 对应 tree
    let commit_obj = Object::load(repo_path.to_str().unwrap(), commit_sha)
        .expect("Failed to load commit object");
    let commit_content = String::from_utf8(commit_obj).unwrap();
    let tree_sha = commit_content
        .lines()
        .find(|l| l.starts_with("tree "))
        .expect("Commit object missing tree")
        .strip_prefix("tree ")
        .unwrap();
    println!("Restoring tree: {}", tree_sha);

    // 2️⃣ 记录工作区现有文件和目录（排除 .git）
    let mut workdir_paths = HashSet::new();
    for entry in walkdir::WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.path().as_os_str() == "." {
            continue;
        }

        if entry.path().components().any(|c| c.as_os_str() == ".git") {
            continue;
        }
        workdir_paths.insert(entry.path().to_path_buf());
    }

    // 3️⃣ 清空 index
    let mut index = Index::load(repo_path);
    index.clear();

    // 4️⃣ 递归恢复 tree 到工作区并更新 index
    let mut commit_paths = HashSet::new();
    restore_tree(repo_path, Path::new("."), tree_sha, &mut index, &mut commit_paths);

    // 5️⃣ 删除工作区中不属于 commit 的文件和空目录
    //    先删除文件，再尝试删除空目录
    for path in workdir_paths.difference(&commit_paths) {
        println!("Removing: {}", path.display());
        if path.is_file() {
            fs::remove_file(path).ok();
            println!("🗑️ Removed file not in target commit: {}", path.display());
        } else if path.is_dir() && path.as_os_str() != ".git" {
            // 尝试递归删除空目录
            fs::remove_dir_all(path).ok();
            println!("🗑️ Removed directory not in target commit: {}", path.display());
        }
    }
}

/// 递归恢复 tree
/// - 目录和文件都会加入 commit_paths，用于后续删除未在 commit 中的路径
fn restore_tree(
    repo_path: &Path,
    current_dir: &Path,
    tree_sha: &str,
    index: &mut Index,
    commit_paths: &mut HashSet<PathBuf>,
) {
    let tree_obj = Object::load(repo_path.to_str().unwrap(), tree_sha)
        .expect("Failed to load tree object");

    let entries = TreeProcessor::parse_tree(&tree_obj);

    for entry in entries {
        let path = current_dir.join(&entry.name);
        println!(
            "Restoring {}: {}",
            if entry.is_dir { "dir" } else { "file" },
            path.display()
        );

        if entry.is_dir {
            fs::create_dir_all(&path).expect("Failed to create directory");
            commit_paths.insert(path.clone()); // 目录也加入 commit_paths
            restore_tree(repo_path, &path, &entry.hash, index, commit_paths);
        } else {
            let blob_obj = Object::load(repo_path.to_str().unwrap(), &entry.hash)
                .expect("Failed to load blob object");
            fs::write_file_bytes(&path.to_str().unwrap(), &blob_obj)
                .expect("Failed to write file");
            index.stage_file(&path, &entry.hash);
            commit_paths.insert(path); // 文件加入 commit_paths
        }
    }
}
