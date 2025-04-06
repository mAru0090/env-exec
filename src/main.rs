/*
// winapiクレートから必要な関数・定数をインポート
use winapi::shared::minwindef::DWORD;
use winapi::um::jobapi2::{CreateJobObjectW, SetInformationJobObject, AssignProcessToJobObject};
use winapi::um::winnt::{
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JobObjectExtendedLimitInformation,
};
use winapi::um::processthreadsapi::GetExitCodeProcess;
use winapi::um::handleapi::CloseHandle;
use winapi::um::errhandlingapi::GetLastError;
use anyhow::{anyhow, Result};
use log::*;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::Builder;
use toml;

// 追加: Windows ハンドル用
use std::ptr::null_mut;
use std::mem::size_of;
use std::os::windows::io::AsRawHandle;

#[derive(Debug, Serialize, Deserialize)]
struct TempData {
    config_file: String,
    program: String,
    ppid: String,
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

fn monitor_child_process(child: &mut Child, temp_path: &std::path::Path) {
    // プロセスの終了コードを監視
    let child_handle = child.as_raw_handle();
    let mut exit_code: DWORD = 0;

    loop {
        // 子プロセスが終了したかどうか確認
        unsafe {
            GetExitCodeProcess(child_handle as *mut winapi::ctypes::c_void, &mut exit_code);
        }

        // プロセスが終了している場合
        if exit_code != 259 { // STILL_ACTIVE (259) はプロセスがまだ実行中であることを示す
            // 終了した場合、一時ファイルを削除
            if let Err(e) = std::fs::remove_file(temp_path) {
                eprintln!("Failed to delete temp file: {}", e);
            }
            break;
        }

        // 一定時間待機して再度確認
        std::thread::sleep(Duration::from_millis(100));
    }

    unsafe {
        CloseHandle(child_handle as *mut winapi::ctypes::c_void);
    }
}


fn main() -> Result<()> {
    let _ = SimpleLogger::new().init();
    let args: Vec<String> = env::args().collect();
    let self_program = &args[0];

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

    let temp_file = Builder::new()
        .prefix(&format!("{}_{}_", self_program, program))
        .suffix(".tmp")
        .keep(true)
        .tempfile()?;

    debug!("temp file path: {:?}", temp_file.path());
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
                env::set_var(key, values.join(";"));
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

    // 子プロセスを起動する前に Job オブジェクトを作成
    let job = unsafe { CreateJobObjectW(null_mut(), null_mut()) };
    if job.is_null() {
        return Err(anyhow!("Failed to create job object"));
    }

    // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE を設定
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let ret = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &mut info as *mut _ as *mut _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD,
        )
    };
    if ret == 0 {
        return Err(anyhow!("Failed to set job information"));
    }

    let mut command = Command::new(program);
    command.args(command_args);
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child: Child = command.spawn()?;
    debug!("Process started: {:?}", child.id());

    // 子プロセスを Job オブジェクトに割り当てる
    let assign_ret = unsafe { 
    AssignProcessToJobObject(job, child.as_raw_handle() as *mut winapi::ctypes::c_void) 
    };
    if assign_ret == 0 {
    	let error_code = unsafe { GetLastError() };
    	let error_str = format!("Failed to assitgn process to job. Error code: {}",error_code);
    	error!("{}",error_str);
    	return Err(anyhow!(error_str));
   }


    let temp_path = temp_file.path().to_path_buf();
    
    // 新しいスレッドで子プロセスの終了を監視
    thread::spawn(move || monitor_child_process(&mut child, &temp_path));

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

*/
















// winapiクレートから必要な関数・定数をインポート
use winapi::shared::minwindef::DWORD;
use winapi::um::jobapi2::{CreateJobObjectW, SetInformationJobObject, AssignProcessToJobObject};
use winapi::um::winnt::{
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JobObjectExtendedLimitInformation,
};
use anyhow::Result;
use log::*;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::Builder;
use toml;

#[derive(Debug, Serialize, Deserialize)]
struct TempData {
    config_file: String,
    program: String,
    ppid: String,
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

    let temp_file = Builder::new()
        .prefix(&format!("{}_{}_", self_program, program))
        .suffix(".tmp")
        .keep(true)
        .tempfile()?;

    debug!("temp file path: {:?}", temp_file.path());
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

    let temp_path = temp_file.path().to_path_buf();
    /*
    thread::spawn(move || {
        let _ = child.wait();
        if let Err(e) = std::fs::remove_file(&temp_path) {
            eprintln!("Failed to delete temp file: {}", e);
        }
    }).join().unwrap();
    */
    thread::spawn(move || match child.wait_with_output() {
        Ok(_) => {
            if let Err(e) = std::fs::remove_file(&temp_path) {
                eprintln!("Failed to delete temp file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to wait for child process: {}", e);
        }
    })
    .join()
    .unwrap();

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

