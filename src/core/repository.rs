use crate::utils::fs::{create_dir, create_file, check_path_exists};
use std::path::Path;

/// Git 仓库结构体
pub struct Repository {
    pub path: String, // 仓库路径
}

impl Repository {
    /// 初始化一个新的 Git 仓库
    pub fn init(path: &str) -> Self {
        let git_dir = Path::new(path).join(".git");

        // =========================
        // 创建核心目录结构
        // =========================
        create_dir(git_dir.to_str().unwrap());                           // .git 根目录
        create_dir(git_dir.join("objects").to_str().unwrap());           // 存储 Git 对象（blob/tree/commit/tag）
        create_dir(git_dir.join("objects").join("info").to_str().unwrap()); // objects/info，存储附加信息
        create_dir(git_dir.join("objects").join("pack").to_str().unwrap()); // objects/pack，存储打包对象
        create_dir(git_dir.join("refs").to_str().unwrap());              // 存储指针引用目录
        create_dir(git_dir.join("refs").join("heads").to_str().unwrap());   // 本地分支指针目录
        create_dir(git_dir.join("refs").join("tags").to_str().unwrap());    // 标签指针目录
        create_dir(git_dir.join("refs").join("remotes").to_str().unwrap()); // 远程跟踪分支指针目录
        create_dir(git_dir.join("hooks").to_str().unwrap());             // 钩子脚本目录

        // =========================
        // 创建核心文件
        // =========================
        create_file(git_dir.join("HEAD").to_str().unwrap());          // HEAD 文件
        create_file(git_dir.join("config").to_str().unwrap());        // 配置文件
        create_file(git_dir.join("description").to_str().unwrap());   // 仓库描述文件
        create_file(git_dir.join("index").to_str().unwrap());         // 暂存区索引文件

        // 设置 HEAD 默认指向 main 分支
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/master\n").unwrap();

        Repository { path: path.to_string() }
    }

    /// 检查指定目录是否是一个 Git 仓库
    pub fn is_git_repo(path: &str) -> bool {
        let git_dir = Path::new(path).join(".git");
        check_path_exists(git_dir.to_str().unwrap()) // 检测 .git 目录是否存在
    }
}
