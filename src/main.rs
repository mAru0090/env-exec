// ====================
// ====================
// ====================
// インポート部
// ====================
// ====================
// ====================
mod cmd;
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
        /// タグ名
        #[arg(short, long)]
        name: String,
        /// 環境設定ファイルパス
        #[arg(long)]
        config_file: PathBuf,
        /// 実行対象のプログラムパス
        #[arg(long)]
        program: PathBuf,
        /// 実行時の引数（スペース区切りで渡す）
        #[arg()]
        program_args: Vec<String>,
    },
    List,
    Remove {
        /// タグ名
        #[arg(short, long)]
        name: String,
    },
}
// ====================
// リスタート用コマンド
// ====================
#[derive(Subcommand, Debug, Clone)]
enum RestartCommand {
    Run {
        #[arg(short, long)]
        config_file: PathBuf,
        #[arg(short, long, default_value = "eec", required = false)]
        exec_path: PathBuf,

        #[arg(long, required = false)]
        arg0_program: Option<PathBuf>,
        #[arg(short, long, required = false)]
        arg1_program_args: Option<Vec<String>>,
    },
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
            cmd::run::run_cmd(
                config_file.clone(),
                program.clone(),
                program_args.clone(),
                tag.clone(),
            )?;
        }
        Commands::Tag(tag_cmd) => match tag_cmd {
            TagCommand::Add {
                name,
                config_file,
                program,
                program_args,
            } => {
                cmd::tag::tag_add_cmd(
                    name.clone(),
                    config_file.clone(),
                    program.clone(),
                    program_args.clone(),
                )?;
            }
            TagCommand::List => {}
            TagCommand::Remove { .. } => {}
        },
        Commands::Restart(restart_cmd) => match restart_cmd {
            RestartCommand::Run {
                config_file,
                exec_path,
                arg0_program,
                arg1_program_args,
            } => {
                cmd::restart::restart_run_cmd(
                    config_file.clone(),
                    exec_path.clone(),
                    arg0_program.clone(),
                    arg1_program_args.clone(),
                )?;
            }
        },
    }

    println!("何かキーを押してください...");
    let mut _input = String::new();
    io::stdin().read_line(&mut _input).unwrap();

    Ok(())
}
