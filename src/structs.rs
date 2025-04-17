use anyhow::Result;
use log::*;
use serde::de::Error;
use serde::{Deserialize, Serialize};
// ====================
// 環境設定用構造体
// ====================
#[derive(Debug, Deserialize)]
pub struct Config {
    paths: Vec<String>,
    envs: Vec<EnvVar>,
}
impl Config {
	pub fn set_paths(&mut self,paths:Vec<String>) {
		self.paths = paths;
	}
	pub fn get_paths(&self) -> Vec<String>{
		self.paths.clone()
	}
	pub fn set_envs(&mut self,envs:Vec<EnvVar>){
		self.envs = envs;
	}
	pub fn get_envs(&self) -> Vec<EnvVar>{
		self.envs.clone()
	}
}
// ====================
// 環境変数用構造体
// ====================
#[derive(Debug, Deserialize,Clone)]
#[serde(untagged)]
pub enum EnvVar {
    Single(Vec<String>),
    Multiple(String, Vec<String>),
}

// ====================
// ====================
// 構造体定義部
// ====================
// ====================
#[derive(Debug, Serialize, Deserialize)]
pub struct TempData {
    parent_pid: u32,
    child_pid: u32,
    config_file: String,
    program: String,
    program_args: Vec<String>,
}
impl TempData {
    pub fn new() -> Self {
        Self {
            parent_pid: 0,
            child_pid: 0,
            config_file: String::new(),
            program: String::new(),
            program_args:Vec::new(),
        }
    }
    pub fn set_config_file(&mut self, config_file: String) {
        self.config_file = config_file;
    }
    pub fn set_program(&mut self, program: String) {
        self.program = program;
    }
    pub fn set_parent_pid(&mut self, ppid: u32) {
        self.parent_pid = ppid;
    }
    pub fn set_child_pid(&mut self, ppid: u32) {
        self.child_pid = ppid;
    }
    pub fn set_program_args(&mut self, args: Vec<String>) {
        self.program_args = args;
    }
}