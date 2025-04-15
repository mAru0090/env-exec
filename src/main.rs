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

#[derive(Debug, Serialize, Deserialize)]
struct TempData {
    parent_ppid: u32,
    child_ppid: u32,
    config_file: String,
    program: String,
    program_args: Vec<String>,
}
impl TempData {
    fn new() -> Self {
        Self {
            parent_ppid: 0,
            child_ppid: 0,
            config_file: String::new(),
            program: String::new(),
            program_args:Vec::new(),
        }
    }
    fn set_config_file(&mut self, config_file: String) {
        self.config_file = config_file;
    }
    fn set_program(&mut self, program: String) {
        self.program = program;
    }
    fn set_parent_ppid(&mut self, ppid: u32) {
        self.parent_ppid = ppid;
    }
    fn set_child_ppid(&mut self, ppid: u32) {
        self.child_ppid = ppid;
    }
    fn set_program_args(&mut self, args: Vec<String>) {
        self.program_args = args;
    }
}
#[derive(Debug, Deserialize)]
struct Config {
    paths: Vec<String>,
    envs: Vec<EnvVar>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EnvVar {
    Single(Vec<String>),
    Multiple(String, Vec<String>),
}

fn main() -> Result<()> {
    let _ = SimpleLogger::new().init();
    let args: Vec<String> = env::args().collect();
    let self_program = &args[0];
    let mut temp_data = TempData::new();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <config_file> <program> [command...]",
            self_program
        );
        std::process::exit(1);
    }

    let config_file = &args[1];
    let program = &args[2];
    let program_args = &args[3..];
    
    let temp_prefix = format!(
        "{}_{}_",
        Path::new(self_program)
            .file_stem()
            .unwrap_or_else(|| OsStr::new("env-exec"))
            .to_string_lossy(),
        Path::new(program)
            .file_stem()
            .unwrap_or_else(|| OsStr::new("program"))
            .to_string_lossy()
    );
    let mut temp_file = Builder::new()
        .prefix(&temp_prefix)
        .suffix(".tmp")
        .keep(true)
        .tempfile()?;

    info!("Created temp file: {:?}", temp_file.path());

    let config: Config = read_toml(config_file)?;

    let current_path = env::var("Path").unwrap_or_default();
    let mut new_path = current_path.clone();

    for env_var in config.envs {
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

    for path in config.paths {
        let expanded_path = expand_env_variables(&path);
        if !expanded_path.trim().is_empty() {
            new_path.push(';');
            new_path.push_str(&expanded_path);
        }
    }

    env::set_var("Path", new_path);
    let mut command = Command::new(program);
    command.args(program_args);
    command
        //.creation_flags((CREATE_BREAKAWAY_FROM_JOB.0) | (CREATE_NEW_CONSOLE.0))
        .creation_flags(CREATE_NEW_CONSOLE.0)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child: Child = command.spawn()?;
    let child_id = child.id();
    info!("Sub process started ppid: {:?}", child_id);
    
    // 書き込みデータを設定
    temp_data.set_parent_ppid(std::process::id());
    temp_data.set_child_ppid(child_id);
    temp_data.set_config_file(config_file.to_string());
    temp_data.set_program(program.to_string());
    temp_data.set_program_args(program_args.to_vec());
 
    // TempData をバイナリにシリアライズして書き込み
    let encoded: Vec<u8> = bincode::serialize(&temp_data)?;
    temp_file.write_all(&encoded)?;
    info!("Written temp file: {:?}", temp_data);

    let temp_path = temp_file.path().to_path_buf();
    let status = child.wait()?;
    info!("Sub process exited with: {}", status);

    if let Err(e) = std::fs::remove_file(&temp_path) {
        error!("Failed to delete temp file: {}", e);
    }

    Ok(())
}

fn read_toml<P>(filename: P) -> Result<Config, toml::de::Error>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename).map_err(|e| toml::de::Error::custom(e.to_string()))?;
    let mut contents = String::new();
    io::Read::read_to_string(&mut file, &mut contents).unwrap();
    toml::de::from_str(&contents)
}

fn expand_env_variables(input: &str) -> String {
    let re = Regex::new(r"\$\(([^)]+)\)").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        env::var(&caps[1]).unwrap_or_else(|_| "".to_string())
    })
    .to_string()
}
fn expand_env_variables_vec(inputs: &[String]) -> Vec<String> {
    inputs.iter().map(|s| expand_env_variables(s)).collect()
}
