use std::fs;
use std::env;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// 创建目录（递归创建）
pub fn create_dir(path: &str) {
    if !Path::new(path).exists() {
        fs::create_dir_all(path).unwrap();
    }
}

/// 递归创建目录（如果不存在）
pub fn create_dir_all(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 递归删除目录及其所有内容
pub fn remove_dir_all(path: &Path) -> std::io::Result<()> {
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// 列出指定目录下的所有文件和子目录
/// 
/// # 参数
/// - `dir_path`: 要读取的目录路径
///
/// # 返回
/// - `Ok(Vec<PathBuf>)`: 所有文件（或子目录）的完整路径
/// - `Err(String)`: 若目录不存在或读取失败
pub fn list_dir(dir_path: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir_path.exists() {
        return Err(format!("Directory not found: {}", dir_path.display()));
    }

    let mut entries = Vec::new();

    match fs::read_dir(dir_path) {
        Ok(read_dir) => {
            for entry in read_dir {
                match entry {
                    Ok(e) => entries.push(e.path()),
                    Err(err) => return Err(format!("Failed to read entry: {}", err)),
                }
            }
            Ok(entries)
        }
        Err(err) => Err(format!("Failed to read directory {}: {}", dir_path.display(), err)),
    }
}


/// 创建空文件
pub fn create_file(path: &str) {
    if !Path::new(path).exists() {
        fs::File::create(path).unwrap();
    }
}

/// 读取文件内容为字符串
pub fn read_file(path: &str) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// 写入内容到文件（覆盖模式）
pub fn write_file(path: &str, data: &str) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// 追加内容到文件末尾
pub fn append_file(path: &str, data: &str) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// 删除文件
pub fn remove_file(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// 读取二进制文件
pub fn read_file_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// 写入二进制文件
pub fn write_file_bytes(path: &str, data: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, data)
}

/// 读取二进制文件
pub fn read_binary(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// 写入二进制文件
pub fn write_binary(path: &str, data: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

/// 检查目录或文件是否存在
pub fn check_path_exists(path: &str) -> bool {
    Path::new(path).exists()
}


/// 递归向上查找 `.git` 文件夹，找到则返回仓库根路径
pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let git_dir = current.join(".git");

        if git_dir.exists() && git_dir.is_dir() {
            // 找到了仓库根路径
            return Some(current);
        }

        // 到达系统根目录（例如 C:\ 或 /）
        if !current.pop() {
            return None;
        }
    }
}

/// 获取当前仓库根路径（从当前工作目录起）
pub fn get_repo_path() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    find_repo_root(&cwd)
}

/// utils/fs.rs 中读取 HEAD 文件内容
pub fn get_current_branch(repo_path: &Path) -> Option<String> {
    let head_path = repo_path.join(".git").join("HEAD");
    if head_path.exists() {
        let content = read_file(head_path.to_str().unwrap()).ok()?;
        if content.starts_with("ref: refs/heads/") {
            return Some(content["ref: refs/heads/".len()..].trim().to_string());
        }
    }
    None
}