mod ci;
mod diagnostic;
mod diagnostic_severity;
mod git;
mod init;
mod paths;

use std::{env, path::Path, process};

use crate::git::GitCommands;
use crate::init::InitCommand;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["inspect"] => {
            let git = GitCommands::new(Path::new("."));

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

            if let Err(error) = InitCommand::run(path) {
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
