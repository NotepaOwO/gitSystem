use crate::cli::args::git_parse_args;
use crate::commands::init::git_init;
use crate::commands::add::git_add;
use crate::commands::rm::git_rm;
use crate::commands::commit::git_commit;
use crate::commands::branch::git_branch;
use crate::commands::checkout::git_checkout;
use crate::commands::merge::git_merge;
use crate::commands::fetch::git_fetch;
use crate::commands::pull::git_pull;
use crate::commands::push::git_push;
use crate::utils::fs::get_repo_path; // 需要你在 utils/fs.rs 实现

pub fn git_execute() {
    // === 解析命令行参数 ===
    let matches = git_parse_args();

    // === 统一获取 repo_path（非 init 命令） ===
    let repo_path = match matches.subcommand_name() {
        Some("init") => None,
        Some(_) => Some(get_repo_path().expect("❌ Not a git repository")),
        None => None,
    };

    // === 执行对应子命令 ===
    match matches.subcommand() {
        // ------------------ init ------------------
        Some(("init", sub_m)) => {
            let path = sub_m
                .get_one::<String>("path")
                .map(|s| s.as_str())
                .unwrap_or(".");
            git_init(path);
        }

        // ------------------ add ------------------
        Some(("add", sub_m)) => {
            let files: Vec<String> = sub_m.get_many::<String>("files")
                .unwrap()
                .map(|s| s.to_string())
                .collect();

            git_add(&repo_path.unwrap(), &files);
        }

        // ------------------ rm ------------------
        Some(("rm", sub_m)) => {
            // 获取所有文件参数（支持多文件）
            let files: Vec<String> = sub_m
                .get_many::<String>("file")
                .expect("Missing <file>")
                .map(|s| s.to_string())
                .collect();

            let keep_in_workdir = sub_m.get_flag("keep"); // keep 选项
            git_rm(&repo_path.unwrap(), &files, keep_in_workdir);
        }


        // ------------------ commit ------------------
        Some(("commit", sub_m)) => {
            let msg = sub_m.get_one::<String>("message").expect("Missing <message>");
            git_commit(&repo_path.unwrap(), msg);
        }

        // // ------------------ branch ------------------
        // Some(("branch", sub_m)) => {
        //     let branch_name = sub_m
        //         .get_one::<String>("branch_name")
        //         .map(|s| s.as_str());
        //     let delete = sub_m.get_flag("delete");
        //     git_branch(&repo_path.unwrap(), branch_name, delete);
        // }

        // // ------------------ checkout ------------------
        // Some(("checkout", sub_m)) => {
        //     let target = sub_m.get_one::<String>("target").expect("Missing <target>");
        //     git_checkout(&repo_path.unwrap(), target);
        // }

        // // ------------------ merge ------------------
        // Some(("merge", sub_m)) => {
        //     let branch_name = sub_m.get_one::<String>("branch_name").expect("Missing <branch>");
        //     git_merge(repo_path.unwrap(), branch_name);
        // }

        // // ------------------ fetch ------------------
        // Some(("fetch", sub_m)) => {
        //     let remote_url = sub_m.get_one::<String>("remote_url").expect("Missing <url>");
        //     git_fetch(repo_path.unwrap(), remote_url);
        // }

        // // ------------------ pull ------------------
        // Some(("pull", sub_m)) => {
        //     let remote_url = sub_m.get_one::<String>("remote_url").expect("Missing <url>");
        //     git_pull(repo_path.unwrap(), remote_url);
        // }

        // // ------------------ push ------------------
        // Some(("push", sub_m)) => {
        //     let remote_url = sub_m.get_one::<String>("remote_url").expect("Missing <url>");
        //     git_push(repo_path.unwrap(), remote_url);
        // }

        // ------------------ 未知命令 ------------------
        _ => {
            println!("❌ Unknown or missing command. Try `--help` for usage.");
        }
    }
}
