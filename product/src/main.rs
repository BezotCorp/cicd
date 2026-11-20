mod ci_config_file;
mod ci_provider;
mod ci_scanner;
mod diagnostic;
mod diagnostic_severity;
mod git;
mod git_branches;
mod git_error;
mod git_info;
mod git_remote;
mod git_status;
mod init_action;
mod init_analyzer;
mod init_command;
mod init_diagnosis;
mod init_error;
mod init_plan;
mod init_planner;
mod init_step;

#[cfg(test)]
#[path = "tests/ci_provider.rs"]
mod ci_provider_tests;

use std::{env, path::Path, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["inspect"] => {
            let git = git::Git::new(Path::new("."));

            match git.inspect() {
                Ok(info) => {
                    println!("Repository: {}", info.root().display());
                    println!("Branch: {}", info.branch().unwrap_or("(detached HEAD)"));
                    println!("HEAD: {}", info.head().unwrap_or("(no commit)"));

                    match git.status() {
                        Ok(status) => {
                            println!("--- Git Status ---");
                            println!("Staged: {:?}", status.staged());
                            println!("Unstaged: {:?}", status.unstaged());
                            println!("Untracked: {:?}", status.untracked());
                            println!("Deleted: {:?}", status.deleted());
                            println!("Conflicted: {:?}", status.conflicted());
                        }
                        Err(error) => {
                            eprintln!("Git status failed: {error}");
                            process::exit(1);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Git inspection failed: {error}");
                    process::exit(1);
                }
            }
        }
        ["init"] | ["init", _] => {
            let path = args.get(1).map_or(Path::new("."), |value| Path::new(value));

            if let Err(error) = init_command::InitCommand::run(path) {
                eprintln!("Init failed: {error}");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: cicd inspect | cicd init [path]");
            process::exit(2);
        }
    }
}
