mod ci;
mod collect;
mod data_sources;
mod diagnostic;
mod diagnostic_severity;
mod git;
mod init;
mod paths;

use std::{env, path::Path, process};

use crate::collect::ProjectCollector;
use crate::git::GitCommands;
use crate::init::InitCommand;
use common::data_source_kind::DataSourceKind;

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
        ["collect", options @ ..] => {
            let mut path = Path::new(".");
            let mut has_path = false;
            let mut source = None;
            let mut index = 0;

            while index < options.len() {
                match options[index] {
                    "--source" => {
                        if source.is_some() {
                            eprintln!("Duplicate --source option");
                            process::exit(2);
                        }

                        index += 1;
                        source = match options.get(index).copied() {
                            Some("git") => Some("git"),
                            Some("ci") => Some("ci"),
                            Some("all") => Some("all"),
                            _ => {
                                eprintln!("Expected --source git|ci|all");
                                process::exit(2);
                            }
                        };
                    }
                    value if !value.starts_with('-') && !has_path => {
                        path = Path::new(value);
                        has_path = true;
                    }
                    _ => {
                        eprintln!("Invalid collect argument: {}", options[index]);
                        process::exit(2);
                    }
                }
                index += 1;
            }

            let sources: Vec<DataSourceKind> = match source {
                Some("git") => vec![DataSourceKind::Git],
                Some("ci") => vec![DataSourceKind::Ci],
                Some("all") => DataSourceKind::ALL.to_vec(),
                None => {
                    let configured = match crate::data_sources::DataSources::load() {
                        Ok(value) => value,
                        Err(error) => {
                            eprintln!("Data sources configuration failed: {error}");
                            process::exit(1);
                        }
                    };

                    configured.enabled_sources().collect()
                }
                _ => {
                    eprintln!("Invalid source");
                    process::exit(2);
                }
            };

            match ProjectCollector::collect(path, &sources) {
                Ok(report) => {
                    println!("=== Collection ===");
                    println!(
                        "Sources: {}",
                        report
                            .sources()
                            .iter()
                            .map(|source| source.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );

                    if let Some(git) = report.git() {
                        println!("Git repository: {}", git.info().root().display());
                        println!(
                            "Branch: {}",
                            git.info().branch().unwrap_or("(detached HEAD)")
                        );
                        println!("HEAD: {}", git.info().head().unwrap_or("(no commit)"));
                        println!("Local branches: {}", git.branches().local().join(", "));
                        println!("Remote branches: {}", git.branches().remote().join(", "));
                        println!("Remotes: {}", git.remotes().len());
                        println!(
                            "Working tree: {} staged, {} unstaged, {} untracked",
                            git.status().staged().len(),
                            git.status().unstaged().len(),
                            git.status().untracked().len()
                        );
                        println!("Git hosting: {:?}", git.hosting());
                        println!("Suggested CI provider: {:?}", git.suggested_provider());
                    }

                    if let Some(ci) = report.ci() {
                        println!("CI scan root: {}", ci.root().display());
                        println!(
                            "cicd configuration: {}",
                            if ci.already_initialized() {
                                "present"
                            } else {
                                "absent"
                            }
                        );
                        println!("CI files: {}", ci.files().len());

                        for file in ci.files() {
                            println!("  {} [{}]", file.path().display(), file.provider());
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Collection failed: {error}");
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
            eprintln!(
                "Usage: cicd inspect | cicd collect [path] [--source git|ci|all] | cicd init [path] [--apply] [--profile simple|pro]"
            );
            process::exit(2);
        }
    }
}
