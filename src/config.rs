use anyhow::{Context, Result};
use serde::Deserialize;
use toml;

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub problem_config: ProblemConfig,

    #[serde(default)]
    pub exec_config: ExecConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProblemConfig {
    #[serde(default)]
    pub parameter_names: Vec<String>,
}

impl Default for ProblemConfig {
    fn default() -> Self {
        ProblemConfig {
            parameter_names: Vec::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExecConfig {
    #[serde(default = "default_tester_cmd")]
    pub tester_cmd: String,

    #[serde(default)]
    pub tester_arg: Vec<String>,

    #[serde(default = "default_solver_cmd")]
    pub solver_cmd: String,

    #[serde(default)]
    pub solver_arg: Vec<String>,
}

fn default_tester_cmd() -> String {
    "./tools/target/release/tester".to_string()
}

fn default_solver_cmd() -> String {
    "./a.out".to_string()
}

impl Default for ExecConfig {
    fn default() -> Self {
        ExecConfig {
            tester_cmd: default_tester_cmd(),
            tester_arg: Vec::default(),
            solver_cmd: default_solver_cmd(),
            solver_arg: Vec::default(),
        }
    }
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
            let default_config_path = PathBuf::from("./.ahc-local-tools/config.toml");

            if default_config_path.is_file() {
                let config_toml_str =
                    fs::read_to_string(&default_config_path).with_context(|| {
                        format!(
                            "could not read config-file `{}`",
                            default_config_path.to_str().unwrap()
                        )
                    })?;

                let config: Config = toml::from_str(&config_toml_str).with_context(|| {
                    format!(
                        "could not parse config toml `{}`",
                        default_config_path.to_str().unwrap()
                    )
                })?;

                Ok(config)
            } else {
                Ok(Config::default())
            }
        }
    }
}
