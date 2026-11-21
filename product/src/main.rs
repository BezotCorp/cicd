mod ci;
mod collect;
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
        ["init", options @ ..] => {
            use crate::git::GitWorkflow;

            let mut path = Path::new(".");
            let mut has_path = false;
            let mut apply = false;
            let mut profile = None;
            let mut index = 0;

            while index < options.len() {
                match options[index] {
                    "--apply" => {
                        if apply {
                            eprintln!("Duplicate --apply option");
                            process::exit(2);
                        }
                        apply = true;
                    }
                    "--profile" => {
                        if profile.is_some() {
                            eprintln!("Duplicate --profile option");
                            process::exit(2);
                        }

                        index += 1;

                        profile = match options.get(index).copied() {
                            Some("simple") => Some(GitWorkflow::Simple),
                            Some("pro") => Some(GitWorkflow::Pro),
                            _ => {
                                eprintln!("Expected --profile simple|pro");
                                process::exit(2);
                            }
                        };
                    }
                    value if !value.starts_with('-') && !has_path => {
                        path = Path::new(value);
                        has_path = true;
                    }
                    _ => {
                        eprintln!("Invalid init argument: {}", options[index]);
                        process::exit(2);
                    }
                }

                index += 1;
            }

            if let Err(error) = InitCommand::run(path, apply, profile) {
                eprintln!("Init failed: {error}");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: cicd inspect | cicd init [path] [--apply] [--profile simple|pro]");
            process::exit(2);
        }
    }
}
