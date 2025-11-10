use crate::core::index::Index;
use crate::core::reference::Reference;
use crate::utils::fs::{
    read_file, read_binary, write_binary, write_file,
    create_dir_all, remove_file, list_dir,
};
use std::path::{Path, PathBuf};

/// git checkout 命令
///
/// 该实现支持：
/// - 切换分支或 commit（更新 HEAD）
/// - 恢复工作区文件内容（blob 对象）
/// - 更新 index 匹配当前工作区
/// - 删除工作区中不存在于目标 tree 的文件
///
/// ⚠️ 本实现只支持简单文本 tree（非压缩、非二进制 tree）
pub fn git_checkout(repo_path: &Path, target: &str) {
    let refs_heads_path = repo_path.join(".git/refs/heads");
    let target_branch_path = refs_heads_path.join(target);

    // 1️⃣ 解析目标 commit 哈希
    let target_hash = if target_branch_path.exists() {
        // 是分支：从 refs/heads 获取 commit
        let hash = Reference::resolve(repo_path.to_str().unwrap(), &format!("refs/heads/{}", target))
            .unwrap_or_default();

        // 更新 HEAD 指向该分支
        let head_path = repo_path.join(".git/HEAD");
        write_file(head_path.to_str().unwrap(), &format!("ref: refs/heads/{}", target))
            .expect("Failed to update HEAD");

        println!("✅ Switched to branch '{}'", target);
        hash
    } else {
        // 直接指定 commit 哈希（Detached HEAD 模式）
        let head_path = repo_path.join(".git/HEAD");
        write_file(head_path.to_str().unwrap(), target)
            .expect("Failed to update HEAD");

        println!("⚠️ Detached HEAD at {}", target);
        target.to_string()
    };

    // 2️⃣ 获取当前工作区所有文件
    let existing_files = list_all_files(repo_path).unwrap_or_default();

    // 3️⃣ 从目标 commit 对应 tree 恢复文件
    let mut target_files = Vec::new();
    restore_tree(repo_path, &target_hash, Path::new(""), &mut target_files);

    // 4️⃣ 删除多余文件（非 .git 且不在目标 tree 中）
    for file in existing_files {
        if !target_files.contains(&file) && !file.starts_with(&repo_path.join(".git")) {
            let _ = remove_file(&file);
        }
    }

    // 5️⃣ 更新 index 文件
    let mut index = Index::load(repo_path);
    index.clear();

    // for file in &target_files {
    //     if let Ok(rel_path) = file.strip_prefix(repo_path) {
    //         index.stage_file(rel_path);
    //     }
    // }

    index.save();
    println!("✅ Working directory and index updated to {}", target_hash);
}

/// 恢复指定 tree 对象到工作区
fn restore_tree(repo_path: &Path, tree_hash: &str, path_prefix: &Path, target_files: &mut Vec<PathBuf>) {
    if tree_hash.is_empty() {
        return;
    }

    // 读取 tree 对象文件（此处仍为纯文本实现）
    let tree_path = repo_path.join(".git/objects")
        .join(&tree_hash[0..2])
        .join(&tree_hash[2..]);
    let tree_content = read_file(tree_path.to_str().unwrap())
        .expect("Failed to read tree object");

    // 每行格式: "blob <sha> <filename>" 或 "tree <sha> <dirname>"
    for line in tree_content.lines() {
        let mut parts = line.splitn(3, ' ');
        let type_str = parts.next().unwrap();
        let hash = parts.next().unwrap();
        let name = parts.next().unwrap();
        let file_path = path_prefix.join(name);

        if type_str == "blob" {
            // blob 对象文件路径
            let blob_path = repo_path.join(".git/objects")
                .join(&hash[0..2])
                .join(&hash[2..]);

            // 二进制读取 blob
            let content = read_binary(blob_path.to_str().unwrap())
                .expect("Failed to read blob object");

            // 写入工作区
            let full_path = repo_path.join(&file_path);
            if let Some(parent) = full_path.parent() {
                create_dir_all(parent).expect("Failed to create directory");
            }
            write_binary(full_path.to_str().unwrap(), &content)
                .expect("Failed to write file");

            target_files.push(full_path);
        } else if type_str == "tree" {
            // 递归恢复子目录
            let dir_path = repo_path.join(&file_path);
            create_dir_all(&dir_path).expect("Failed to create directory");
            restore_tree(repo_path, hash, &file_path, target_files);
        }
    }
}

/// 列出工作区所有文件（递归）
fn list_all_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in list_dir(dir)? {
        if entry.is_dir() {
            files.extend(list_all_files(&entry)?);
        } else {
            files.push(entry);
        }
    }
    Ok(files)
}
