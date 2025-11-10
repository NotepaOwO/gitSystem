// commands/init.rs
use crate::core::repository::Repository; // 引入核心仓库模块
use std::path::Path;

/// Git 初始化命令处理器
pub fn git_init(target_path: &str) {
    // 1️⃣ 检查是否已存在 .git 仓库
    let git_dir = Path::new(target_path).join(".git");
    if git_dir.exists() {
        println!("Error: Git repository already exists at {}", target_path);
        return;
    }

    // 2️⃣ 调用核心模块初始化仓库
    let _repo = Repository::init(target_path);

    // 3️⃣ 输出成功信息
    println!("Initialized empty Git repository in {}/.git", target_path);
}
