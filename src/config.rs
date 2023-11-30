use anyhow::{Context, Result};

use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    tester_path: PathBuf,
    solver_path: PathBuf,
}

pub fn load_config(config_file: Option<PathBuf>) -> Result<Config> {
    match config_file {
        Some(config_path) => {
            todo!()
        }
        None => {
            todo!()
        }
    }
}
