use crate::core::index::Index;
use crate::utils::fs::remove_file;
use std::path::Path;
use walkdir::WalkDir; // 需要在 Cargo.toml 添加依赖 walkdir = "2"

/// git rm 命令（从暂存区和工作区删除文件）
///
/// # 功能
/// - 支持单文件、多文件、或目录（包括 "."）
/// - 从 index（暂存区）中移除指定文件
/// - 从工作区删除该文件（可选）
///
/// # 参数
/// - `repo_path`: 仓库根路径
/// - `paths`: 要删除的文件或目录路径列表
/// - `keep_in_workdir`: 是否保留工作区文件
pub fn git_rm(repo_path: &Path, paths: &[String], keep_in_workdir: bool) {
    let mut index = Index::load(repo_path);

    for path_str in paths {
        let path = Path::new(path_str);

        if path.is_dir() {
            // 遍历目录下所有文件
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();
                // 排除 .git 目录
                if file_path.components().any(|c| c.as_os_str() == ".git") {
                    continue;
                }
                remove_file_from_index_and_workdir(file_path, &mut index, keep_in_workdir);
            }
        } else {
            remove_file_from_index_and_workdir(path, &mut index, keep_in_workdir);
        }
    }
}

/// 单文件删除逻辑
fn remove_file_from_index_and_workdir(file_path: &Path, index: &mut Index, keep_in_workdir: bool) {
    if !index.entries.contains_key(file_path) {
        eprintln!("⚠️  Skipped: file not staged '{}'", file_path.display());
        return;
    }

    // 从 index 移除
    index.unstage_file(file_path);
    println!("✅ Removed from index: {}", file_path.display());

    // 删除工作区文件（如果未指定保留）
    if !keep_in_workdir && file_path.exists() {
        remove_file(file_path).expect("Failed to remove working file");
        println!("🗑️  Removed from working directory: {}", file_path.display());
    }
}
