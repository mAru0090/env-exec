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
use std::fs::File;
use std::io::{self, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
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
        #[arg(short, long)]
        config_file: PathBuf,
        #[arg(short, long)]
        program: PathBuf,
        #[arg(required = false)]
        program_args: Option<Vec<String>>,
    },
    #[command(subcommand)]
    Tag(TagCommand),
    Restart {},
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
        } => {
            let args: Vec<String> = env::args().collect();
            let self_program = &args[0];
            let mut temp_data = TempData::new();
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
            let mut temp_file = Builder::new()
                .prefix(&temp_prefix)
                .suffix(".tmp")
                .keep(true)
                .tempfile()?;

            debug!("Created temp file: {:?}", temp_file.path());

            let config: Config = read_toml(config_file)?;
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
            let mut command = Command::new(program);
            debug!("program_args = {:?}", program_args);
            let p_args = if let Some(args) = program_args.clone() {
                args
            } else {
                vec![]
            };
            command.args(p_args.clone());
            command
                .creation_flags(CREATE_NEW_CONSOLE.0)
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
                error!("Failed to delete temp file: {}", e);
            }
        }
        Commands::Tag(tag_cmd) => match tag_cmd {
            TagCommand::Add { name } => {
                println!("Add tag: {}", name);
            }
            TagCommand::List => {
                println!("Listed tag");
            }
        },
        Commands::Restart {} => {
            println!("Restart Process（）");
        }
    }

    println!("何かキーを押してください...");
    let mut _input = String::new();
    io::stdin().read_line(&mut _input).unwrap();
    Ok(())
}