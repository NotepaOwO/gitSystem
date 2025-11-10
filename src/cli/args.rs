use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn git_parse_args() -> ArgMatches {

    let app = Command::new("rust-git")
        .version("0.1.0")
        .author("NotepaOwO https://github.com/NotepaOwO")
        .about("A simple Git implementation in Rust")
        
        // 初始化仓库
        .subcommand(
            Command::new("init")
                .about("Initialize a new repository")
                .arg(
                    Arg::new("path")
                        .help("Path to repository")
                        .required(false),
                )
        )

        // 添加文件到暂存区
        .subcommand(
            Command::new("add")
                .about("Add file to the index")
                .arg(
                    Arg::new("files")
                        .help("File to add")
                        .required(true)
                        .num_args(1..),
                )
        )

        // 删除文件
        .subcommand(
            Command::new("rm")
                .about("Remove files from the working tree and the index")
                .arg(
                    Arg::new("files")
                        .help("File to remove")
                        .required(true)
                        .num_args(1..),
                )
                .arg(
                    Arg::new("force")
                        .help("Force removal")
                        .short('f')
                        .long("force")
                        .action(ArgAction::SetTrue)
                        .required(false),
                )
        )

        // 提交更改
        .subcommand(
            Command::new("commit")
                .about("Record changes to the repository")
                .arg(
                    Arg::new("message")
                        .short('m')
                        .long("message")
                        .help("Commit message")
                        .required(true),
                )
        )

        // 分支管理
        .subcommand(
            Command::new("branch")
                .about("List, create, or delete branches")
                .arg(
                    Arg::new("branch_name")
                        .help("Branch name")
                        .required(false),
                )
                .arg(
                    Arg::new("delete")
                        .short('d')
                        .long("delete")
                        .help("Delete branch")
                        .action(ArgAction::SetTrue)
                        .required(false),
                )
        )

        // 切换分支或恢复工作区文件
        .subcommand(
            Command::new("checkout")
                .about("Switch branches or restore working tree files")
                .arg(
                    Arg::new("target")
                        .help("Branch or commit to checkout")
                        .required(true),
                )
        )

        // 合并分支
        .subcommand(
            Command::new("merge")
                .about("Join two or more development histories together")
                .arg(
                    Arg::new("branch_name")
                        .help("Branch to merge")
                        .required(true),
                )
        )

        // 拉取数据
        .subcommand(
            Command::new("fetch")
                .about("Download objects and refs from another repository")
                .arg(
                    Arg::new("remote_url")
                        .help("Remote repository URL")
                        .required(true),
                )
        )

        // 拉取并合并
        .subcommand(
            Command::new("pull")
                .about("Fetch from and integrate with another repository or a local branch")
                .arg(
                    Arg::new("remote_url")
                        .help("Remote repository URL")
                        .required(true),
                )
        )

        // 推送更改
        .subcommand(
            Command::new("push")
                .about("Update remote refs along with associated objects")
                .arg(
                    Arg::new("remote_url")
                        .help("Remote repository URL")
                        .required(true),
                )
        );

    // 解析命令行参数并返回
    app.get_matches()
}