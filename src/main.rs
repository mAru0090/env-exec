// ====================
// ====================
// ====================
// インポート部
// ====================
// ====================
// ====================
mod structs;
mod utils;
use anyhow::Result;
use clap::{Parser, Subcommand};
use log::*;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use structs::*;
use tempfile::Builder;
use toml;
use utils::*;
use windows::Win32::System::Threading::{CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_CONSOLE};
// ====================
// メイン引数格納用構造体
// ====================
#[derive(Parser, Clone)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// ====================
// 各コマンドリスト
// ====================
#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Run {
        #[arg(short, long, required = false)]
        config_file: Option<PathBuf>,
        #[arg(short, long, required = false)]
        program: Option<PathBuf>,
        #[arg(required = false, required = false)]
        program_args: Option<Vec<String>>,
        #[arg(long, required = false)]
        tag: Option<String>,
    },
    Clear {
    },
    #[command(subcommand)]
    Tag(TagCommand),
    #[command(subcommand)]
    Restart(RestartCommand),
}

// ====================
// タグ用コマンド
// ====================
#[derive(Subcommand, Debug, Clone)]
enum TagCommand {
    Add {
        #[arg(short, long)]
        name: String,
    },
    List,
    Clear,
}
// ====================
// リスタート用コマンド
// ====================
#[derive(Subcommand, Debug, Clone)]
enum RestartCommand {
    Run,
    List,
}

// ====================
// メイン関数
// ====================
fn main() -> Result<()> {
    let _ = SimpleLogger::new().init();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run {
            config_file,
            program,
            program_args,
            tag,
        } => {
            let args: Vec<String> = env::args().collect();
            let self_program = &args[0];
            let mut temp_data = TempData::new();

            let tag_data = match tag {
                Some(ref t) if !t.is_empty() => match read_tag_data(t) {
                    Some(data) => {
                        debug!("Found tag data: name: {:?} {:?}",t,data);
                        data
                    },
                    None => return Err(anyhow::anyhow!("タグデータ読み込みに失敗しました")),
                },
                _ => TagData::new(),
            };
            /*
                        let config_file = if config_file.is_some() {
                            config_file.clone().unwrap()
                        } else {
                            PathBuf::new()
                        };
                        let program = if program.is_some() {
                            program.clone().unwrap()
                        } else {
                            PathBuf::new()
                        };
              let p_args = if let Some(args) = program_args.clone() {
                            args
                        } else {
                            vec![]
                        };
            */
            // tag_data に値があればそちらを優先する（優先度: tag_data > CLI 引数）
            let config_file = if !tag_data.get_config_file().as_os_str().is_empty() {
                tag_data.get_config_file()
            } else if let Some(ref cf) = config_file {
                cf.clone()
            } else {
                PathBuf::new()
            };

            let program = if !tag_data.get_program().as_os_str().is_empty() {
                tag_data.get_program()
            } else if let Some(ref p) = program {
                p.clone()
            } else {
                PathBuf::new()
            };

            let p_args = if !tag_data.get_program_args().is_empty() {
                tag_data.get_program_args()
            } else if let Some(args) = program_args.clone() {
                args
            } else {
                vec![]
            };

            let temp_prefix = format!(
                "{}_{}_",
                Path::new(self_program)
                    .file_stem()
                    .unwrap_or_else(|| OsStr::new("env-exec"))
                    .to_string_lossy(),
                program
                    .file_stem()
                    .unwrap_or_else(|| OsStr::new("program"))
                    .to_string_lossy()
            );

            let config: Config = read_toml(&config_file)?;
            let mut temp_file = Builder::new()
                .prefix(&temp_prefix)
                .suffix(".tmp")
                .keep(true)
                .tempfile()?;

            debug!("Created temp file: {:?}", temp_file.path());
            // eec_manifest.txt を一時ファイルディレクトリに作成
            let manifest_path = write_to_manifest(temp_file.path(), std::process::id())?;
            debug!("Created manifest file: {:?}", manifest_path);

            let current_path = env::var("Path").unwrap_or_default();
            let mut new_path = current_path.clone();

            for env_var in config.get_envs() {
                match env_var {
                    EnvVar::Single(ref env_pair) => {
                        if env_pair.len() == 2 {
                            env::set_var(&env_pair[0], expand_env_variables(&env_pair[1]));
                        }
                    }
                    EnvVar::Multiple(ref key, ref values) => {
                        let expanded_values = expand_env_variables_vec(values);
                        env::set_var(key, expanded_values.join(";"));
                    }
                }
            }

            for path in config.get_paths() {
                let expanded_path = expand_env_variables(&path);
                if !expanded_path.trim().is_empty() {
                    new_path.push(';');
                    new_path.push_str(&expanded_path);
                }
            }

            env::set_var("Path", new_path);
            let mut command = Command::new(&program);
            debug!("program_args = {:?}", p_args);
            command.args(p_args.clone());
            command
                // .creation_flags(CREATE_NEW_CONSOLE.0)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            let mut child: Child = command.spawn()?;
            let child_id = child.id();
            debug!("Sub process started ppid: {:?}", child_id);

            temp_data.set_parent_pid(std::process::id());
            temp_data.set_child_pid(child_id);
            temp_data.set_config_file(config_file.to_string_lossy().to_string());
            temp_data.set_program(program.to_string_lossy().to_string());
            temp_data.set_program_args(p_args.clone());

            let encoded: Vec<u8> = bincode::serialize(&temp_data)?;
            temp_file.write_all(&encoded)?;
            debug!("Written temp file: {:?}", temp_data);

            let temp_path = temp_file.path().to_path_buf();
            let status = child.wait()?;
            debug!("Sub process exited with: {}", status);
            if let Err(e) = std::fs::remove_file(&temp_path) {
                return Err(anyhow::anyhow!("Failed to delete temp file: {}", e));
            }
            debug!("Deleted temp file: {}", temp_path.display());
        }
        Commands::Clear {} => {
        }
        Commands::Tag(tag_cmd) => match tag_cmd {
            TagCommand::Add { name } => {
                println!("Add tag: {}", name);
            }
            TagCommand::List => {
                println!("Listed tag");
            }
            TagCommand::Clear => {
            }
        },
        Commands::Restart(restart_cmd) => match restart_cmd {
            _ => println!("Restart Process（）"),
        },
    }

    println!("何かキーを押してください...");
    let mut _input = String::new();
    io::stdin().read_line(&mut _input).unwrap();
    Ok(())
}
