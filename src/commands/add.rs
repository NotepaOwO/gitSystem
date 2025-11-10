use crate::core::index::Index;
use crate::core::object::Object;
use crate::utils::fs::read_file_bytes;
use std::path::Path;
use walkdir::WalkDir; // ✅ 需要在 Cargo.toml 中添加依赖：walkdir = "2"

/// git add 命令（将文件加入暂存区）
///
/// # 功能
/// - 支持单文件、多文件、或目录（包括 "."）
/// - 读取文件内容并生成 blob 对象
/// - 更新 index（暂存区）
///
/// # 参数
/// - `repo_path`: 仓库根路径
/// - `files`: 要添加的文件或目录路径列表
pub fn git_add(repo_path: &Path, files: &[String]) {
    let mut index = Index::load(repo_path);

    for file in files {
        let path = Path::new(file);

        // ✅ 如果是目录（例如 "."），递归遍历所有文件
        if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();
                // ❌ 排除 .git 目录
                if file_path.components().any(|c| c.as_os_str() == ".git") {
                    continue;
                }
                stage_single_file(repo_path, file_path, &mut index);
            }
        } else {
            // ✅ 单个文件
            stage_single_file(repo_path, path, &mut index);
        }
    }

    println!("✅ Added {} file(s) to staging area", index.entries.len());
}

/// 单文件暂存逻辑
fn stage_single_file(repo_path: &Path, file_path: &Path, index: &mut Index) {
    if !file_path.exists() {
        eprintln!("⚠️  Skipped: file not found '{}'", file_path.display());
        return;
    }

    // 1️⃣ 读取文件内容
    let content = match read_file_bytes(file_path.to_str().unwrap()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  Failed to read '{}': {}", file_path.display(), e);
            return;
        }
    };

    // 2️⃣ 保存 Blob 对象
    let sha = Object::Blob(content.clone()).save(repo_path.to_str().unwrap());

    // 3️⃣ 更新 Index
    index.stage_file(file_path, &sha);
    println!("✅ Staged file: {}", file_path.display());
}
