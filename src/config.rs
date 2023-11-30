use anyhow::{Context, Result};
use serde::Deserialize;
use toml;

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    exec_config: ExecConfig,
}

#[derive(Debug, Deserialize)]
pub struct ExecConfig {
    #[serde(default = "default_tester_cmd")]
    tester_cmd: Vec<String>,

    #[serde(default = "default_solver_cmd")]
    solver_cmd: Vec<String>,
}

fn default_tester_cmd() -> Vec<String> {
    vec!["./tools/target/release/tester".to_string()]
}

fn default_solver_cmd() -> Vec<String> {
    vec!["./a.out".to_string()]
}

pub fn load_config(config_file: Option<PathBuf>) -> Result<Config> {
    match config_file {
        Some(ref config_path) => {
            let config_toml_str = fs::read_to_string(config_path).with_context(|| {
                format!(
                    "could not read config-file `{}`",
                    config_path.to_str().unwrap()
                )
            })?;

            let config: Config = toml::from_str(&config_toml_str).with_context(|| {
                format!(
                    "could not parse config toml `{}`",
                    config_path.to_str().unwrap()
                )
            })?;

            Ok(config)
        }
        None => {
            todo!()
        }
    }
}
