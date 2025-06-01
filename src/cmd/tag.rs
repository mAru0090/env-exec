use crate::structs::*;
use crate::utils::*;
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
use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::Builder;
use toml;
use windows::Win32::System::Threading::{CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_CONSOLE};

pub fn tag_add_cmd(
    name: String,
    config_file: PathBuf,
    program: PathBuf,
    program_args: Vec<String>,
) -> Result<()> {
    // %USERPROFILE%\\.eec ディレクトリを取得
    let home_dir = env::var("USERPROFILE")?;
    let eec_dir = Path::new(&home_dir).join(".eec");
    create_dir_all(&eec_dir)?; // ディレクトリがなければ作成

    // タグファイルのパスを決定
    let tag_path = eec_dir.join(format!("{}.tag", name));

    // タグデータを構造体に格納
    let mut tag_data = TagData::new();
    tag_data.set_config_file(config_file);
    tag_data.set_program(program);
    tag_data.set_program_args(program_args);

    // バイナリとしてファイルに保存
    let encoded = bincode::serialize(&tag_data)?;
    let mut file = File::create(&tag_path)?;
    file.write_all(&encoded)?;

    println!("Tag saved to {:?}", tag_path);

    Ok(())
}

pub fn tag_remove_cmd(name: String) -> Result<()> {
    Ok(())
}
