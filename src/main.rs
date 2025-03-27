use std::env;
use std::fs::File;
use std::process::{Command, Stdio};
use std::io::{self, BufRead};
use toml;
use std::path::Path;
use serde::Deserialize;
use serde::de::Error;
use anyhow::Result;
use regex::Regex;

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
    // Check command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <config_file> <shell> [command...]", args[0]);
        std::process::exit(1);
    }

    // Retrieve the config file and shell
    let config_file = &args[1];
    let shell = &args[2];
    let command_args = &args[3..];

    // Load the configuration file
    let config: Config = read_toml(config_file)?;

    // Get the current "Path" environment variable
    let current_path = env::var("Path").unwrap_or_default();
    let mut new_path = current_path.clone();

    // Loop through the environment variables from the configuration
    for env_var in config.envs {
        match env_var {
            EnvVar::Single(ref env_pair) => {
                if env_pair.len() == 2 {
                    let key = &env_pair[0];
                    let value = expand_env_variables(&env_pair[1]);
                    if !key.is_empty() && !value.is_empty() {
                        env::set_var(key, value);
                    }
                }
            }
            EnvVar::Multiple(ref key, ref values) => {
                let value = values.join(";"); // Join multiple values with a separator (e.g., ';')
                env::set_var(key, value);
            }
        }
    }

    // Add the paths from the TOML configuration to the "Path" environment variable
    for path in config.paths {
        let expanded_path = expand_env_variables(&path);
        if !expanded_path.trim().is_empty() {
            new_path.push(';');
            new_path.push_str(&expanded_path);
        }
    }

    // Update the "Path" environment variable
    env::set_var("Path", new_path);

    // Initialize the command
    let mut command = Command::new(shell);
    command.args(command_args);

    // Set stdin, stdout, and stderr
    command.stdin(Stdio::inherit())
           .stdout(Stdio::inherit())
           .stderr(Stdio::inherit());

    // Execute the command
    let status = command.status()?;
    if !status.success() {
        eprintln!("Command failed with status: {:?}", status);
    }

    Ok(())
}

// Read the TOML configuration file
fn read_toml<P>(filename: P) -> Result<Config, toml::de::Error>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename).map_err(|e| toml::de::Error::custom(e.to_string()))?;
    let mut contents = String::new();
    io::Read::read_to_string(&mut file, &mut contents).unwrap();
    toml::de::from_str(&contents)
}

// Expand environment variables in the input string
fn expand_env_variables(input: &str) -> String {
    let re = Regex::new(r"\$\(([^)]+)\)").unwrap(); // Match $(VAR) pattern
    re.replace_all(input, |caps: &regex::Captures| {
        env::var(&caps[1]).unwrap_or_else(|_| "".to_string()) // Retrieve the environment variable or use an empty string
    })
    .to_string()
}




