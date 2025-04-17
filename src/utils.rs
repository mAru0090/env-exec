// ====================
// ====================
// インポート部
// ====================
// ====================
use anyhow::Result;
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
use std::path::Path;
use std::process::{Child, Command, Stdio};
use tempfile::Builder;
use toml;
use windows::Win32::System::Threading::{CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_CONSOLE};
use crate::structs::*;
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
