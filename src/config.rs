use anyhow::{anyhow, Context, Result};
use dialoguer::Select;
use serde::{Deserialize, Serialize};
use toml;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde()]
    pub problem_config: ProblemConfig,

    #[serde(default)]
    pub exec_config: ExecConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ProblemScoring {
    Maximize,
    Minimize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProblemConfig {
    score: ProblemScoring,
}

#[derive(Debug, Deserialize, Serialize)]
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

pub fn initialize_config() -> Result<()> {
    let default_config_path = PathBuf::from("./.ahc-local-tools/config.toml");
    println!(
        "No config file exists in current directory (`{}`)",
        default_config_path.to_str().unwrap()
    );

    let choices = vec!["Maximize", "Minimize"];
    let scoring_selection = Select::new()
        .with_prompt("Select the scoring type in the problem")
        .items(&choices)
        .interact()?;

    let score = match scoring_selection {
        0_usize => ProblemScoring::Maximize,
        1_usize => ProblemScoring::Minimize,
        _ => return Err(anyhow!("invalid selection for problem scoring")),
    };

    let config = Config {
        problem_config: ProblemConfig { score },
        exec_config: ExecConfig::default(),
    };

    let serialized_str =
        toml::to_string_pretty(&config).with_context(|| format!("could not serialize config"))?;

    if !PathBuf::from("./.ahc-local-tools").is_dir() {
        fs::create_dir("./.ahc-local-tools")
            .with_context(|| format!("could not make directory `./.ahc-local-tools`"))?;
    }

    let mut output_file = File::create(&default_config_path).with_context(|| {
        format!(
            "could not create best-result-file `{}`",
            default_config_path.to_str().unwrap()
        )
    })?;
    output_file
        .write_all(serialized_str.as_bytes())
        .with_context(|| {
            format!(
                "could not write best result to `{}`",
                default_config_path.to_str().unwrap()
            )
        })?;

    Ok(())
}

pub fn load_config(config_file: Option<PathBuf>) -> Result<Config> {
    let config_path = match config_file {
        Some(e) => e,
        None => {
            let default_config_path = PathBuf::from("./.ahc-local-tools/config.toml");
            if !default_config_path.is_file() {
                initialize_config().with_context(|| {
                    format!(
                        "could not make config file `{}`",
                        default_config_path.to_str().unwrap()
                    )
                })?;
            }
            default_config_path
        }
    };

    let config_toml_str = fs::read_to_string(&config_path).with_context(|| {
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
