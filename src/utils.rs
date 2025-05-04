// ====================
// ====================
// インポート部
// ====================
// ====================
use crate::structs::*;
use anyhow::Result;
use log::*;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::{self, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tempfile::Builder;
use toml;
use windows::Win32::System::Threading::{CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_CONSOLE};
// ====================
// tomlファイルを開き、Config構造体を返す関数
// ====================
pub fn read_toml<P>(filename: P) -> Result<Config, toml::de::Error>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename).map_err(|e| toml::de::Error::custom(e.to_string()))?;
    let mut contents = String::new();
    io::Read::read_to_string(&mut file, &mut contents).unwrap();
    toml::de::from_str(&contents)
}

// ====================
// 入力に含まれる環境変数等、キャプチャーしてStringを返す関数
// ====================
pub fn expand_env_variables(input: &str) -> String {
    let re = Regex::new(r"\$\(([^)]+)\)").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        env::var(&caps[1]).unwrap_or_else(|_| "".to_string())
    })
    .to_string()
}
// ====================
// expand_env_variablesのVec版
// ====================
pub fn expand_env_variables_vec(inputs: &[String]) -> Vec<String> {
    inputs.iter().map(|s| expand_env_variables(s)).collect()
}

// ====================
// manifestファイルへ書き込む関数
// ====================
pub fn write_to_manifest(temp_file_path: &Path, eec_pid: u32) -> io::Result<PathBuf> {
    let manifest_dir = temp_file_path.parent().unwrap(); // 一時ファイルと同じディレクトリ
    let manifest_path = manifest_dir.join("eec_manifest.txt");

    // ファイルが存在しない場合は作成、既存の場合は追記
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&manifest_path)?;

    // 一時ファイルのパスとeecのPIDを追記
    writeln!(file, "{} {}", temp_file_path.display(), eec_pid)?;

    // 作成したマニフェストファイルのパスを返す
    Ok(manifest_path)
}

// ====================
// tag名からファイルを読み込みOptionで返す関数
// ====================
pub fn read_tag_data(tag_name: &str) -> Option<TagData> {
    // %USERPROFILE%\\.eec\\{tag_name}.tag を構築
    let home_dir = env::var("USERPROFILE").ok()?;
    let tag_path = PathBuf::from(home_dir)
        .join(".eec")
        .join(format!("{}.tag", tag_name));

    // ファイルを開いて内容を読み込む
    let mut file = File::open(tag_path).ok()?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).ok()?;

    // バイナリデータをデコード（bincode v1）
    bincode::deserialize(&buffer).ok()
}
