// winapiクレートから必要な関数・定数をインポート
use anyhow::Result;
use log::*;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::ptr::null_mut;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tempfile::Builder;
use toml;
use winapi::shared::minwindef::DWORD;
use winapi::um::jobapi2::{AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject};
use winapi::um::winnt::{
    JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

fn assign_to_job_object(child: &std::process::Child) {
    unsafe {
        let h_job = CreateJobObjectW(null_mut(), null_mut());

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        //info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        info.BasicLimitInformation.LimitFlags = 0;
        SetInformationJobObject(
            h_job,
            9, // JobObjectExtendedLimitInformation
            &mut info as *mut _ as *mut _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );

        let handle = child.as_raw_handle();
        AssignProcessToJobObject(h_job, handle as *mut winapi::ctypes::c_void);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TempData {
    config_file: String,
    program: String,
    ppid: u32,
}
impl TempData {
    fn new() -> Self {
        Self {
            config_file: String::new(),
            program: String::new(),
            ppid: 0,
        }
    }
    fn set_config_file(&mut self, config_file: String) {
        self.config_file = config_file;
    }
    fn set_program(&mut self, program: String) {
        self.program = program;
    }
    fn set_ppid(&mut self, ppid: u32) {
        self.ppid = ppid;
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
    let command_args = &args[3..];

    temp_data.set_config_file(config_file.to_string());
    temp_data.set_program(program.to_string());
    temp_data.set_ppid(std::process::id());

    let mut temp_file = Builder::new()
        .prefix(&format!("{}_{}_", self_program, program))
        .suffix(".tmp")
        .keep(true)
        .tempfile()?;

    debug!("temp file path: {:?}", temp_file.path());

    // TempData をバイナリにシリアライズして書き込み
    let encoded: Vec<u8> = bincode::serialize(&temp_data).unwrap();
    temp_file.write_all(&encoded)?;

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
    command.args(command_args);
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child: Child = command.spawn()?;
    debug!("Process started: {:?}", child.id());

    // assign_to_job_object(&child);

    let temp_path = temp_file.path().to_path_buf();

    let status = child.wait()?;
    debug!("Process exited with: {}", status);

    if let Err(e) = std::fs::remove_file(&temp_path) {
        eprintln!("Failed to delete temp file: {}", e);
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
